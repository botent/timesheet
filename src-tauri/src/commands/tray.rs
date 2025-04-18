use std::sync::{Arc, Mutex};
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Runtime};

type TaskId = String;

// Store running task count in state
pub struct TrayState {
    pub running_tasks: Arc<Mutex<Vec<String>>>, // Store IDs of running tasks
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            running_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_running_task(&self, task_id: TaskId) -> Result<(), String> {
        let mut tasks = self
            .running_tasks
            .lock()
            .map_err(|_| "Failed to lock running_tasks".to_string())?;

        // Check if we already reached the maximum number of running tasks
        if tasks.len() >= 3 {
            return Err("Maximum number of concurrent running tasks (3) reached".to_string());
        }

        // Avoid adding duplicates
        if !tasks.contains(&task_id) {
            tasks.push(task_id);
        }

        Ok(())
    }

    pub fn remove_running_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self
            .running_tasks
            .lock()
            .map_err(|_| "Failed to lock running_tasks".to_string())?;

        tasks.retain(|id| id != task_id);

        Ok(())
    }

    pub fn get_running_task_count(&self) -> Result<String, String> {
        let tasks = self
            .running_tasks
            .lock()
            .map_err(|_| "Failed to lock running_tasks".to_string())?;
        Ok(tasks.len().to_string())
    }
}

pub fn set_tray_icon<R: Runtime>(app_handle: &AppHandle<R>) -> Result<(), String> {
    let quit_i =
        MenuItem::with_id(app_handle, "quit", "Quit Time Sync", true, None::<&str>).unwrap();

    let menu = MenuBuilder::new(app_handle).item(&quit_i).build().unwrap();

    let icon = Image::from_path("../src-tauri/icons/icon.png")
        .map_err(|e| format!("Failed to load icon from path: {}", e))?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .title("title")
        .menu(&menu)
        .on_menu_event(|app_handle, event| match event.id.as_ref() {
            "quit" => app_handle.exit(0),
            _ => println!("Unhandled menu item: {:?}", event.id),
        })
        .build(app_handle)
        .expect("Failed to build tray icon");

    Ok(())
}
