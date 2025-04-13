use tauri::window::Window;

#[tauri::command]
pub fn set_title(window: Window, title: String) {
    if let Err(e) = window.set_title(&title) {
        eprintln!("error updating title: {}", e);
    }
}
