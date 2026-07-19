//! Local SQLite library store: schema, seeding from the shared `library.json`, and reads.
//! Kept free of Tauri types so it can be unit-tested against an in-memory DB.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Bump when the library table shape changes; a mismatch drops + rebuilds the
/// library tables on startup (they are always re-seedable from library.json).
const SCHEMA_VERSION: i64 = 2;

// ---- Shared-library seed shape (mirrors library/library.json) ----

#[derive(Deserialize)]
struct Seed {
    canonicals: Vec<SeedCanonical>,
    variants: Vec<SeedVariant>,
    #[serde(default)]
    pricing: Vec<SeedPricing>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedCanonical {
    id: String,
    code: Option<String>,
    chemical_name: Option<String>,
    description: Option<String>,
    formula: Option<String>,
    category: Option<String>,
    form_factor: Option<String>,
    nutrients: SeedNutrients,
    #[serde(default)]
    physical: SeedPhysical,
    #[serde(default)]
    incompatibility: SeedIncompat,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SeedNutrients {
    n_no3: Option<f64>, n_nh4: Option<f64>, p2o5: Option<f64>, k2o: Option<f64>,
    ca: Option<f64>, mg: Option<f64>, s: Option<f64>, fe: Option<f64>, mn: Option<f64>,
    zn: Option<f64>, b: Option<f64>, cu: Option<f64>, si: Option<f64>, mo: Option<f64>,
    na: Option<f64>, cl: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SeedPhysical {
    density_g_cm3: Option<f64>,
    solubility_g_l25c: Option<f64>,
    ec_ms_cm1g_l25c: Option<f64>,
    ph1pct25c: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SeedIncompat {
    phosphates: bool, sulfates: bool, calcium: bool, borate: bool, high_ph_stock: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedVariant {
    id: String,
    canonical_id: String,
    brand_name: Option<String>,
    manufacturer: Option<String>,
    country: Option<String>,
    form_factor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedPricing {
    id: String,
    variant_id: String,
    pack_size_kg: Option<f64>,
    market: Option<String>,
    currency: Option<String>,
    price_per_pack: Option<f64>,
    price_per_kg: Option<f64>,
}

// ---- Summary row for the Browse table (variant joined with its canonical) ----

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
    pub n_no3: f64, pub n_nh4: f64, pub p2o5: f64, pub k2o: f64,
    pub ca: f64, pub mg: f64, pub s: f64,
}

// ---- Full detail for one variant (canonical + variant + pricing) ----

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
    pub density_g_cm3: Option<f64>,
    pub solubility_g_l25c: Option<f64>,
    pub ec_ms_cm1g_l25c: Option<f64>,
    pub ph1pct25c: Option<f64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Incompatibility {
    pub phosphates: bool, pub sulfates: bool, pub calcium: bool, pub borate: bool, pub high_ph_stock: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PriceRow {
    pub pack_size_kg: Option<f64>,
    pub market: Option<String>,
    pub currency: Option<String>,
    pub price_per_pack: Option<f64>,
    pub price_per_kg: Option<f64>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VariantDetail {
    pub variant_id: String,
    pub brand_name: Option<String>,
    pub manufacturer: Option<String>,
    pub country: Option<String>,
    pub form_factor: Option<String>,
    pub chemical_name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub formula: Option<String>,
    pub category: Option<String>,
    pub nutrients: Nutrients,
    pub physical: Physical,
    pub incompatibility: Incompatibility,
    pub pricing: Vec<PriceRow>,
}

pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    Connection::open(path)
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT);")?;
    let ver: i64 = conn
        .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| r.get::<_, String>(0))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if ver != SCHEMA_VERSION {
        conn.execute_batch(
            "DROP TABLE IF EXISTS pricing; DROP TABLE IF EXISTS variant; DROP TABLE IF EXISTS canonical;",
        )?;
    }
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS canonical (
            id TEXT PRIMARY KEY, code TEXT, chemical_name TEXT, description TEXT,
            formula TEXT, category TEXT, form_factor TEXT,
            n_no3 REAL NOT NULL DEFAULT 0, n_nh4 REAL NOT NULL DEFAULT 0,
            p2o5 REAL NOT NULL DEFAULT 0, k2o REAL NOT NULL DEFAULT 0,
            ca REAL NOT NULL DEFAULT 0, mg REAL NOT NULL DEFAULT 0, s REAL NOT NULL DEFAULT 0,
            fe REAL NOT NULL DEFAULT 0, mn REAL NOT NULL DEFAULT 0, zn REAL NOT NULL DEFAULT 0,
            b REAL NOT NULL DEFAULT 0, cu REAL NOT NULL DEFAULT 0, si REAL NOT NULL DEFAULT 0,
            mo REAL NOT NULL DEFAULT 0, na REAL NOT NULL DEFAULT 0, cl REAL NOT NULL DEFAULT 0,
            density_g_cm3 REAL, solubility_g_l25c REAL, ec_ms_cm1g_l25c REAL, ph1pct25c REAL,
            inc_phosphates INTEGER NOT NULL DEFAULT 0, inc_sulfates INTEGER NOT NULL DEFAULT 0,
            inc_calcium INTEGER NOT NULL DEFAULT 0, inc_borate INTEGER NOT NULL DEFAULT 0,
            inc_high_ph INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS variant (
            id TEXT PRIMARY KEY, canonical_id TEXT NOT NULL, brand_name TEXT,
            manufacturer TEXT, country TEXT, form_factor TEXT
        );
        CREATE TABLE IF NOT EXISTS pricing (
            id TEXT PRIMARY KEY, variant_id TEXT NOT NULL, pack_size_kg REAL,
            market TEXT, currency TEXT, price_per_pack REAL, price_per_kg REAL
        );
        CREATE INDEX IF NOT EXISTS idx_variant_canonical ON variant(canonical_id);
        CREATE INDEX IF NOT EXISTS idx_pricing_variant ON pricing(variant_id);
        "#,
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION.to_string()],
    )?;
    Ok(())
}

/// Replace the library tables with the contents of a `library.json` string.
pub fn load_library(conn: &mut Connection, json: &str) -> Result<(usize, usize), String> {
    let seed: Seed = serde_json::from_str(json).map_err(|e| format!("parse library.json: {e}"))?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM pricing", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM variant", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM canonical", []).map_err(|e| e.to_string())?;
    for c in &seed.canonicals {
        let n = &c.nutrients;
        let p = &c.physical;
        let i = &c.incompatibility;
        tx.execute(
            "INSERT OR REPLACE INTO canonical
             (id, code, chemical_name, description, formula, category, form_factor,
              n_no3, n_nh4, p2o5, k2o, ca, mg, s, fe, mn, zn, b, cu, si, mo, na, cl,
              density_g_cm3, solubility_g_l25c, ec_ms_cm1g_l25c, ph1pct25c,
              inc_phosphates, inc_sulfates, inc_calcium, inc_borate, inc_high_ph)
             VALUES (?1,?2,?3,?4,?5,?6,?7, ?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,
                     ?24,?25,?26,?27, ?28,?29,?30,?31,?32)",
            params![
                c.id, c.code, c.chemical_name, c.description, c.formula, c.category, c.form_factor,
                z(n.n_no3), z(n.n_nh4), z(n.p2o5), z(n.k2o), z(n.ca), z(n.mg), z(n.s), z(n.fe),
                z(n.mn), z(n.zn), z(n.b), z(n.cu), z(n.si), z(n.mo), z(n.na), z(n.cl),
                p.density_g_cm3, p.solubility_g_l25c, p.ec_ms_cm1g_l25c, p.ph1pct25c,
                i.phosphates as i64, i.sulfates as i64, i.calcium as i64, i.borate as i64, i.high_ph_stock as i64,
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    for v in &seed.variants {
        tx.execute(
            "INSERT OR REPLACE INTO variant (id, canonical_id, brand_name, manufacturer, country, form_factor)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![v.id, v.canonical_id, v.brand_name, v.manufacturer, v.country, v.form_factor],
        )
        .map_err(|e| e.to_string())?;
    }
    for pr in &seed.pricing {
        tx.execute(
            "INSERT OR REPLACE INTO pricing (id, variant_id, pack_size_kg, market, currency, price_per_pack, price_per_kg)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![pr.id, pr.variant_id, pr.pack_size_kg, pr.market, pr.currency, pr.price_per_pack, pr.price_per_kg],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok((seed.canonicals.len(), seed.variants.len()))
}

fn z(v: Option<f64>) -> f64 {
    v.unwrap_or(0.0)
}

pub fn seed_if_empty(conn: &mut Connection, json: &str) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM variant", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count == 0 {
        load_library(conn, json)?;
    }
    Ok(())
}

pub fn list_library(conn: &Connection) -> Result<Vec<LibraryRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT v.id, v.brand_name, v.manufacturer, v.country,
                    c.chemical_name, c.code, c.category,
                    c.n_no3, c.n_nh4, c.p2o5, c.k2o, c.ca, c.mg, c.s
             FROM variant v JOIN canonical c ON c.id = v.canonical_id
             ORDER BY c.chemical_name, v.brand_name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LibraryRow {
                variant_id: row.get(0)?, brand_name: row.get(1)?, manufacturer: row.get(2)?,
                country: row.get(3)?, chemical_name: row.get(4)?, code: row.get(5)?,
                category: row.get(6)?, n_no3: row.get(7)?, n_nh4: row.get(8)?, p2o5: row.get(9)?,
                k2o: row.get(10)?, ca: row.get(11)?, mg: row.get(12)?, s: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())
}

pub fn variant_detail(conn: &Connection, variant_id: &str) -> Result<Option<VariantDetail>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT v.id, v.brand_name, v.manufacturer, v.country, v.form_factor,
                    c.chemical_name, c.code, c.description, c.formula, c.category,
                    c.n_no3, c.n_nh4, c.p2o5, c.k2o, c.ca, c.mg, c.s, c.fe, c.mn, c.zn,
                    c.b, c.cu, c.si, c.mo, c.na, c.cl,
                    c.density_g_cm3, c.solubility_g_l25c, c.ec_ms_cm1g_l25c, c.ph1pct25c,
                    c.inc_phosphates, c.inc_sulfates, c.inc_calcium, c.inc_borate, c.inc_high_ph
             FROM variant v JOIN canonical c ON c.id = v.canonical_id WHERE v.id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let detail = stmt
        .query_row(params![variant_id], |r| {
            Ok(VariantDetail {
                variant_id: r.get(0)?, brand_name: r.get(1)?, manufacturer: r.get(2)?,
                country: r.get(3)?, form_factor: r.get(4)?, chemical_name: r.get(5)?,
                code: r.get(6)?, description: r.get(7)?, formula: r.get(8)?, category: r.get(9)?,
                nutrients: Nutrients {
                    n_no3: r.get(10)?, n_nh4: r.get(11)?, p2o5: r.get(12)?, k2o: r.get(13)?,
                    ca: r.get(14)?, mg: r.get(15)?, s: r.get(16)?, fe: r.get(17)?, mn: r.get(18)?,
                    zn: r.get(19)?, b: r.get(20)?, cu: r.get(21)?, si: r.get(22)?, mo: r.get(23)?,
                    na: r.get(24)?, cl: r.get(25)?,
                },
                physical: Physical {
                    density_g_cm3: r.get(26)?, solubility_g_l25c: r.get(27)?,
                    ec_ms_cm1g_l25c: r.get(28)?, ph1pct25c: r.get(29)?,
                },
                incompatibility: Incompatibility {
                    phosphates: r.get::<_, i64>(30)? != 0, sulfates: r.get::<_, i64>(31)? != 0,
                    calcium: r.get::<_, i64>(32)? != 0, borate: r.get::<_, i64>(33)? != 0,
                    high_ph_stock: r.get::<_, i64>(34)? != 0,
                },
                pricing: Vec::new(),
            })
        })
        .ok();

    let mut detail = match detail {
        Some(d) => d,
        None => return Ok(None),
    };

    let mut ps = conn
        .prepare(
            "SELECT pack_size_kg, market, currency, price_per_pack, price_per_kg
             FROM pricing WHERE variant_id = ?1 ORDER BY pack_size_kg",
        )
        .map_err(|e| e.to_string())?;
    let prices = ps
        .query_map(params![variant_id], |r| {
            Ok(PriceRow {
                pack_size_kg: r.get(0)?, market: r.get(1)?, currency: r.get(2)?,
                price_per_pack: r.get(3)?, price_per_kg: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
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
    fn seeds_and_lists_real_data() {
        let conn = seeded();
        let rows = list_library(&conn).unwrap();
        assert_eq!(rows.len(), 22, "every variant should join to a canonical");
        assert!(rows.iter().any(|r| r.code.as_deref() == Some("sop-k2so4") && r.k2o == 50.0));
    }

    #[test]
    fn variant_detail_returns_full_record() {
        let conn = seeded();
        // pick any variant and fetch its detail
        let some_id: String = conn
            .query_row("SELECT v.id FROM variant v JOIN canonical c ON c.id=v.canonical_id WHERE c.code='sop-k2so4' LIMIT 1", [], |r| r.get(0))
            .unwrap();
        let d = variant_detail(&conn, &some_id).unwrap().expect("detail present");
        assert_eq!(d.code.as_deref(), Some("sop-k2so4"));
        assert_eq!(d.nutrients.k2o, 50.0);
        assert_eq!(d.nutrients.s, 18.0);
        assert!(variant_detail(&conn, "does-not-exist").unwrap().is_none());
    }
}
