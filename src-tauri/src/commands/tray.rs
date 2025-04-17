use std::sync::{Arc, Mutex};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent, TrayIconId};
use tauri::{AppHandle, Runtime};

// Store running task count in state
pub struct TrayState {
    pub running_tasks: Arc<Mutex<Vec<String>>>, // Store IDs of running tasks
}

type TaskId = String;

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

use tauri::Manager;

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

    if let Some(tray) = app_handle.tray_by_id("tray_ts") {
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
    let tray = TrayIconBuilder::with_id("tray_ts")
        .icon(app_handle.default_window_icon().unwrap().clone())
        .build(app_handle);
    Ok(())
}
