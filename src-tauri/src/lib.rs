mod commands;
mod helpers;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            commands::setup_app(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ui::set_title,
            commands::tasks::create_task,
            commands::tasks::display_tasks,
            commands::database::reset_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
