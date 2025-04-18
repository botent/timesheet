use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::models::TaskSession;

// Function to create the task_sessions table (if it doesn't exist)
pub fn create_task_sessions_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_sessions (
            id TEXT PRIMARY KEY,
            duration INTEGER NOT NULL,
            task_id TEXT NOT NULL,
            started TEXT NOT NULL,
            ended TEXT NULL,
            FOREIGN KEY (task_id) REFERENCES tasks (id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_sessions table: {}", e))?;

    Ok(())
}

// Function to create a new TaskSession
pub fn create_task_session(conn: &Connection, session: TaskSession) -> Result<(), String> {
    let session_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO task_sessions (id, duration, task_id, started, ended) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session_id,
            session.duration,
            session.task_id,
            session.started,
            session.ended,
        ],
    )
    .map_err(|e| format!("Failed to insert task session: {}", e))?;

    Ok(())
}

// Function to get TaskSessions by task_id
#[allow(dead_code)]
pub fn get_task_sessions(conn: &Connection, task_id: &str) -> Result<Vec<TaskSession>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, duration, task_id, started, ended FROM task_sessions WHERE task_id = ?1",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let session_iter = stmt
        .query_map(params![task_id], |row| {
            Ok(TaskSession {
                id: row.get(0)?,
                duration: row.get(1)?,
                task_id: row.get(2)?,
                started: row.get(3)?,
                ended: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    session_iter
        .collect::<Result<Vec<TaskSession>, rusqlite::Error>>()
        .map_err(|e| format!("Failed to collect task sessions: {}", e))
}

// Function to close any open/active sessions that might have been left open when the app closed
pub fn close_orphaned_sessions(conn: &Connection) -> Result<(), String> {
    // Get the current time
    let now = chrono::Utc::now().to_rfc3339();
    
    // First find all sessions that are still open (no end time)
    let mut stmt = conn.prepare(
        "SELECT id, task_id, started FROM task_sessions WHERE ended IS NULL"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let open_sessions = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,  // id
            row.get::<_, String>(1)?,  // task_id
            row.get::<_, String>(2)?,  // started
        ))
    }).map_err(|e| format!("Failed to execute query: {}", e))?;
    
    for session_result in open_sessions {
        if let Ok((session_id, task_id, started_str)) = session_result {
            // Update task status to Paused if it was Running
            conn.execute(
                "UPDATE tasks SET status = 'Paused' WHERE id = ?1 AND status = 'Running'",
                params![task_id],
            ).map_err(|e| format!("Failed to update task status: {}", e))?;
            
            // Calculate duration for this orphaned session
            if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&started_str) {
                let started_utc = started.with_timezone(&chrono::Utc);
                let app_exit_time = chrono::Utc::now(); // Assume the app just exited before this function was called
                
                // Calculate the duration using timestamps
                let session_duration = app_exit_time.signed_duration_since(started_utc).num_seconds();
                let final_duration = if session_duration > 0 { session_duration } else { 0 };
                
                // Close the session with the calculated duration
                conn.execute(
                    "UPDATE task_sessions SET ended = ?1, duration = ?2 WHERE id = ?3",
                    params![now, final_duration, session_id],
                ).map_err(|e| format!("Failed to close orphaned session: {}", e))?;
            } else {
                // If we can't parse the start time, just close the session with 0 duration
                conn.execute(
                    "UPDATE task_sessions SET ended = ?1, duration = 0 WHERE id = ?2",
                    params![now, session_id],
                ).map_err(|e| format!("Failed to close orphaned session: {}", e))?;
            }
        }
    }
    
    println!("Closed any orphaned sessions from previous app run");
    Ok(())
}

// Function to sum the total duration of TaskSessions for a task_id
pub fn sum_task_session_duration(conn: &Connection, task_id: &str) -> Result<i64, String> {
    // Get all completed sessions with start and end times
    let mut stmt = conn.prepare(
        "SELECT started, ended FROM task_sessions 
         WHERE task_id = ?1 AND ended IS NOT NULL"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let rows = stmt.query_map(params![task_id], |row| {
        Ok((
            row.get::<_, String>(0)?,  // started
            row.get::<_, String>(1)?,   // ended
        ))
    }).map_err(|e| format!("Failed to execute query: {}", e))?;
    
    let mut total_duration: i64 = 0;
    
    // Calculate durations directly from timestamps for all completed sessions
    for row_result in rows {
        match row_result {
            Ok((started_str, ended_str)) => {
                if let (Ok(started), Ok(ended)) = (
                    chrono::DateTime::parse_from_rfc3339(&started_str),
                    chrono::DateTime::parse_from_rfc3339(&ended_str)
                ) {
                    let session_duration = ended.signed_duration_since(started).num_seconds();
                    if session_duration > 0 {
                        total_duration += session_duration;
                    }
                }
            },
            Err(_) => continue,
        }
    }
    
    // Check if the task is currently running
    let task_status: String = conn
        .query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0)
        )
        .unwrap_or_else(|_| String::from(""));
    
    // Only calculate active duration if the task is actually running
    if task_status == "Running" {
        // Get the most recent active session
        let active_session_start: Option<String> = conn
            .query_row(
                "SELECT started FROM task_sessions 
                 WHERE task_id = ?1 AND ended IS NULL 
                 ORDER BY started DESC LIMIT 1",
                params![task_id],
                |row| row.get(0)
            )
            .ok();
        
        if let Some(started_str) = active_session_start {
            if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&started_str) {
                let now = chrono::Utc::now();
                let active_duration = now.signed_duration_since(started).num_seconds();
                if active_duration > 0 {
                    total_duration += active_duration;
                }
            }
        }
    }
    
    Ok(total_duration)
}
