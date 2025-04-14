pub mod sessions;
pub mod tasks;

use r2d2_sqlite::SqliteConnectionManager;
pub use sessions::TaskSession;
pub use tasks::UserTask;

pub type SqlitePool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct DbState {
    pub pool: SqlitePool,
}
