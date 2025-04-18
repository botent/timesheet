use std::sync::{Arc, Mutex};
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};

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

    pub fn get_running_task_count(&self) -> Result<usize, String> {
        let tasks = self
            .running_tasks
            .lock()
            .map_err(|_| "Failed to lock running_tasks".to_string())?;
        Ok(tasks.len())
    }
}

// Update the tray icon with task count information
pub fn update_tray_icon<R: Runtime>(app_handle: &AppHandle<R>) -> Result<(), String> {
    // Get the current task count
    let tray_state = app_handle.state::<TrayState>();
    let running_count = tray_state.get_running_task_count()?;

    // Create an appropriate tray title based on running tasks
    let tray_title = match running_count {
        0 => "No tasks running".to_string(),
        1 => "1 task running".to_string(),
        count => format!("{} tasks running", count),
    };

    if let Some(tray) = app_handle.tray_by_id("tray_ts_t") {
        if let Err(e) = tray.set_title(Some(&tray_title)) {
            eprintln!("Failed to set tray title: {}", e);
        }
    } else {
        eprintln!("Tray with ID 'tray' not found");
    }

    println!("Tray would display: {}", tray_title);

    Ok(())
}

pub fn set_tray_icon<R: Runtime>(app_handle: &AppHandle<R>) -> Result<(), String> {
    let quit_i =
        MenuItem::with_id(app_handle, "quit", "Quit Time Sync", true, None::<&str>).unwrap();

    let menu = MenuBuilder::new(app_handle).item(&quit_i).build().unwrap();

    let _tray = TrayIconBuilder::with_id("tray_ts_t")
        .icon(Image::from_path("././icons/icon.png").expect("msg"))
        .title("title")
        .menu(&menu)
        .on_menu_event(|app_handle, event| match event.id.as_ref() {
            "quit" => app_handle.exit(0),
            _ => println!("Unhandled menu item: {:?}", event.id),
        })
        .build(app_handle)
        .unwrap();
    Ok(())
}
