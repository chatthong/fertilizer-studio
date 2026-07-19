//! Local SQLite library store: schema, seeding from the shared `library.json`, and reads.
//! Kept free of Tauri types so it can be unit-tested against an in-memory DB.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ---- Shared-library seed shape (mirrors library/library.json) ----

#[derive(Deserialize)]
struct Seed {
    canonicals: Vec<SeedCanonical>,
    variants: Vec<SeedVariant>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedCanonical {
    id: String,
    code: Option<String>,
    chemical_name: Option<String>,
    category: Option<String>,
    form_factor: Option<String>,
    nutrients: SeedNutrients,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedNutrients {
    n_no3: Option<f64>,
    n_nh4: Option<f64>,
    p2o5: Option<f64>,
    k2o: Option<f64>,
    ca: Option<f64>,
    mg: Option<f64>,
    s: Option<f64>,
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

// ---- Row returned to the frontend (variant joined with its canonical) ----

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
    pub n_no3: f64,
    pub n_nh4: f64,
    pub p2o5: f64,
    pub k2o: f64,
    pub ca: f64,
    pub mg: f64,
    pub s: f64,
}

pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    Connection::open(path)
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS canonical (
            id           TEXT PRIMARY KEY,
            code         TEXT,
            chemical_name TEXT,
            category     TEXT,
            form_factor  TEXT,
            n_no3 REAL NOT NULL DEFAULT 0,
            n_nh4 REAL NOT NULL DEFAULT 0,
            p2o5  REAL NOT NULL DEFAULT 0,
            k2o   REAL NOT NULL DEFAULT 0,
            ca    REAL NOT NULL DEFAULT 0,
            mg    REAL NOT NULL DEFAULT 0,
            s     REAL NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS variant (
            id           TEXT PRIMARY KEY,
            canonical_id TEXT NOT NULL,
            brand_name   TEXT,
            manufacturer TEXT,
            country      TEXT,
            form_factor  TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_variant_canonical ON variant(canonical_id);
        "#,
    )
}

/// Replace the library tables with the contents of a `library.json` string.
/// Idempotent: used both for first-run seeding and for refreshing after a GitHub pull.
pub fn load_library(conn: &mut Connection, json: &str) -> Result<(usize, usize), String> {
    let seed: Seed = serde_json::from_str(json).map_err(|e| format!("parse library.json: {e}"))?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM variant", []).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM canonical", []).map_err(|e| e.to_string())?;
    for c in &seed.canonicals {
        let n = &c.nutrients;
        tx.execute(
            "INSERT OR REPLACE INTO canonical
             (id, code, chemical_name, category, form_factor, n_no3, n_nh4, p2o5, k2o, ca, mg, s)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                c.id, c.code, c.chemical_name, c.category, c.form_factor,
                n.n_no3.unwrap_or(0.0), n.n_nh4.unwrap_or(0.0), n.p2o5.unwrap_or(0.0),
                n.k2o.unwrap_or(0.0), n.ca.unwrap_or(0.0), n.mg.unwrap_or(0.0), n.s.unwrap_or(0.0),
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    for v in &seed.variants {
        tx.execute(
            "INSERT OR REPLACE INTO variant
             (id, canonical_id, brand_name, manufacturer, country, form_factor)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![v.id, v.canonical_id, v.brand_name, v.manufacturer, v.country, v.form_factor],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok((seed.canonicals.len(), seed.variants.len()))
}

/// Seed only if the library is empty (first run).
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
             FROM variant v
             JOIN canonical c ON c.id = v.canonical_id
             ORDER BY c.chemical_name, v.brand_name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LibraryRow {
                variant_id: row.get(0)?,
                brand_name: row.get(1)?,
                manufacturer: row.get(2)?,
                country: row.get(3)?,
                chemical_name: row.get(4)?,
                code: row.get(5)?,
                category: row.get(6)?,
                n_no3: row.get(7)?,
                n_nh4: row.get(8)?,
                p2o5: row.get(9)?,
                k2o: row.get(10)?,
                ca: row.get(11)?,
                mg: row.get(12)?,
                s: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = include_str!("../../library/library.json");

    #[test]
    fn seeds_and_lists_real_data() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let (ncan, nvar) = load_library(&mut conn, SEED).unwrap();
        assert_eq!(ncan, 29, "expected 29 canonicals from the rescued dump");
        assert_eq!(nvar, 22, "expected 22 variants from the rescued dump");

        let rows = list_library(&conn).unwrap();
        assert_eq!(rows.len(), 22, "every variant should join to a canonical");
        // Known rescued row: Potassium Sulfate (SOP), K2O 50 / S 18
        assert!(
            rows.iter().any(|r| r.code.as_deref() == Some("sop-k2so4") && r.k2o == 50.0),
            "expected the SOP canonical (k2o=50) in the joined results"
        );
    }

    #[test]
    fn seed_if_empty_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        seed_if_empty(&mut conn, SEED).unwrap();
        seed_if_empty(&mut conn, SEED).unwrap(); // second call is a no-op
        assert_eq!(list_library(&conn).unwrap().len(), 22);
    }
}
