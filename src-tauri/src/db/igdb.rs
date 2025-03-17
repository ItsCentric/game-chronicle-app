use std::{path::Path, str::FromStr};

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteConnectOptions, Sqlite, SqlitePool, sqlite::SqliteRow, Row};

use crate::{DatabasePools, Error};

use super::IgdbDb;

use tauri::State;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Game {
    pub id: i32,
    pub name: String,
    #[serde(rename(deserialize = "cover"))]
    pub cover_id: Option<i32>,
    #[serde(
        rename(deserialize = "websites"),
        deserialize_with = "deserialize_list"
    )]
    pub website_ids: Option<Vec<i32>>,
    #[serde(deserialize_with = "deserialize_list")]
    pub similar_games: Option<Vec<i32>>,
    pub category: i32,
    pub version_parent: Option<i32>,
    pub total_rating: Option<f32>,
    #[serde(
        rename(deserialize = "platforms"),
        deserialize_with = "deserialize_list"
    )]
    pub platform_ids: Option<Vec<i32>>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cover {
    pub id: i32,
    pub image_id: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Website {
    pub id: i32,
    pub url: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, sqlx::FromRow)]
pub struct Platform {
    pub id: i32,
    pub name: String,
    pub category: Option<i32>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone, sqlx::FromRow)]
pub struct GameInfo {
    pub id: i32,
    #[sqlx(rename = "name")]
    pub title: String,
    #[sqlx(rename = "image_id")]
    pub cover_image_id: Option<String>,
    #[sqlx(json)]
    pub websites: Option<Vec<String>>,
    #[sqlx(json)]
    pub similar_games: Option<Vec<i32>>,
    pub category: i32,
    pub version_parent: Option<i32>,
    pub total_rating: Option<f32>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, sqlx::FromRow)]
pub struct PopularityPrimitive {
    pub id: i32,
    pub game_id: i32,
    pub popularity_type: i32,
    pub value: f32,
}


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

fn deserialize_list<'de, D>(deserializer: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    let s = s.trim_start_matches('{').trim_end_matches('}');
    if s.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        s.split(',')
            .map(|item| match item.trim().parse::<i32>() {
                Ok(id) => id,
                Err(_) => 0,
            })
            .collect(),
    ))
}

fn game_info_columns() -> &'static str {
    "g.id, g.name, c.image_id, GROUP_CONCAT(w.url, ',') websites, GROUP_CONCAT(sg.similar_game_id, ',') similar_game_ids, g.category, g.version_parent, total_rating FROM games g LEFT JOIN covers c ON g.cover_id = c.id LEFT JOIN game_websites gw ON g.id = gw.game_id LEFT JOIN websites w ON gw.website_id = w.id LEFT JOIN similar_games sg ON sg.game_id = g.id LEFT JOIN game_platforms gp ON g.id = gp.game_id LEFT JOIN platforms p ON p.id = gp.platform_id LEFT JOIN popularity_primitives pp ON g.id = pp.game_id"
}

#[tauri::command]
pub async fn get_games_by_id(
    state: State<'_, DatabasePools>,
    game_ids: Vec<i32>,
) -> Result<Vec<GameInfo>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let query = format!(
        "SELECT {} WHERE g.id IN ({}) AND g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id;",
        game_info_columns(), game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let games: Vec<GameInfo> = sqlx::query(&query).map(|row: SqliteRow| {
    GameInfo {
        id: row.get("id"),
        title: row.get("name"),
        cover_image_id: row.get("image_id"),
        websites: row.get::<Option<String>, _>("websites").map(|s| s.split(',').map(String::from).collect()),
        similar_games: row.get::<Option<String>, _>("similar_game_ids").map(|s| s.split(',').filter_map(|x| x.parse::<i32>().ok()).collect()),
        category: row.get("category"),
        version_parent: row.get("version_parent"),
        total_rating: row.get("total_rating"),
    }
    }).fetch_all(&state.igdb_pool).await?;
    Ok(games)
}

#[tauri::command]
pub async fn get_popular_games(
    state: State<'_, DatabasePools>,
    amount: i32,
) -> Result<Vec<GameInfo>, Error> {
    let games: Vec<GameInfo> = sqlx::query(&format!("SELECT {} WHERE g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id ORDER BY pp.value DESC LIMIT $1;", game_info_columns()).to_string()).bind(amount).map(|row: SqliteRow| {
    GameInfo {
        id: row.get("id"),
        title: row.get("name"),
        cover_image_id: row.get("image_id"),
        websites: row.get::<Option<String>, _>("websites").map(|s| s.split(',').map(String::from).collect()),
        similar_games: row.get::<Option<String>, _>("similar_game_ids").map(|s| s.split(',').filter_map(|x| x.parse::<i32>().ok()).collect()),
        category: row.get("category"),
        version_parent: row.get("version_parent"),
        total_rating: row.get("total_rating"),
    }
    }).fetch_all(&state.igdb_pool).await?;
    Ok(games)
}

#[tauri::command]
pub async fn search_game(
    state: State<'_, DatabasePools>,
    search_query: String,
) -> Result<Vec<GameInfo>, Error> {
    let results: Vec<i32> = sqlx::query_scalar("SELECT rowid FROM games_fts WHERE name MATCH $1;")
        .bind(search_query.replace("'", " "))
        .fetch_all(&state.igdb_pool)
        .await?;
    let games = get_games_by_id(state, results).await?;
    Ok(games)
}

pub async fn get_games_from_links(
    state: State<'_, DatabasePools>,
    links: Vec<String>,
) -> Result<Vec<GameInfo>, Error> {
    let formatted_links = links
        .iter()
        .map(|l| format!("'{}'", l))
        .collect::<Vec<String>>();
    let games: Vec<GameInfo> = sqlx::query(format!("SELECT {} WHERE w.url IN ({}) AND g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id;", game_info_columns(), formatted_links.join(",").as_str()).as_str()).map(|row: SqliteRow| {
    GameInfo {
        id: row.get("id"),
        title: row.get("name"),
        cover_image_id: row.get("image_id"),
        websites: row.get::<Option<String>, _>("websites").map(|s| s.split(',').map(String::from).collect()),
        similar_games: row.get::<Option<String>, _>("similar_game_ids").map(|s| s.split(',').filter_map(|x| x.parse::<i32>().ok()).collect()),
        category: row.get("category"),
        version_parent: row.get("version_parent"),
        total_rating: row.get("total_rating"),
    }
    }).fetch_all(&state.igdb_pool).await?;
    Ok(games)
}
