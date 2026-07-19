mod db;

use std::sync::Mutex;
use tauri::Manager;

/// The shared fertilizer library, embedded at build time as the first-run seed.
/// A later phase pulls a fresher copy from the public GitHub repo and calls `load_library`.
const SEED: &str = include_str!("../../library/library.json");

/// SQLite connection wrapped for Tauri's managed state (Connection is Send but not Sync).
struct Db(Mutex<rusqlite::Connection>);

#[tauri::command]
fn list_library(state: tauri::State<Db>) -> Result<Vec<db::LibraryRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::list_library(&conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("fertilizer-studio.db");
            let mut conn = db::open(&db_path)?;
            db::init_schema(&conn)?;
            db::seed_if_empty(&mut conn, SEED).map_err(std::io::Error::other)?;
            app.manage(Db(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_library])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
