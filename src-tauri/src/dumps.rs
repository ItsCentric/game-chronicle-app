use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    thread,
};

use csv::Reader;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use tauri::{Emitter, Manager, State};

use crate::{
    helpers::get_app_data_directory,
    igdb::{Cover, Game, Platform, PopularityPrimitive, Website},
    DatabaseConnections, Error,
};

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

fn import_csv<T: serde::de::DeserializeOwned>(
    file_path: PathBuf,
    parse_func: fn(&PathBuf) -> Result<Vec<T>, Error>,
    insert_func: fn(&mut Transaction, &Vec<T>) -> Result<(), Error>,
    transaction: &mut rusqlite::Transaction,
) -> Result<(), Error> {
    if file_path.exists() {
        let items = parse_func(&file_path)?;
        transaction.busy_timeout(std::time::Duration::from_secs(10))?;
        insert_func(transaction, &items)?;
    }
    Ok(())
}

#[tauri::command]
pub fn import_dumps(app_handle: tauri::AppHandle, from_directory: PathBuf) -> Result<(), Error> {
    thread::spawn(move || {
        let app_data_dir = get_app_data_directory(&app_handle).unwrap();
        let mut conn = Connection::open(app_data_dir.join("igdb.db")).unwrap();
        let mut transaction = conn.transaction().unwrap();
        import_csv(
            from_directory.join("covers.csv"),
            parse_csv::<Cover>,
            insert_covers,
            &mut transaction,
        )
        .unwrap();
        import_csv(
            from_directory.join("websites.csv"),
            parse_csv::<Website>,
            insert_websites,
            &mut transaction,
        )
        .unwrap();
        import_csv(
            from_directory.join("platforms.csv"),
            parse_csv::<Platform>,
            insert_platforms,
            &mut transaction,
        )
        .unwrap();
        import_csv(
            from_directory.join("games.csv"),
            parse_csv::<Game>,
            insert_games,
            &mut transaction,
        )
        .unwrap();
        import_csv(
            from_directory.join("popularity_primitives.csv"),
            parse_csv::<PopularityPrimitive>,
            insert_popularity_primitives,
            &mut transaction,
        )
        .unwrap();
        transaction.commit().unwrap();
        app_handle.emit("import_finished", "").unwrap();
    });
    Ok(())
}

fn insert_covers(
    transaction: &mut rusqlite::Transaction,
    covers: &Vec<Cover>,
) -> Result<(), Error> {
    let mut select_stmt = transaction.prepare("SELECT image_id FROM covers WHERE id = ?1")?;
    let mut insert_stmt =
        transaction.prepare("INSERT INTO covers (id, image_id) VALUES (?1, ?2)")?;
    let mut update_stmt = transaction.prepare("UPDATE covers SET image_id = ?1 WHERE id = ?2")?;
    for cover in covers {
        match select_stmt
            .query_row(&[&cover.id], |row| row.get::<_, String>(0))
            .optional()?
        {
            Some(i_id) => {
                if i_id != cover.image_id {
                    update_stmt.execute(params![&cover.image_id, &cover.id])?;
                }
            }
            None => {
                insert_stmt.execute((cover.id, &cover.image_id))?;
            }
        };
    }
    Ok(())
}

fn insert_websites(
    transaction: &mut rusqlite::Transaction,
    websites: &Vec<Website>,
) -> Result<(), Error> {
    let mut select_stmt = transaction.prepare("SELECT url FROM websites WHERE id = ?1")?;
    let mut insert_stmt = transaction.prepare("INSERT INTO websites (id, url) VALUES (?1, ?2)")?;
    let mut update_stmt = transaction.prepare("UPDATE websites SET url = ?1 WHERE id = ?2")?;
    for website in websites {
        let url: Option<String> = select_stmt
            .query_row(&[&website.id], |row| row.get(0))
            .optional()?;
        match url {
            Some(u) => {
                if u != website.url {
                    update_stmt.execute(params![&website.url, &website.id])?;
                }
            }
            None => {
                insert_stmt.execute((website.id, &website.url))?;
            }
        }
    }
    Ok(())
}

