use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::fs;
use tauri::{AppHandle, Manager, Runtime, State};

use crate::models::DbState;

#[tauri::command]
pub fn reset_database<R: Runtime>(
    state: State<'_, DbState>,
    manager: AppHandle<R>,
) -> Result<(), String> {
    let app_data_dir = manager
        .path()
        .app_data_dir()
        .expect("Failed to find app data dir");

    let db_file_path = app_data_dir.join("timesheet.sqlite");

    if db_file_path.exists() {
        if let Err(e) = fs::remove_file(&db_file_path) {
            return Err(format!("Failed to delete database file: {}", e));
        }
        println!("Existing database file deleted.");
    }

    let conn_result = Connection::open(&db_file_path);

    let conn = match conn_result {
        Ok(connection) => connection,
        Err(e) => return Err(format!("Failed to open database connection: {}", e)),
    };

    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            created TEXT
        )",
        [],
    ) {
        return Err(format!("Failed to create table: {}", e));
    }
    println!("New database and table created.");

    let manager_for_conn = SqliteConnectionManager::file(&db_file_path);
    let pool = r2d2::Pool::builder()
        .max_size(10)
        .build(manager_for_conn)
        .map_err(|e| format!("Failed to create DB pool: {}", e))?;

    let mut db_state = state.inner().clone();
    db_state.pool = pool;

    manager.manage(db_state);

    manager.request_restart();

    Ok(())
}
