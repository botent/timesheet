mod commands;
mod helpers;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Configure the app on setup
        .setup(|app| {
            // Initialize the database and application state
            commands::setup_app(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ui::set_title,
            commands::tasks::create_task,
            commands::tasks::display_tasks,
            commands::tasks::update_task_status,
            commands::tasks::delete_task,
            commands::tasks::get_task_sessions,
            commands::database::reset_database,
        ])
        .on_window_event(|event_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                event_window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
