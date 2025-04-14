use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskSession {
    pub id: Option<String>,
    pub duration: i64,
    pub task_id: String,
    pub started: String,
    pub ended: Option<String>,
}
