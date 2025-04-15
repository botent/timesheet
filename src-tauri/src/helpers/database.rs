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

// // Function to get TaskSessions by task_id
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

// Function to sum the total duration of TaskSessions for a task_id
pub fn sum_task_session_duration(conn: &Connection, task_id: &str) -> Result<i64, String> {
    let mut stmt = conn
        .prepare("SELECT SUM(duration) FROM task_sessions WHERE task_id = ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let mut rows = stmt
        .query(params![task_id])
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to fetch row: {}", e))?
    {
        row.get(0)
            .map_err(|e| format!("Failed to get duration sum: {}", e))
    } else {
        Ok(0) // No sessions found, return 0
    }
}
