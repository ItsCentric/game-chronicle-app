use std::{path::Path, str::FromStr};

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteConnectOptions, Sqlite, SqlitePool};

use crate::Error;

use super::IgdbDb;

pub async fn init_igdb_db(dir: &Path) -> Result<IgdbDb, Error> {
    let db_path = Path::new("sqlite:").join(dir).join("igdb.db?mode=rwc");
    let db_url = match db_path.to_str() {
        Some(url) => url,
        None => return Err(Error::from("Could not convert database path to string"))
    };
    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(db_url)?.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal),
    ).await?;

    sqlx::migrate!("migrations/igdb").run(&pool).await?;

    Ok(pool)
}
