use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::window::Window;
use tauri::Manager;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            tray.on_tray_icon_event(|tray, event| match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    println!("left click pressed and released");
                    // in this example, let's show and focus the main window when the tray is clicked
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {
                    // println!("unhandled event {event:?}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_title, greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
