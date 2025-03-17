use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use csv::Reader;
use tauri::{Emitter, State};

use crate::{
    helpers::get_app_data_directory,
    igdb::{Cover, Game, Platform, PopularityPrimitive, Website},
    DatabasePools, Error,
};

use sqlx::{sqlite::SqliteRow, Row, Sqlite, Transaction};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct CsvUrlResponse {
    pub url: String,
    pub version: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DumpInfo<'a> {
    pub name: &'a str,
    pub url: String,
    pub version: String,
}

type DumpVersions = HashMap<String, String>;

async fn get_csv_url(endpoint: &str) -> Result<CsvUrlResponse, Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("https://api.gamechronicle.app/csv/{}", endpoint))
        .send()
        .await?
        .json::<CsvUrlResponse>()
        .await?;
    Ok(response)
}

async fn get_csv_data(url: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

fn parse_csv<T>(csv_path: &std::path::PathBuf) -> Result<Vec<T>, Error>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    let mut rdr = Reader::from_path(csv_path)?;
    let mut items = vec![];
    for result in rdr.deserialize() {
        let item: T = result?;
        items.push(item);
    }
    Ok(items)
}

#[tauri::command]
pub fn get_local_dump_versions(app_handle: tauri::AppHandle) -> Result<DumpVersions, Error> {
    let app_data_dir = get_app_data_directory(&app_handle)?;
    let mut file = match fs::File::open(app_data_dir.join("dump_versions.toml")) {
        Ok(file) => file,
        Err(_) => {
            let mut hm = DumpVersions::new();
            hm.insert("covers".to_string(), "".to_string());
            hm.insert("websites".to_string(), "".to_string());
            hm.insert("platforms".to_string(), "".to_string());
            hm.insert("games".to_string(), "".to_string());
            hm.insert("popularity_primitives".to_string(), "".to_string());
            return Ok(hm);
        }
    };
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let dump_versions: HashMap<String, toml::Value> = toml::from_str(&file_contents)?;
    let dump_versions = dump_versions
        .into_iter()
        .map(|(k, v)| (k, v.as_str().unwrap_or_default().to_string()))
        .collect();

    Ok(dump_versions)
}

#[tauri::command]
pub async fn get_all_dump_info() -> Result<Vec<DumpInfo<'static>>, Error> {
    let endpoints = [
        "covers",
        "websites",
        "platforms",
        "games",
        "popularity_primitives",
    ];
    let mut dumps_info: Vec<DumpInfo> = vec![];
    for endpoint in endpoints {
        let csv_response = get_csv_url(endpoint).await?;
        dumps_info.push(DumpInfo {
            name: endpoint,
            url: csv_response.url,
            version: csv_response.version,
        });
    }
    Ok(dumps_info)
}

