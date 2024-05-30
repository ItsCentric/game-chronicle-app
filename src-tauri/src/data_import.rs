use std::env;

use crate::{
    database::{Game, LogData, SafeConnection},
    igdb::{authenticate_with_twitch, multi_search_game_links, IgdbGame},
};
use chrono::{DateTime, Local};
use reqwest::Client;
use rusqlite::{params, OptionalExtension};
use tauri::State;

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

#[tauri::command]
pub async fn get_steam_data(
    app_handle: tauri::AppHandle,
    steam_id: String,
) -> Result<LogAndIgdbData, Error> {
    let http_client = Client::new();
    let steam_key = env::var("STEAM_KEY")?;
    let owned_steam_games_response = http_client
        .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?")
        .query(&[
            ("key", steam_key),
            ("steamid", steam_id),
            ("include_appinfo", "true".to_string()),
            ("include_played_free_games", "true".to_string()),
        ])
        .send()
        .await?;
    let owned_steam_games = owned_steam_games_response
        .json::<SteamResponse<OwnedGames>>()
        .await?
        .response
        .games;
    let access_token_response = authenticate_with_twitch(app_handle.clone()).await?;
    let mut logs_and_games_data: LogAndIgdbData = Vec::new();
    for steam_game_chunk in owned_steam_games.chunks(8) {
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
        }
    }
    Ok(logs_and_games_data)
}

#[tauri::command]
pub fn import_igdb_games(
    state: State<'_, SafeConnection>,
    data: Vec<(crate::database::LogData, IgdbGame)>,
) -> Result<(), Error> {
    let mut conn = state.lock().unwrap();
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
        }
    }
    logged_games_transaction.commit()?;
    let logs_transaction = conn.transaction()?;
    {
        let mut stmt = logs_transaction.prepare(
            "INSERT INTO logs (date, status, minutes_played, notes, game_id) VALUES (?, ?, ?, ?, ?)",
        )?;
        for (log_data, game) in data {
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
    Ok(())
}
