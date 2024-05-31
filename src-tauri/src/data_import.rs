use std::{
    env,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
};

use crate::{
    database::{Game, LogData, SafeConnection},
    igdb::{authenticate_with_twitch, multi_search_game_links, IgdbGame},
};
use chrono::{DateTime, Local};
use reqwest::Client;
use rusqlite::{params, OptionalExtension};
use tauri::{Manager, State};

use crate::Error;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct SteamGame {
    appid: i32,
    name: String,
    playtime_forever: i32,
    playtime_2weeks: Option<i32>,
    rtime_last_played: i32,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct OwnedGames {
    game_count: i32,
    games: Vec<SteamGame>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct SteamResponse<T> {
    response: T,
}

type LogAndIgdbData = Vec<(LogData, IgdbGame)>;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
struct RetrievePayload {
    status: String,
    total_games: Option<i32>,
    games_retrieved: Option<i32>,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
struct ImportProgressPayload {
    games_imported: i32,
}

#[tauri::command]
pub async fn get_steam_data(
    app_handle: tauri::AppHandle,
    steam_id: String,
) -> Result<LogAndIgdbData, Error> {
    let http_client = Client::new();
    let steam_key = env::var("STEAM_KEY")?;
    let owned_steam_games_http_response = http_client
        .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?")
        .query(&[
            ("key", steam_key),
            ("steamid", steam_id),
            ("include_appinfo", "true".to_string()),
            ("include_played_free_games", "true".to_string()),
        ])
        .send()
        .await?;
    let owned_steam_games_response = owned_steam_games_http_response
        .json::<SteamResponse<OwnedGames>>()
        .await?
        .response;
    app_handle.emit(
        "retrieval",
        RetrievePayload {
            status: "started".to_string(),
            total_games: Some(owned_steam_games_response.game_count),
            games_retrieved: None,
        },
    )?;
    let retrieval_finished = Arc::new((Mutex::new(false), Condvar::new()));
    let games_retrieved = Arc::new(RwLock::new(0));
    let retrieval_finished_clone = Arc::clone(&retrieval_finished);
    let games_retrieved_clone = Arc::clone(&games_retrieved);
    let app_handle_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        let (retrieval_finished_lock, cvar) = &*retrieval_finished_clone;
        loop {
            let mut retrieval_finished_lock = retrieval_finished_lock.lock().unwrap();
            if *retrieval_finished_lock {
                break;
            }
            {
                let mut games_retrieved_lock = games_retrieved_clone.write().unwrap();
                app_handle_clone
                    .emit(
                        "retrieval",
                        RetrievePayload {
                            status: "progress".to_string(),
                            total_games: None,
                            games_retrieved: Some(*games_retrieved_lock),
                        },
                    )
                    .unwrap();
                *games_retrieved_lock = 0;
            }
            retrieval_finished_lock = cvar
                .wait_timeout(retrieval_finished_lock, std::time::Duration::from_secs(2))
                .unwrap()
                .0;
        }
    });
    let access_token_response = authenticate_with_twitch(app_handle.clone()).await?;
    let mut logs_and_games_data: LogAndIgdbData = Vec::new();
    for steam_game_chunk in owned_steam_games_response.games.chunks(8) {
        let steam_links = steam_game_chunk
            .iter()
            .map(|s_game| format!("https://store.steampowered.com/app/{}", s_game.appid))
            .collect::<Vec<String>>();
        let igdb_responses = multi_search_game_links(
            access_token_response.access_token.clone(),
            steam_links,
            app_handle.clone(),
        )
        .await?;
        for igdb_response in igdb_responses {
            if igdb_response.result.is_empty() {
                continue;
            }
            let igdb_game = &igdb_response.result[0];
            let steam_game =
                match steam_game_chunk
                    .iter()
                    .find(|&s_game| match igdb_game.websites {
                        Some(ref websites) => websites
                            .iter()
                            .any(|website| website.url.contains(&s_game.appid.to_string())),
                        None => false,
                    }) {
                    Some(game) => game,
                    None => continue,
                };
            let status = match steam_game.playtime_forever {
                0 => "backlog".to_string(),
                _ => "played".to_string(),
            };
            let date: DateTime<Local> =
                match DateTime::from_timestamp(steam_game.rtime_last_played as i64, 0) {
                    Some(date) => match date {
                        date if date == DateTime::UNIX_EPOCH => Local::now(),
                        _ => date.into(),
                    },
                    None => Local::now(),
                };
            let game = match &igdb_game.cover {
                Some(cover) => Game {
                    id: igdb_game.id,
                    cover_id: cover.id.to_string(),
                    title: igdb_game.title.clone(),
                },
                None => Game {
                    id: igdb_game.id,
                    cover_id: "".to_string(),
                    title: igdb_game.title.clone(),
                },
            };
            logs_and_games_data.push((
                LogData {
                    date: date.format("%Y-%m-%d %H:%M:%S").to_string(),
                    rating: 0,
                    notes: "".to_string(),
                    status,
                    minutes_played: steam_game.playtime_forever,
                    game,
                },
                igdb_game.clone(),
            ));
            let mut games_retrieved_lock = games_retrieved.write().unwrap();
            *games_retrieved_lock += 1;
        }
    }
    {
        let (retrieval_finished_lock, cvar) = &*retrieval_finished;
        let mut retrieval_finished_lock = retrieval_finished_lock.lock().unwrap();
        *retrieval_finished_lock = true;
        cvar.notify_one();
    }
    handle.join().unwrap();
    Ok(logs_and_games_data)
}

#[tauri::command]
pub fn import_igdb_games(
    app_handle: tauri::AppHandle,
    state: State<'_, SafeConnection>,
    data: Vec<(crate::database::LogData, IgdbGame)>,
) -> Result<usize, Error> {
    let mut conn = state.lock().unwrap();
    let app_handle_clone = app_handle.clone();
    let import_finished = Arc::new((Mutex::new(false), Condvar::new()));
    let games_imported = Arc::new(RwLock::new(0));
    let import_finished_clone = Arc::clone(&import_finished);
    let games_imported_clone = Arc::clone(&games_imported);
    let handle = thread::spawn(move || {
        let (import_finished_lock, cvar) = &*import_finished_clone;
        loop {
            let mut import_finished_lock = import_finished_lock.lock().unwrap();
            if *import_finished_lock {
                break;
            }
            {
                let mut games_imported_lock = games_imported_clone.write().unwrap();
                app_handle_clone
                    .emit(
                        "import",
                        ImportProgressPayload {
                            games_imported: *games_imported_lock,
                        },
                    )
                    .unwrap();
                *games_imported_lock = 0;
            }
            import_finished_lock = cvar
                .wait_timeout(import_finished_lock, std::time::Duration::from_secs(2))
                .unwrap()
                .0;
        }
    });
    let logged_games_transaction = conn.transaction()?;
    {
        let mut no_cover_stmt = logged_games_transaction
            .prepare("INSERT INTO logged_games (id, title) VALUES (?, ?)")?;
        let mut cover_stmt = logged_games_transaction
            .prepare("INSERT INTO logged_games (id, title, cover_id) VALUES (?, ?, ?)")?;
        for (_, game) in &data {
            let game_id = logged_games_transaction
                .query_row(
                    "SELECT id FROM logged_games where id = ?",
                    [game.id],
                    |row| {
                        let id: i32 = row.get(0)?;
                        Ok(id)
                    },
                )
                .optional()?;
            match game_id {
                Some(_) => continue,
                None => match &game.cover {
                    Some(cover) => {
                        cover_stmt.execute(params![game.id, game.title, cover.cover_id])?;
                    }
                    None => {
                        no_cover_stmt.execute(params![game.id, game.title])?;
                    }
                },
            }
            let mut games_imported_lock = games_imported.write().unwrap();
            *games_imported_lock += 1;
        }
    }
    logged_games_transaction.commit()?;
    let logs_transaction = conn.transaction()?;
    {
        let mut stmt = logs_transaction.prepare(
            "INSERT INTO logs (date, status, minutes_played, notes, game_id) VALUES (?, ?, ?, ?, ?)",
        )?;
        for (log_data, game) in &data {
            stmt.execute(params![
                &log_data.date,
                &log_data.status,
                &log_data.minutes_played,
                &log_data.notes,
                &game.id
            ])?;
        }
    }
    logs_transaction.commit()?;
    {
        let (import_finished_lock, cvar) = &*import_finished;
        let mut import_finished_lock = import_finished_lock.lock().unwrap();
        *import_finished_lock = true;
        cvar.notify_one();
    }
    handle.join().unwrap();
    Ok(data.len())
}
