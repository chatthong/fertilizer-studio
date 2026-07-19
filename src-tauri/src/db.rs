//! Local SQLite library store: schema, seeding from the shared `library.json`, and reads.
//! Kept free of Tauri types so it can be unit-tested against an in-memory DB.
//!
//! Nutrients are stored at the VARIANT level: `library.json` resolves each variant's
//! nutrients as the canonical base with nutrient overrides applied (matching legacy
//! production), so two variants of one canonical can differ. Seed structs use
//! `deny_unknown_fields` so any drift between library.json and this loader fails loudly
//! instead of silently dropping data.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: i64 = 3;

// ---- Shared-library seed shape (mirrors library/library.json, field-for-field) ----

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Seed {
    #[allow(dead_code)] schema_version: i64,
    #[allow(dead_code)] source: String,
    #[allow(dead_code)] override_policy: String,
    #[allow(dead_code)] counts: serde_json::Value,
    canonicals: Vec<SeedCanonical>,
    aliases: Vec<SeedAlias>,
    variants: Vec<SeedVariant>,
    pricing: Vec<SeedPricing>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedCanonical {
    id: String,
    code: Option<String>,
    chemical_name: Option<String>,
    description: Option<String>,
    formula: Option<String>,
    cas: Option<String>,
    category: Option<String>,
    form_factor: Option<String>,
    status: Option<String>,
    #[allow(dead_code)] image_url: Option<String>,
    // canonical base nutrients — carried in library.json; the app reads VARIANT nutrients
    #[allow(dead_code)] nutrients: SeedNutrients,
    physical: SeedPhysical,
    assay: SeedAssay,
    incompatibility: SeedIncompat,
    // carried in library.json; not columned (kept in the committed source of truth)
    #[allow(dead_code)] compatibility_notes: Option<String>,
    #[allow(dead_code)] certs_labels: Vec<String>,
    #[allow(dead_code)] hazard_ghs_codes: Vec<String>,
    #[allow(dead_code)] hazard_notes: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedNutrients {
    n_no3: Option<f64>, n_nh4: Option<f64>, p2o5: Option<f64>, k2o: Option<f64>,
    ca: Option<f64>, mg: Option<f64>, s: Option<f64>, fe: Option<f64>, mn: Option<f64>,
    zn: Option<f64>, b: Option<f64>, cu: Option<f64>, si: Option<f64>, mo: Option<f64>,
    na: Option<f64>, cl: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedPhysical {
    density_g_cm3: Option<f64>,
    #[allow(dead_code)] bulk_density_kg_m3: Option<f64>,
    solubility_g_l25c: Option<f64>,
    ec_ms_cm1g_l25c: Option<f64>,
    ph1pct25c: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedAssay {
    purity_pct: Option<f64>,
    moisture_pct: Option<f64>,
    insoluble_pct: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedIncompat {
    phosphates: bool, sulfates: bool, calcium: bool, borate: bool, high_ph_stock: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedAlias {
    canonical_id: String,
    alias: String,
    alias_type: Option<String>,
    lang_code: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedVariant {
    id: String,
    canonical_id: String,
    status: Option<String>,
    manufacturer: Option<String>,
    brand_name: Option<String>,
    model_sku: Option<String>,
    country: Option<String>,
    #[allow(dead_code)] grade_label: Option<String>,
    form_factor: Option<String>,
    #[allow(dead_code)] image_url: Option<String>,
    #[allow(dead_code)] packaging_pack_sizes_kg: Vec<f64>,
    #[allow(dead_code)] packaging_shelf_life_months: Option<i64>,
    #[allow(dead_code)] certs_labels: Vec<String>,
    #[allow(dead_code)] hazard_labels: Vec<String>,
    nutrients: SeedNutrients,
    has_overrides: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedPricing {
    id: String,
    variant_id: String,
    pack_size_kg: Option<f64>,
    market: Option<String>,
    #[allow(dead_code)] market_region: Option<String>,
    currency: Option<String>,
    price_per_pack: Option<f64>,
    price_per_kg: Option<f64>,
    incoterm: Option<String>,
    #[allow(dead_code)] min_order_pack: Option<i64>,
    #[allow(dead_code)] lead_time_days: Option<i64>,
    #[allow(dead_code)] last_update: Option<String>,
    price_reference: Option<String>,
    #[allow(dead_code)] data_source: Option<String>,
}

// ---- Outputs to the frontend ----

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRow {
    pub variant_id: String,
    pub brand_name: Option<String>,
    pub manufacturer: Option<String>,
    pub country: Option<String>,
    pub chemical_name: Option<String>,
    pub code: Option<String>,
    pub category: Option<String>,
    pub has_overrides: bool,
    pub n_no3: f64, pub n_nh4: f64, pub p2o5: f64, pub k2o: f64,
    pub ca: f64, pub mg: f64, pub s: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Nutrients {
    pub n_no3: f64, pub n_nh4: f64, pub p2o5: f64, pub k2o: f64, pub ca: f64, pub mg: f64,
    pub s: f64, pub fe: f64, pub mn: f64, pub zn: f64, pub b: f64, pub cu: f64, pub si: f64,
    pub mo: f64, pub na: f64, pub cl: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Physical {
    pub density_g_cm3: Option<f64>, pub solubility_g_l25c: Option<f64>,
    pub ec_ms_cm1g_l25c: Option<f64>, pub ph1pct25c: Option<f64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Assay {
    pub purity_pct: Option<f64>, pub moisture_pct: Option<f64>, pub insoluble_pct: Option<f64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Incompatibility {
    pub phosphates: bool, pub sulfates: bool, pub calcium: bool, pub borate: bool, pub high_ph_stock: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceRow {
    pub pack_size_kg: Option<f64>, pub market: Option<String>, pub currency: Option<String>,
    pub price_per_pack: Option<f64>, pub price_per_kg: Option<f64>,
    pub incoterm: Option<String>, pub price_reference: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VariantDetail {
    pub variant_id: String, pub brand_name: Option<String>, pub manufacturer: Option<String>,
    pub country: Option<String>, pub form_factor: Option<String>, pub status: Option<String>,
    pub has_overrides: bool,
    pub chemical_name: Option<String>, pub code: Option<String>, pub cas: Option<String>,
    pub description: Option<String>, pub formula: Option<String>, pub category: Option<String>,
    pub nutrients: Nutrients, pub physical: Physical, pub assay: Assay,
    pub incompatibility: Incompatibility, pub pricing: Vec<PriceRow>,
}

pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    Connection::open(path)
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT);")?;
    let ver: i64 = conn
        .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| r.get::<_, String>(0))
        .ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    if ver != SCHEMA_VERSION {
        conn.execute_batch(
            "DROP TABLE IF EXISTS alias; DROP TABLE IF EXISTS pricing; DROP TABLE IF EXISTS variant; DROP TABLE IF EXISTS canonical;",
        )?;
    }
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS canonical (
            id TEXT PRIMARY KEY, code TEXT, chemical_name TEXT, description TEXT, formula TEXT,
            cas TEXT, category TEXT, form_factor TEXT, status TEXT,
            density_g_cm3 REAL, solubility_g_l25c REAL, ec_ms_cm1g_l25c REAL, ph1pct25c REAL,
            assay_purity REAL, assay_moisture REAL, assay_insoluble REAL,
            inc_phosphates INTEGER NOT NULL DEFAULT 0, inc_sulfates INTEGER NOT NULL DEFAULT 0,
            inc_calcium INTEGER NOT NULL DEFAULT 0, inc_borate INTEGER NOT NULL DEFAULT 0,
            inc_high_ph INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS variant (
            id TEXT PRIMARY KEY, canonical_id TEXT NOT NULL, status TEXT, brand_name TEXT,
            manufacturer TEXT, model_sku TEXT, country TEXT, form_factor TEXT, has_overrides INTEGER NOT NULL DEFAULT 0,
            n_no3 REAL NOT NULL DEFAULT 0, n_nh4 REAL NOT NULL DEFAULT 0, p2o5 REAL NOT NULL DEFAULT 0,
            k2o REAL NOT NULL DEFAULT 0, ca REAL NOT NULL DEFAULT 0, mg REAL NOT NULL DEFAULT 0,
            s REAL NOT NULL DEFAULT 0, fe REAL NOT NULL DEFAULT 0, mn REAL NOT NULL DEFAULT 0,
            zn REAL NOT NULL DEFAULT 0, b REAL NOT NULL DEFAULT 0, cu REAL NOT NULL DEFAULT 0,
            si REAL NOT NULL DEFAULT 0, mo REAL NOT NULL DEFAULT 0, na REAL NOT NULL DEFAULT 0, cl REAL NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS pricing (
            id TEXT PRIMARY KEY, variant_id TEXT NOT NULL, pack_size_kg REAL, market TEXT,
            currency TEXT, price_per_pack REAL, price_per_kg REAL, incoterm TEXT, price_reference TEXT
        );
        CREATE TABLE IF NOT EXISTS alias (
            canonical_id TEXT NOT NULL, alias TEXT NOT NULL, alias_type TEXT, lang_code TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_variant_canonical ON variant(canonical_id);
        CREATE INDEX IF NOT EXISTS idx_pricing_variant ON pricing(variant_id);
        CREATE INDEX IF NOT EXISTS idx_alias_canonical ON alias(canonical_id);
        "#,
    )?;
    conn.execute("INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)", params![SCHEMA_VERSION.to_string()])?;
    Ok(())
}

fn z(v: Option<f64>) -> f64 { v.unwrap_or(0.0) }

pub fn load_library(conn: &mut Connection, json: &str) -> Result<(usize, usize), String> {
    let seed: Seed = serde_json::from_str(json).map_err(|e| format!("parse library.json: {e}"))?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for t in ["alias", "pricing", "variant", "canonical"] {
        tx.execute(&format!("DELETE FROM {t}"), []).map_err(|e| e.to_string())?;
    }
    for c in &seed.canonicals {
        let (p, a, i) = (&c.physical, &c.assay, &c.incompatibility);
        tx.execute(
            "INSERT OR REPLACE INTO canonical
             (id, code, chemical_name, description, formula, cas, category, form_factor, status,
              density_g_cm3, solubility_g_l25c, ec_ms_cm1g_l25c, ph1pct25c,
              assay_purity, assay_moisture, assay_insoluble,
              inc_phosphates, inc_sulfates, inc_calcium, inc_borate, inc_high_ph)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)",
            params![
                c.id, c.code, c.chemical_name, c.description, c.formula, c.cas, c.category, c.form_factor, c.status,
                p.density_g_cm3, p.solubility_g_l25c, p.ec_ms_cm1g_l25c, p.ph1pct25c,
                a.purity_pct, a.moisture_pct, a.insoluble_pct,
                i.phosphates as i64, i.sulfates as i64, i.calcium as i64, i.borate as i64, i.high_ph_stock as i64,
            ],
        ).map_err(|e| e.to_string())?;
    }
    for v in &seed.variants {
        let n = &v.nutrients;
        tx.execute(
            "INSERT OR REPLACE INTO variant
             (id, canonical_id, status, brand_name, manufacturer, model_sku, country, form_factor, has_overrides,
              n_no3, n_nh4, p2o5, k2o, ca, mg, s, fe, mn, zn, b, cu, si, mo, na, cl)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9, ?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
            params![
                v.id, v.canonical_id, v.status, v.brand_name, v.manufacturer, v.model_sku, v.country, v.form_factor, v.has_overrides as i64,
                z(n.n_no3), z(n.n_nh4), z(n.p2o5), z(n.k2o), z(n.ca), z(n.mg), z(n.s), z(n.fe), z(n.mn),
                z(n.zn), z(n.b), z(n.cu), z(n.si), z(n.mo), z(n.na), z(n.cl),
            ],
        ).map_err(|e| e.to_string())?;
    }
    for pr in &seed.pricing {
        tx.execute(
            "INSERT OR REPLACE INTO pricing (id, variant_id, pack_size_kg, market, currency, price_per_pack, price_per_kg, incoterm, price_reference)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![pr.id, pr.variant_id, pr.pack_size_kg, pr.market, pr.currency, pr.price_per_pack, pr.price_per_kg, pr.incoterm, pr.price_reference],
        ).map_err(|e| e.to_string())?;
    }
    for al in &seed.aliases {
        tx.execute(
            "INSERT INTO alias (canonical_id, alias, alias_type, lang_code) VALUES (?1,?2,?3,?4)",
            params![al.canonical_id, al.alias, al.alias_type, al.lang_code],
        ).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok((seed.canonicals.len(), seed.variants.len()))
}

pub fn seed_if_empty(conn: &mut Connection, json: &str) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM variant", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    if count == 0 {
        load_library(conn, json)?;
    }
    Ok(())
}

pub fn list_library(conn: &Connection) -> Result<Vec<LibraryRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT v.id, v.brand_name, v.manufacturer, v.country, c.chemical_name, c.code, c.category,
                    v.has_overrides, v.n_no3, v.n_nh4, v.p2o5, v.k2o, v.ca, v.mg, v.s
             FROM variant v JOIN canonical c ON c.id = v.canonical_id
             ORDER BY c.chemical_name, v.brand_name",
        ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LibraryRow {
                variant_id: row.get(0)?, brand_name: row.get(1)?, manufacturer: row.get(2)?,
                country: row.get(3)?, chemical_name: row.get(4)?, code: row.get(5)?, category: row.get(6)?,
                has_overrides: row.get::<_, i64>(7)? != 0,
                n_no3: row.get(8)?, n_nh4: row.get(9)?, p2o5: row.get(10)?, k2o: row.get(11)?,
                ca: row.get(12)?, mg: row.get(13)?, s: row.get(14)?,
            })
        }).map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())
}

pub fn variant_detail(conn: &Connection, variant_id: &str) -> Result<Option<VariantDetail>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT v.id, v.brand_name, v.manufacturer, v.country, v.form_factor, v.status, v.has_overrides,
                    c.chemical_name, c.code, c.cas, c.description, c.formula, c.category,
                    v.n_no3, v.n_nh4, v.p2o5, v.k2o, v.ca, v.mg, v.s, v.fe, v.mn, v.zn, v.b, v.cu, v.si, v.mo, v.na, v.cl,
                    c.density_g_cm3, c.solubility_g_l25c, c.ec_ms_cm1g_l25c, c.ph1pct25c,
                    c.assay_purity, c.assay_moisture, c.assay_insoluble,
                    c.inc_phosphates, c.inc_sulfates, c.inc_calcium, c.inc_borate, c.inc_high_ph
             FROM variant v JOIN canonical c ON c.id = v.canonical_id WHERE v.id = ?1",
        ).map_err(|e| e.to_string())?;
    let detail = stmt
        .query_row(params![variant_id], |r| {
            Ok(VariantDetail {
                variant_id: r.get(0)?, brand_name: r.get(1)?, manufacturer: r.get(2)?, country: r.get(3)?,
                form_factor: r.get(4)?, status: r.get(5)?, has_overrides: r.get::<_, i64>(6)? != 0,
                chemical_name: r.get(7)?, code: r.get(8)?, cas: r.get(9)?, description: r.get(10)?,
                formula: r.get(11)?, category: r.get(12)?,
                nutrients: Nutrients {
                    n_no3: r.get(13)?, n_nh4: r.get(14)?, p2o5: r.get(15)?, k2o: r.get(16)?, ca: r.get(17)?,
                    mg: r.get(18)?, s: r.get(19)?, fe: r.get(20)?, mn: r.get(21)?, zn: r.get(22)?, b: r.get(23)?,
                    cu: r.get(24)?, si: r.get(25)?, mo: r.get(26)?, na: r.get(27)?, cl: r.get(28)?,
                },
                physical: Physical {
                    density_g_cm3: r.get(29)?, solubility_g_l25c: r.get(30)?, ec_ms_cm1g_l25c: r.get(31)?, ph1pct25c: r.get(32)?,
                },
                assay: Assay { purity_pct: r.get(33)?, moisture_pct: r.get(34)?, insoluble_pct: r.get(35)? },
                incompatibility: Incompatibility {
                    phosphates: r.get::<_, i64>(36)? != 0, sulfates: r.get::<_, i64>(37)? != 0,
                    calcium: r.get::<_, i64>(38)? != 0, borate: r.get::<_, i64>(39)? != 0, high_ph_stock: r.get::<_, i64>(40)? != 0,
                },
                pricing: Vec::new(),
            })
        }).ok();
    let mut detail = match detail { Some(d) => d, None => return Ok(None) };

    let mut ps = conn
        .prepare("SELECT pack_size_kg, market, currency, price_per_pack, price_per_kg, incoterm, price_reference FROM pricing WHERE variant_id = ?1 ORDER BY pack_size_kg")
        .map_err(|e| e.to_string())?;
    let prices = ps.query_map(params![variant_id], |r| {
        Ok(PriceRow {
            pack_size_kg: r.get(0)?, market: r.get(1)?, currency: r.get(2)?, price_per_pack: r.get(3)?,
            price_per_kg: r.get(4)?, incoterm: r.get(5)?, price_reference: r.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    detail.pricing = prices.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
    Ok(Some(detail))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = include_str!("../../library/library.json");

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        load_library(&mut conn, SEED).unwrap();
        conn
    }

    #[test]
    fn seeds_real_data_without_archived() {
        let conn = seeded();
        let rows = list_library(&conn).unwrap();
        assert_eq!(rows.len(), 20, "22 variants minus 2 archived");
        // no 'test' brand junk leaked through
        assert!(!rows.iter().any(|r| r.brand_name.as_deref() == Some("test")));
        let aliases: i64 = conn.query_row("SELECT COUNT(*) FROM alias", [], |r| r.get(0)).unwrap();
        assert_eq!(aliases, 243, "aliases carried");
    }

    #[test]
    fn nutrient_override_applied_at_variant_level() {
        let conn = seeded();
        // Haifa SOP GG variant carries a K2O override 50 -> 51 (matches legacy production)
        let d = conn
            .query_row(
                "SELECT id FROM variant WHERE brand_name LIKE 'Haifa SOP%' LIMIT 1",
                [], |r| r.get::<_, String>(0),
            )
            .unwrap();
        let detail = variant_detail(&conn, &d).unwrap().expect("detail");
        assert_eq!(detail.nutrients.k2o, 51.0, "override 50->51 applied at variant level");
        assert!(detail.has_overrides);
        assert!(variant_detail(&conn, "nope").unwrap().is_none());
    }
}
