use std::{path::Path, str::FromStr};

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteConnectOptions, Sqlite, SqlitePool};

use super::LogsDb;

pub async fn init_logs_db(dir: &Path) -> Result<LogsDb, sqlx::Error> {
    let path = Path::new("sqlite:").join(dir).join("logs.db?mode=rwc");
    let db_url = match  path.to_str() {
        Some(url) => url,
        None => return Err(sqlx::Error::Configuration("Invalid database URL".into())),
    };
    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(db_url)?.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
    ).await?;
    sqlx::migrate!("migrations/logs").run(&pool).await?;

    Ok(pool)
}
