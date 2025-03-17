pub mod igdb;
pub mod logs;

pub type LogsDb = sqlx::SqlitePool;
pub type IgdbDb = sqlx::SqlitePool;
