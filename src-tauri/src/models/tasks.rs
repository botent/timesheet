use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserTask {
    pub id: Option<String>,
    pub title: String,
    pub status: String,
    pub created: String,
}
