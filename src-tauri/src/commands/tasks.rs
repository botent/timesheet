use crate::commands::tray::TrayState;
use crate::helpers::database::{create_task_session, get_task_sessions, sum_task_session_duration};
use crate::models::{DbState, TaskSession, UserTask};
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Error};
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Runtime, State, Window};
use uuid::Uuid;

#[tauri::command]
pub fn create_task(task_data: UserTask, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state
        .pool
        .get()
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    let task_id = Uuid::new_v4().to_string();

    match conn.execute(
        "INSERT INTO tasks (id, title, status, created) VALUES (?1, ?2, ?3, ?4)",
        params![
            task_id.clone(), // Clone task_id for session creation
            task_data.title,
            task_data.status.clone(), // Clone status for session creation
            task_data.created
        ],
    ) {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                println!("Task inserted successfully.");

                // Create initial TaskSession when a task is created
                let new_session = TaskSession {
                    id: None,
                    duration: 0,
                    task_id: task_id.clone(),
                    started: Utc::now().to_rfc3339(),
                    ended: None,
                };
                create_task_session(&conn, new_session).map_err(|e| e.to_string())?;

                Ok(())
            } else {
                eprintln!("Task insert affected 0 rows.");
                Err("Insert failed: 0 rows affected".to_string())
            }
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

#[tauri::command]
pub fn update_task_status<R: Runtime>(
    window: Window,
    task_id: String,
    status: String,
    state: State<'_, DbState>,
    app_handle: AppHandle<R>,
) -> Result<(), String> {
    let conn = state
        .pool
        .get()
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    // Get reference to tray state
    let tray_state = window.state::<TrayState>();

    // Get the current task status from the database to handle status transitions properly
    let current_status: String = conn
        .query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            params![task_id.clone()],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| String::from(""));

    // Use more specific pattern matching to handle all cases explicitly based on the current status
    match (status.as_str(), current_status.as_str()) {
        // Running status transitions
        ("Running", "Running") => {
            // Already running - just ensure it's in the running tasks list
            tray_state.add_running_task(task_id.clone())?;
        }
        ("Running", _) => {
            // Starting a new session or resuming a paused/completed task
            tray_state.add_running_task(task_id.clone())?;

            // Create a new TaskSession
            let new_session = TaskSession {
                id: None,
                duration: 0, // Duration will be calculated when paused/completed
                task_id: task_id.clone(),
                started: Utc::now().to_rfc3339(),
                ended: None,
            };
            create_task_session(&conn, new_session).map_err(|e| e.to_string())?;
        }

        // Paused or Completed status transitions
        ("Paused" | "Completed", "Running") => {
            // Going from Running to Paused/Completed - need to stop the session and calculate duration
            tray_state.remove_running_task(&task_id)?;

            // End the current TaskSession and update duration
            let sessions = get_task_sessions(&conn, &task_id).map_err(|e| e.to_string())?;
            if let Some(current_session) = sessions.into_iter().filter(|s| s.ended.is_none()).next()
            {
                let started: DateTime<Utc> = DateTime::parse_from_rfc3339(&current_session.started)
                    .map_err(|e| e.to_string())?
                    .with_timezone(&Utc);
                let ended: DateTime<Utc> = Utc::now();

                // Calculate precise duration in seconds for this session only
                let session_duration = ended.signed_duration_since(started).num_seconds();

                // Make sure we're storing accurate duration (avoid negative values)
                let final_duration = if session_duration > 0 {
                    session_duration
                } else {
                    0
                };

                conn.execute(
                    "UPDATE task_sessions SET ended = ?1, duration = ?2 WHERE id = ?3",
                    params![
                        ended.to_rfc3339(),
                        final_duration,
                        current_session.id.unwrap()
                    ],
                )
                .map_err(|e| format!("Failed to update session: {}", e))?;
            }
        }
        ("Paused" | "Completed", _) => {
            // Already paused/completed or changing between Paused and Completed
            // Just ensure it's not in the running tasks list
            tray_state.remove_running_task(&task_id)?;
        }

        // Other status transitions (fallback)
        (_, _) => {
            // No action needed for other status combinations
        }
    }

    let running_count = tray_state.get_running_task_count()?;

    if let Some(tray) = app_handle.tray_by_id("main") {
        if let Err(e) = tray.set_title(Some(running_count)) {
            eprintln!("Failed to set tray icon: {}", e);
        }
    } else {
        eprintln!("Tray with ID 'tray' not found");
    }

    // Now update the database with the new status
    conn.execute(
        "UPDATE tasks SET status = ?1 WHERE id = ?2",
        params![status.clone(), task_id.clone()],
    )
    .map_err(|e| format!("Failed to update task status: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn delete_task(task_id: String, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state
        .pool
        .get()
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    // First delete the task sessions associated with this task
    conn.execute(
        "DELETE FROM task_sessions WHERE task_id = ?1",
        params![task_id],
    )
    .map_err(|e| format!("Failed to delete task sessions: {}", e))?;

    // Then delete the task itself
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
        .map_err(|e| format!("Failed to delete task: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn display_tasks(
    state: State<'_, DbState>,
) -> Result<HashMap<String, Vec<(UserTask, i64)>>, String> {
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

    let tasks: Result<Vec<UserTask>, Error> = task_iter.collect();
    let tasks = tasks.map_err(|e| format!("Failed to collect tasks: {}", e))?;

    // Group tasks by date and calculate total duration
    let mut grouped_tasks: HashMap<String, Vec<(UserTask, i64)>> = HashMap::new();
    for task in tasks {
        let date = match NaiveDateTime::parse_from_str(&task.created, "%Y-%m-%dT%H:%M:%S%.fZ") {
            Ok(datetime) => datetime.format("%Y-%m-%d").to_string(),
            Err(_) => {
                eprintln!("Failed to parse date: {}", task.created);
                continue;
            }
        };

        // Calculate total duration for the task
        let task_id_str = task.id.as_deref().unwrap_or("");
        let total_duration = sum_task_session_duration(&conn, task_id_str).unwrap_or(0);

        grouped_tasks
            .entry(date)
            .or_default()
            .push((task, total_duration));
    }

    // Sort dates in descending order
    let mut sorted_dates: Vec<String> = grouped_tasks.keys().cloned().collect();
    sorted_dates.sort_by(|a, b| b.cmp(a));

    // Create a new HashMap with sorted dates
    let mut sorted_grouped_tasks: HashMap<String, Vec<(UserTask, i64)>> = HashMap::new();
    for date in sorted_dates {
        if let Some(tasks) = grouped_tasks.get(&date) {
            sorted_grouped_tasks.insert(date.clone(), tasks.clone());
        }
    }

    Ok(sorted_grouped_tasks)
}
