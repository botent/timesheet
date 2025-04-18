use tauri::{Manager, Runtime};

pub mod database;
pub mod tasks;
pub mod tray;
pub mod ui;

use crate::commands::tray::{set_tray_icon, TrayState};
use crate::helpers::database::create_task_sessions_table;
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
        create_task_sessions_table(&conn)?;
    }

    app_handle.manage(DbState { pool });

    // Initialize the tray state
    let tray_state = TrayState::new();
    app_handle.manage(tray_state);

    // Initialize the tray icon
    set_tray_icon(&app_handle).map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    Ok(())
}
