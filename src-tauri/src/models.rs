use chrono::{DateTime, Utc};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

// Pool type alias for cleaner code
pub type SqlitePool = r2d2::Pool<SqliteConnectionManager>;

// State holds the connection pool
#[derive(Clone)]
pub struct DbState {
    pub pool: SqlitePool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTask {
    pub id: Option<String>,
    pub title: String,
    pub status: TaskStatus,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "Running")]
    Running,
    #[serde(rename = "Completed")]
    Completed,
    #[serde(rename = "Paused")]
    Paused,
    #[serde(rename = "Not yet started")]
    NotStarted,
}

impl ToSql for TaskStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        // Convert the enum variant to its string representation
        let status_str = match self {
            TaskStatus::Running => "Running",
            TaskStatus::Completed => "Completed",
            TaskStatus::Paused => "Paused",
            TaskStatus::NotStarted => "Not yet started",
        };
        // Ok wrapping the conversion to ToSqlOutput
        Ok(ToSqlOutput::from(status_str))
    }
}

impl FromSql for TaskStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        // Get the string value from the database column
        value.as_str().and_then(|s| {
            // Attempt to match the string back to an enum variant
            match s {
                "Running" => Ok(TaskStatus::Running),
                "Completed" => Ok(TaskStatus::Completed),
                "Paused" => Ok(TaskStatus::Paused),
                "Not yet started" => Ok(TaskStatus::NotStarted),
                // If the string doesn't match any known status, return an error
                _ => Err(FromSqlError::Other(
                    format!("Invalid TaskStatus string: {}", s).into(),
                )),
            }
        })
    }
}
