use chrono::NaiveDateTime;
use rusqlite::{params, Error};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::models::{DbState, UserTask};

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
            task_id,
            task_data.title,
            task_data.status,
            task_data.created
        ],
    ) {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                println!("Task inserted successfully.");
                Ok(())
            } else {
                eprintln!("Task insert affected 0 rows.");
                Err("Insert failed: 0 rows affected".to_string())
            }
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

// #[tauri::command]
// pub fn display_tasks(state: State<'_, DbState>) -> Result<Vec<UserTask>, String> {
//     let conn = state
//         .pool
//         .get()
//         .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

//     let mut stmt = conn
//         .prepare("SELECT id, title, status, created FROM tasks")
//         .map_err(|e| format!("Failed to prepare statement: {}", e))?;

//     let task_iter = stmt
//         .query_map([], |row| {
//             Ok(UserTask {
//                 id: row.get(0)?,
//                 title: row.get(1)?,
//                 status: row.get(2)?,
//                 created: row.get(3)?,
//             })
//         })
//         .map_err(|e| format!("Failed to execute query: {}", e))?;

//     task_iter
//         .collect::<Result<Vec<UserTask>, rusqlite::Error>>()
//         .map_err(|e| format!("Failed to collect tasks: {}", e))
// }

#[tauri::command]
pub fn display_tasks(state: State<'_, DbState>) -> Result<HashMap<String, Vec<UserTask>>, String> {
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

    // Group tasks by date
    let mut grouped_tasks: HashMap<String, Vec<UserTask>> = HashMap::new();
    for task in tasks {
        let date = match NaiveDateTime::parse_from_str(&task.created, "%Y-%m-%dT%H:%M:%S%.fZ") {
            Ok(datetime) => datetime.format("%Y-%m-%d").to_string(), // Extract date
            Err(_) => {
                eprintln!("Failed to parse date: {}", task.created);
                continue; // Skip tasks with invalid dates
            }
        };

        grouped_tasks.entry(date).or_default().push(task);
    }

    // Sort dates in descending order
    let mut sorted_dates: Vec<String> = grouped_tasks.keys().cloned().collect();
    sorted_dates.sort_by(|a, b| b.cmp(a)); // Descending order

    // Create a new HashMap with sorted dates
    let mut sorted_grouped_tasks: HashMap<String, Vec<UserTask>> = HashMap::new();
    for date in sorted_dates {
        if let Some(tasks) = grouped_tasks.get(&date) {
            sorted_grouped_tasks.insert(date.clone(), tasks.clone());
        }
    }

    Ok(sorted_grouped_tasks)
}
