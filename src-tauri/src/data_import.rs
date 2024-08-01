use std::{
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
};

use crate::{database::LogData, igdb::get_games_from_links, DatabaseConnections};
use chrono::{DateTime, Local};
use reqwest::Client;
use rusqlite::params;
use tauri::{Emitter, Manager, State};

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
    steam_key: String,
) -> Result<Vec<LogData>, Error> {
    let http_client = Client::new();
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
            let _ = *retrieval_finished_lock;
        }
    });
    let mut logs_data: Vec<LogData> = Vec::new();
    let steam_links = owned_steam_games_response
        .games
        .iter()
        .map(|s_game| format!("https://store.steampowered.com/app/{}", s_game.appid))
        .collect::<Vec<String>>();
    let games = get_games_from_links(app_handle.state::<DatabaseConnections>(), steam_links)?;
    for steam_game in owned_steam_games_response.games {
        let igdb_game = match games.iter().find(|g| {
            g.websites.iter().any(|w| {
                w.contains(&format!(
                    "https://store.steampowered.com/app/{}",
                    steam_game.appid
                ))
            })
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
        let formatted_date = date.format("%Y-%m-%d %H:%M:%S").to_string();
        logs_data.push(LogData {
            game_id: igdb_game.id,
            start_date: formatted_date.clone(),
            end_date: formatted_date,
            rating: 0,
            notes: "".to_string(),
            status,
            minutes_played: steam_game.playtime_forever,
        });
    }
    let mut games_retrieved_lock = games_retrieved.write().unwrap();
    *games_retrieved_lock += 1;
    {
        let (retrieval_finished_lock, cvar) = &*retrieval_finished;
        let mut retrieval_finished_lock = retrieval_finished_lock.lock().unwrap();
        *retrieval_finished_lock = true;
        cvar.notify_one();
    }
    handle.join().unwrap();
    Ok(logs_data)
}

#[tauri::command]
pub fn import_igdb_games(
    app_handle: tauri::AppHandle,
    state: State<DatabaseConnections>,
    data: Vec<LogData>,
) -> Result<usize, Error> {
    let mut conn = state.logs_conn.lock().unwrap();
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
            let _ = *import_finished_lock;
        }
    });
    let logs_transaction = conn.transaction()?;
    {
        let mut stmt = logs_transaction.prepare(
            "INSERT INTO logs (start_date, end_date, status, minutes_played, notes, game_id) VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        for log_data in &data {
            stmt.execute(params![
                &log_data.start_date,
                &log_data.end_date,
                &log_data.status,
                &log_data.minutes_played,
                &log_data.notes,
                &log_data.game_id,
            ])?;
            let mut games_imported_lock = games_imported.write().unwrap();
            *games_imported_lock += 1;
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