fn insert_platforms(
    transaction: &mut rusqlite::Transaction,
    platforms: &Vec<Platform>,
) -> Result<(), Error> {
    let mut select_stmt = transaction.prepare("SELECT * FROM platforms WHERE id = ?1")?;
    let mut insert_stmt =
        transaction.prepare("INSERT INTO platforms (id, name, category) VALUES (?1, ?2, ?3)")?;
    let mut update_stmt =
        transaction.prepare("UPDATE platforms SET name = ?1, category = ?2 WHERE id = ?3")?;
    for csv_platform in platforms {
        let platform: Option<Platform> = select_stmt
            .query_row(&[&csv_platform.id], |row| {
                Ok(Platform {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                })
            })
            .optional()?;
        match platform {
            Some(p) => {
                if p.name != csv_platform.name || p.category != csv_platform.category {
                    update_stmt.execute((
                        &csv_platform.name,
                        csv_platform.category,
                        csv_platform.id,
                    ))?;
                }
            }
            None => {
                insert_stmt.execute((
                    csv_platform.id,
                    &csv_platform.name,
                    csv_platform.category,
                ))?;
            }
        }
    }
    Ok(())
}

fn insert_games(transaction: &mut rusqlite::Transaction, games: &Vec<Game>) -> Result<(), Error> {
    let mut select_game_stmt =
        transaction.prepare("SELECT g.id, g.name, g.cover_id, g.category, g.version_parent, g.total_rating, GROUP_CONCAT(w.id, ','), GROUP_CONCAT(p.id, ','), GROUP_CONCAT(sg.game_id, ',') FROM games g LEFT JOIN covers c ON c.id = g.cover_id LEFT JOIN game_websites gw ON gw.game_id = g.id LEFT JOIN websites w ON w.id = gw.website_id LEFT JOIN similar_games sg ON sg.game_id = g.id LEFT JOIN game_platforms gp ON gp.game_id = g.id LEFT JOIN platforms p ON p.id = gp.platform_id WHERE g.id = ?1 GROUP BY g.id;")?;
    let mut insert_game_stmt = transaction.prepare("INSERT INTO games (id, name, cover_id, category, version_parent, total_rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
    let mut update_game_stmt = transaction.prepare("UPDATE games SET name = ?1, cover_id = ?2, category = ?3, version_parent = ?4, total_rating = ?5 WHERE id = ?6")?;
    let mut insert_game_websites_stmt =
        transaction.prepare("INSERT INTO game_websites (game_id, website_id) VALUES (?1, ?2)")?;
    let mut delete_game_websites_stmt =
        transaction.prepare("DELETE FROM game_websites WHERE game_id = ?1")?;
    let mut insert_similar_games_stmt = transaction
        .prepare("INSERT INTO similar_games (game_id, similar_game_id) VALUES (?1, ?2)")?;
    let mut delete_similar_games_stmt =
        transaction.prepare("DELETE FROM similar_games WHERE game_id = ?1")?;
    let mut insert_game_platforms_stmt =
        transaction.prepare("INSERT INTO game_platforms (game_id, platform_id) VALUES (?1, ?2)")?;
    let mut delete_game_platforms_stmt =
        transaction.prepare("DELETE FROM game_platforms WHERE game_id = ?1")?;
    let mut select_cover_stmt = transaction.prepare("SELECT id FROM covers WHERE id = ?1")?;

    for csv_game in games {
        let result = select_game_stmt
            .query_row(&[&csv_game.id], |row| {
                let website_ids: Option<Vec<i32>> = match row.get::<usize, Option<String>>(6)? {
                    Some(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    None => None,
                };
                let platform_ids: Option<Vec<i32>> = match row.get::<usize, Option<String>>(7)? {
                    Some(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    None => None,
                };
                let similar_games: Option<Vec<i32>> = match row.get::<usize, Option<String>>(8)? {
                    Some(string) => Some(
                        string
                            .split(',')
                            .map(|s| s.parse::<i32>().unwrap())
                            .collect(),
                    ),
                    None => None,
                };
                let cover_id = row.get::<usize, Option<i32>>(2)?;
                let g = Game {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover_id,
                    category: row.get(3)?,
                    version_parent: row.get(4)?,
                    total_rating: row.get(5)?,
                    website_ids: website_ids.clone(),
                    similar_games: similar_games.clone(),
                    platform_ids: platform_ids.clone(),
                };
                Ok(g)
            })
            .optional()?;
        match result {
            Some(db_game) => {
                if db_game.name != csv_game.name
                    || db_game.cover_id != csv_game.cover_id
                    || db_game.category != csv_game.category
                    || db_game.version_parent != csv_game.version_parent
                    || db_game.total_rating != csv_game.total_rating
                {
                    update_game_stmt.execute((
                        &csv_game.name,
                        csv_game.cover_id,
                        csv_game.category,
                        csv_game.version_parent,
                        csv_game.total_rating,
                        csv_game.id,
                    ))?;
                }
                if let Some(website_ids) = &csv_game.website_ids {
                    delete_game_websites_stmt.execute(&[&csv_game.id])?;
                    for website_id in website_ids {
                        insert_game_websites_stmt.execute((csv_game.id, website_id))?;
                    }
                }
                if let Some(similar_games) = &csv_game.similar_games {
                    delete_similar_games_stmt.execute(&[&csv_game.id])?;
                    for similar_game_id in similar_games {
                        insert_similar_games_stmt.execute((csv_game.id, similar_game_id))?;
                    }
                }
                if let Some(platform_ids) = &csv_game.platform_ids {
                    delete_game_platforms_stmt.execute(&[&csv_game.id])?;
                    for platform_id in platform_ids {
                        insert_game_platforms_stmt.execute((csv_game.id, platform_id))?;
                    }
                }
            }
            None => {
                let cover_id = match csv_game.cover_id {
                    Some(id) => match select_cover_stmt
                        .query_row(&[&id], |row| row.get::<usize, i32>(0))
                        .optional()?
                    {
                        Some(_) => Some(id),
                        None => None,
                    },
                    None => None,
                };
                insert_game_stmt.execute((
                    csv_game.id,
                    &csv_game.name,
                    cover_id,
                    csv_game.category,
                    csv_game.version_parent,
                    csv_game.total_rating,
                ))?;
                if let Some(website_ids) = &csv_game.website_ids {
                    for website_id in website_ids {
                        insert_game_websites_stmt.execute((csv_game.id, website_id))?;
                    }
                }
                if let Some(platform_ids) = &csv_game.platform_ids {
                    for platform_id in platform_ids {
                        insert_game_platforms_stmt.execute((csv_game.id, platform_id))?;
                    }
                }
            }
        }
    }
    for csv_game in games {
        if let Some(similar_games) = &csv_game.similar_games {
            for similar_game_id in similar_games {
                match transaction
                    .query_row(
                        "SELECT id FROM games WHERE id = ?",
                        &[similar_game_id],
                        |row| row.get::<usize, i32>(0),
                    )
                    .optional()?
                {
                    Some(_) => {}
                    None => {
                        continue;
                    }
                };
                insert_similar_games_stmt.execute((csv_game.id, similar_game_id))?;
            }
        }
    }

    Ok(())
}

pub fn insert_popularity_primitives(
    transaction: &mut rusqlite::Transaction,
    popularity_primitives: &Vec<PopularityPrimitive>,
) -> Result<(), Error> {
    let mut select_stmt =
        transaction.prepare("SELECT * FROM popularity_primitives WHERE id = ?1")?;
    let mut insert_stmt = transaction.prepare("INSERT INTO popularity_primitives (id, game_id, popularity_type, value) VALUES (?1, ?2, ?3, ?4)")?;
    let mut update_stmt = transaction.prepare("UPDATE popularity_primitives SET game_id = ?1, popularity_type = ?2, value = ?3 WHERE id = ?4")?;
    for csv_popularity_primitive in popularity_primitives {
        let popularity_primitive: Option<PopularityPrimitive> = select_stmt
            .query_row(&[&csv_popularity_primitive.id], |row| {
                Ok(PopularityPrimitive {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    popularity_type: row.get(2)?,
                    value: row.get(3)?,
                })
            })
            .optional()?;
        match popularity_primitive {
            Some(p) => {
                if p.game_id != csv_popularity_primitive.game_id
                    || p.popularity_type != csv_popularity_primitive.popularity_type
                    || p.value != csv_popularity_primitive.value
                {
                    update_stmt.execute((
                        csv_popularity_primitive.game_id,
                        csv_popularity_primitive.popularity_type,
                        csv_popularity_primitive.value,
                        csv_popularity_primitive.id,
                    ))?;
                }
            }
            None => {
                match transaction
                    .query_row(
                        "SELECT id FROM games WHERE id = ?",
                        params![csv_popularity_primitive.game_id],
                        |row| row.get::<usize, i32>(0),
                    )
                    .optional()?
                {
                    Some(_) => {}
                    None => {
                        continue;
                    }
                };
                insert_stmt.execute((
                    csv_popularity_primitive.id,
                    csv_popularity_primitive.game_id,
                    csv_popularity_primitive.popularity_type,
                    csv_popularity_primitive.value,
                ))?;
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
