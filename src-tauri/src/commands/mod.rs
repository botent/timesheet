use tauri::{Manager, Runtime};

pub mod database;
pub mod tasks;
pub mod ui;

use crate::models::DbState;
use r2d2_sqlite::SqliteConnectionManager;

pub fn setup_app<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle();
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to find app data dir");

    std::fs::create_dir_all(&app_data_dir)?;

    let db_file_path = app_data_dir.join("timesheet.sqlite");

    let manager = SqliteConnectionManager::file(&db_file_path);
    let pool = r2d2::Pool::builder().max_size(10).build(manager)?;

    {
        let conn = pool.get()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id STRING PRIMARY KEY,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                created TEXT
            )",
            [],
        )?;
    }

    app_handle.manage(DbState { pool });

    Ok(())
}
