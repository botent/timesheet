use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::fs;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::window::Window;
use tauri::{AppHandle, Manager, Runtime, State};
use uuid::Uuid;

mod models;
use crate::models::{DbState, UserTask};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_title(window: Window, title: String) {
    if let Err(e) = window.set_title(&title) {
        eprintln!("error updating title: {}", e);
    }
}

#[tauri::command]
fn create_task(task_data: UserTask, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state
        .pool
        .get()
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    let task_id = Uuid::new_v4().to_string();

    match conn.execute(
        "INSERT INTO tasks (id, title, status, created) VALUES (?1, ?2, ?3, ?4)",
        params![
            task_id,
            task_data.title,
            task_data.status,
            task_data.created
        ],
    ) {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                println!("Task inserted successfully.");
                Ok(()) // Return Ok(()) on success
            } else {
                eprintln!("Task insert affected 0 rows.");
                Err("Insert failed: 0 rows affected".to_string())
            }
        }
        Err(e) => {
            eprintln!("Failed to insert task: {}", e);
            Err(format!("Database error: {}", e)) // Return Err with a message string
        }
    }
}

#[tauri::command]
fn display_tasks(state: State<'_, DbState>) -> Result<Vec<UserTask>, String> {
    let conn = state
        .pool
        .get()
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, title, status, created FROM tasks")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let task_iter = stmt
        .query_map([], |row| {
            Ok(UserTask {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                created: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let tasks: Result<Vec<UserTask>, rusqlite::Error> = task_iter.collect();

    tasks.map_err(|e| format!("Failed to collect tasks: {}", e))
}

#[tauri::command]
fn reset_database<R: Runtime>(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get the AppHandle - needed for path resolver and potentially later use
            let app_handle = app.handle();

            // --- Setup Tray Icon ---
            let tray_icon = app.default_window_icon().cloned().ok_or_else(|| {
                // Provide a more descriptive error if the icon is missing
                Box::<dyn std::error::Error>::from("Application icon not found for tray")
            })?;

            let tray = TrayIconBuilder::new()
                .icon(tray_icon) // Use the cloned icon
                .tooltip("Time Sync") // Add a tooltip
                .on_tray_icon_event(|tray_handle, event| {
                    // Use tray_handle.app_handle() inside the event handler
                    let app_event_handle = tray_handle.app_handle();
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            println!("Tray left click");
                            // Show/focus the main window
                            if let Some(window) = app_event_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            } else {
                                eprintln!("Could not find main window to show/focus");
                            }
                        }
                        _ => {
                            // ignore other events for now
                        }
                    }
                })
                .build(app_handle)?; // Build requires AppHandle

            // --- Check for app data folder and create one if it doesn't exist ---
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("Failed to find app data dir");

            match std::fs::create_dir_all(&app_data_dir) {
                Ok(_) => {
                    println!("App data directory ensured: {:?}", app_data_dir);
                }
                Err(e) => {
                    eprintln!(
                        "Failed to create app data directory at {:?}: {}",
                        app_data_dir, e
                    );
                    // Return the error, boxing it
                    return Err(Box::new(e));
                }
            }

            let db_file_path = app_data_dir.join("timesheet.sqlite");

            // Establish db connection
            let manager = SqliteConnectionManager::file(&db_file_path);
            let pool = r2d2::Pool::builder()
                .max_size(10) // Configure pool size (adjust as needed)
                .build(manager)
                .map_err(|e| {
                    Box::<dyn std::error::Error>::from(format!("Failed to create DB pool: {}", e))
                })?; // Handle pool creation error

            println!("Database connection pool created successfully.");
            {
                // Create a scope for the pooled connection
                let conn = pool.get().map_err(|e| {
                    Box::<dyn std::error::Error>::from(format!(
                        "Failed to get connection from pool for init: {}",
                        e
                    ))
                })?;

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS tasks (
                        id STRING PRIMARY KEY,
                        title TEXT NOT NULL,
                        status TEXT NOT NULL,
                        created TEXT
                    )",
                    [],
                )
                .map_err(|e| {
                    Box::<dyn std::error::Error>::from(format!("Failed to create table: {}", e))
                })?;

                println!("Database table 'tasks' ensured.");
            } // Connection automatically returned to pool here

            // --- Store pool in managed state ---
            app_handle.manage(DbState { pool }); // Store the pool itself

            // If all setup steps are successful, return Ok
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_title,
            greet,
            create_task,
            display_tasks,
            reset_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