#[tauri::command]
pub async fn download_dumps(
    dump_info: Vec<DumpInfo<'_>>,
    to_directory: PathBuf,
) -> Result<(), Error> {
    for info in dump_info {
        let csv_data = get_csv_data(&info.url).await?;
        let mut csv_file = File::create(to_directory.join(format!("{}.csv", info.name)))?;
        std::io::copy(&mut csv_data.as_bytes(), &mut csv_file)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn import_dumps(
    state: State<'_, DatabasePools>,
    app_handle: tauri::AppHandle,
    from_directory: PathBuf,
) -> Result<(), Error> {
    let igdb_pool = state.igdb_pool.clone();
    tauri::async_runtime::spawn(async move {
        let mut transaction = igdb_pool.begin().await.unwrap();
        let covers_dir = from_directory.join("covers.csv");
        if covers_dir.exists() {
            let covers = parse_csv::<Cover>(&covers_dir).unwrap();
            insert_covers(&mut transaction, &covers).await.unwrap();
        }
        let websites_dir = from_directory.join("websites.csv");
        if websites_dir.exists() {
            let websites = parse_csv::<Website>(&websites_dir).unwrap();
            insert_websites(&mut transaction, &websites).await.unwrap();
        }
        let platforms_dir = from_directory.join("platforms.csv");
        if platforms_dir.exists() {
            let platforms = parse_csv::<Platform>(&platforms_dir).unwrap();
            insert_platforms(&mut transaction, &platforms)
                .await
                .unwrap();
        }
        let games_dir = from_directory.join("games.csv");
        if games_dir.exists() {
            let games = parse_csv::<Game>(&games_dir).unwrap();
            insert_games(&mut transaction, &games).await.unwrap();
        }
        let popularity_primitives_dir = from_directory.join("popularity_primitives.csv");
        if popularity_primitives_dir.exists() {
            let popularity_primitives =
                parse_csv::<PopularityPrimitive>(&popularity_primitives_dir).unwrap();
            insert_popularity_primitives(&mut transaction, &popularity_primitives).await.unwrap();
        }
        transaction.commit().await.unwrap();
        app_handle.emit("import_finished", "").unwrap();
    });
    Ok(())
}

async fn insert_covers(
    transaction: &mut Transaction<'_, Sqlite>,
    covers: &Vec<Cover>,
) -> Result<(), Error> {
    for cover in covers {
        match sqlx::query_scalar::<Sqlite, String>(
            "SELECT image_id FROM covers WHERE id = ?1",
        )
        .bind(cover.id)
        .fetch_optional(&mut **transaction)
        .await?
        {
            Some(i_id) => {
                if i_id != cover.image_id {
                    sqlx::query("UPDATE covers SET image_id = ?1 WHERE id = ?2")
                        .bind(cover.image_id.clone())
                        .bind(cover.id)
                        .execute(&mut **transaction)
                        .await?;
                }
            }
            None => {
                sqlx::query("INSERT INTO covers (id, image_id) VALUES (?1, ?2)")
                    .bind(cover.id)
                    .bind(cover.image_id.clone())
                    .execute(&mut **transaction)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn insert_websites(
    transaction: &mut Transaction<'_, Sqlite>,
    websites: &Vec<Website>,
) -> Result<(), Error> {
    for website in websites {
        let url =
            sqlx::query_scalar::<Sqlite, String>("SELECT url FROM websites WHERE id = ?1")
                .bind(website.id)
                .fetch_optional(&mut **transaction)
                .await?;
        match url {
            Some(u) => {
                if u != website.url {
                    sqlx::query("UPDATE websites SET url = ?1 WHERE id = ?2")
                        .bind(website.url.clone())
                        .bind(website.id)
                        .execute(&mut **transaction)
                        .await?;
                }
            }
            None => {
                sqlx::query("INSERT INTO websites (id, url) VALUES (?1, ?2)")
                    .bind(website.id)
                    .bind(website.url.clone())
                    .execute(&mut **transaction)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn insert_platforms(
    transaction: &mut Transaction<'_, Sqlite>,
    platforms: &Vec<Platform>,
) -> Result<(), Error> {
    for csv_platform in platforms {
        let platform: Option<Platform> = sqlx::query_as::<Sqlite, Platform>(
            "SELECT id, name, category FROM platforms WHERE id = ?1",
        )
        .bind(csv_platform.id)
        .fetch_optional(&mut **transaction)
        .await?;
        match platform {
            Some(p) => {
                if p.name != csv_platform.name || p.category != csv_platform.category {
                    sqlx::query("UPDATE platforms SET name = ?1, category = ?2 WHERE id = ?3")
                        .bind(&csv_platform.name)
                        .bind(csv_platform.category)
                        .bind(csv_platform.id)
                        .execute(&mut **transaction)
                        .await?;
                }
            }
            None => {
                sqlx::query("INSERT INTO platforms (id, name, category) VALUES (?1, ?2, ?3)")
                    .bind(csv_platform.id)
                    .bind(&csv_platform.name)
                    .bind(csv_platform.category)
                    .execute(&mut **transaction)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn insert_games(
    transaction: &mut Transaction<'_, Sqlite>,
    games: &Vec<Game>,
) -> Result<(), Error> {
    for csv_game in games {
        let result = sqlx::query("SELECT g.id, g.name, c.image_id, g.category, g.version_parent, g.total_rating, GROUP_CONCAT(w.id, ','), GROUP_CONCAT(p.id, ','), GROUP_CONCAT(sg.game_id, ',') FROM games g LEFT JOIN covers c ON c.id = g.cover_id LEFT JOIN game_websites gw ON gw.game_id = g.id LEFT JOIN websites w ON w.id = gw.website_id LEFT JOIN similar_games sg ON sg.game_id = g.id LEFT JOIN game_platforms gp ON gp.game_id = g.id LEFT JOIN platforms p ON p.id = gp.platform_id WHERE g.id = ?1 GROUP BY g.id;")
            .bind(csv_game.id)
            .map(|row: SqliteRow| {
                let website_ids: Option<Vec<i32>> = match row.try_get::<String, usize>(6) {
                    Ok(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    Err(_) => None,
                };

                let platform_ids: Option<Vec<i32>> = match row.try_get::<String, usize>(7) {
                    Ok(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    Err(_) => None,
                };

                let similar_games: Option<Vec<i32>> = match row.try_get::<String, usize>(8) {
                    Ok(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    Err(_) => None,
                };

                let cover_id = match row.try_get::<Option<i32>, usize>(2) {
                    Ok(id) => id,
                    Err(_) => None,
                };

                Game {
                    id: row.get::<i32, usize>(0),
                    name: row.get::<String, usize>(1),
                    cover_id,
                    category: row.get::<i32, usize>(3),
                    version_parent: row.get::<Option<i32>, usize>(4),
                    total_rating: row.get::<Option<f32>, usize>(5),
                    website_ids,
                    similar_games,
                    platform_ids,
                }
            })
            .fetch_optional(&mut **transaction).await?;
        let select_cover =
            sqlx::query_scalar::<Sqlite, i32>("SELECT id FROM covers WHERE id = ?1");
        match result {
            Some(db_game) => {
                if db_game.name != csv_game.name
                    || db_game.cover_id != csv_game.cover_id
                    || db_game.category != csv_game.category
                    || db_game.version_parent != csv_game.version_parent
                    || db_game.total_rating != csv_game.total_rating
                {
                    let cover_id = match csv_game.cover_id {
                        Some(id) => select_cover
                            .bind(id)
                            .fetch_optional(&mut **transaction)
                            .await?.map(|_| id),
                        None => None,
                    };
                    sqlx::query("UPDATE games SET name = ?1, cover_id = ?2, category = ?3, version_parent = ?4, total_rating = ?5 WHERE id = ?6")
                        .bind(&csv_game.name)
                        .bind(cover_id)
                        .bind(csv_game.category)
                        .bind(csv_game.version_parent)
                        .bind(csv_game.total_rating)
                        .bind(csv_game.id)
                        .execute(&mut **transaction).await?;
                }
                if let Some(website_ids) = &csv_game.website_ids {
                    sqlx::query("DELETE FROM game_websites WHERE game_id = ?1")
                        .bind(csv_game.id)
                        .execute(&mut **transaction)
                        .await?;
                    for website_id in website_ids {
                        sqlx::query(
                            "INSERT INTO game_websites (game_id, website_id) VALUES (?1, ?2)",
                        )
                        .bind(csv_game.id)
                        .bind(website_id)
                        .execute(&mut **transaction)
                        .await?;
                    }
                }
                if let Some(similar_games) = &csv_game.similar_games {
                    sqlx::query("DELETE FROM similar_games WHERE game_id = ?1")
                        .bind(csv_game.id)
                        .execute(&mut **transaction)
                        .await?;
                    for similar_game_id in similar_games {
                        sqlx::query(
                            "INSERT INTO similar_games (game_id, similar_game_id) VALUES (?1, ?2)",
                        )
                        .bind(csv_game.id)
                        .bind(similar_game_id)
                        .execute(&mut **transaction)
                        .await?;
                    }
                }
                if let Some(platform_ids) = &csv_game.platform_ids {
                    sqlx::query("DELETE FROM game_platforms WHERE game_id = ?1")
                        .bind(csv_game.id)
                        .execute(&mut **transaction)
                        .await?;
                    for platform_id in platform_ids {
                        sqlx::query(
                            "INSERT INTO game_platforms (game_id, platform_id) VALUES (?1, ?2)",
                        )
                        .bind(csv_game.id)
                        .bind(platform_id)
                        .execute(&mut **transaction)
                        .await?;
                    }
                }
            }
            None => {
                let cover_id = match csv_game.cover_id {
                    Some(id) => select_cover
                        .bind(id)
                        .fetch_optional(&mut **transaction)
                        .await?.map(|_| id),
                    None => None,
                };
                sqlx::query("INSERT INTO games (id, name, cover_id, category, version_parent, total_rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
                    .bind(csv_game.id)
                    .bind(&csv_game.name)
                    .bind(cover_id)
                    .bind(csv_game.category)
                    .bind(csv_game.version_parent)
                    .bind(csv_game.total_rating)
                    .execute(&mut **transaction).await?;
                if let Some(website_ids) = &csv_game.website_ids {
                    for website_id in website_ids {
                        sqlx::query(
                            "INSERT INTO game_websites (game_id, website_id) VALUES (?1, ?2)",
                        )
                        .bind(csv_game.id)
                        .bind(website_id)
                        .execute(&mut **transaction)
                        .await?;
                    }
                }
                if let Some(platform_ids) = &csv_game.platform_ids {
                    for platform_id in platform_ids {
                        sqlx::query(
                            "INSERT INTO game_platforms (game_id, platform_id) VALUES (?1, ?2)",
                        )
                        .bind(csv_game.id)
                        .bind(platform_id)
                        .execute(&mut **transaction)
                        .await?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn insert_popularity_primitives(
    transaction: &mut Transaction<'_, Sqlite>,
    popularity_primitives: &Vec<PopularityPrimitive>,
) -> Result<(), Error> {
    for csv_popularity_primitive in popularity_primitives {
        let popularity_primitive = sqlx::query_as::<Sqlite, PopularityPrimitive>(
            "SELECT * FROM popularity_primitives WHERE id = ?1",
        )
        .bind(csv_popularity_primitive.id)
        .fetch_optional(&mut **transaction)
        .await?;
        match popularity_primitive {
            Some(p) => {
                if p.game_id != csv_popularity_primitive.game_id
                    || p.popularity_type != csv_popularity_primitive.popularity_type
                    || p.value != csv_popularity_primitive.value
                {
                    sqlx::query("UPDATE popularity_primitives SET game_id = ?1, popularity_type = ?2, value = ?3 WHERE id = ?4")
                        .bind(csv_popularity_primitive.game_id)
                        .bind(csv_popularity_primitive.popularity_type)
                        .bind(csv_popularity_primitive.value)
                        .bind(csv_popularity_primitive.id)
                        .execute(&mut **transaction)
                        .await?;
                }
            }
            None => {
                match sqlx::query_scalar::<Sqlite, i32>("SELECT id FROM games WHERE id = ?1")
                    .bind(csv_popularity_primitive.game_id)
                    .fetch_optional(&mut **transaction)
                    .await?
                {
                    Some(_) => {}
                    None => {
                        continue;
                    }
                };
                sqlx::query("INSERT INTO popularity_primitives (id, game_id, popularity_type, value) VALUES (?1, ?2, ?3, ?4)")
                    .bind(csv_popularity_primitive.id)
                    .bind(csv_popularity_primitive.game_id)
                    .bind(csv_popularity_primitive.popularity_type)
                    .bind(csv_popularity_primitive.value)
                    .execute(&mut **transaction).await?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn save_local_dump_versions(
    dump_versions: DumpVersions,
    app_handle: tauri::AppHandle,
) -> Result<(), Error> {
    let app_data_dir = get_app_data_directory(&app_handle)?;
    let dump_versions_str = toml::to_string(&dump_versions)?;
    fs::write(app_data_dir.join("dump_versions.toml"), dump_versions_str)?;

    Ok(())
}
