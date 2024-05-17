use std::io::Read;

use reqwest::Client;
use tauri::Manager;

use crate::{Error, UserSettings};

use rand::Rng;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct IgdbGame {
    pub id: i32,
    #[serde(rename(deserialize = "name"))]
    pub title: String,
    pub cover: Option<Cover>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cover {
    pub id: i32,
    #[serde(rename(deserialize = "image_id"))]
    pub cover_id: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct SimilarGames {
    pub similar_games: Vec<IgdbGame>,
}

pub async fn send_igdb_request(
    endpoint: &String,
    access_token: &String,
    body: String,
    settings_path: std::path::PathBuf,
) -> Result<reqwest::Response, Error> {
    let client = Client::new();
    let mut contents = String::new();
    let mut file = std::fs::File::open(settings_path)?;
    file.read_to_string(&mut contents).unwrap();
    let settings = toml::from_str::<UserSettings>(&contents)?;
    let twitch_client_id;
    match settings.twitch_client_id {
        Some(id) => twitch_client_id = id,
        None => return Err(Error::from("Twitch client ID not found")),
    }
    let response = client
        .post(&format!("https://api.igdb.com/v4/{}", endpoint))
        .header("Client-ID", twitch_client_id)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    Ok(response)
}

#[tauri::command]
pub async fn authenticate_with_twitch(
    app_handle: tauri::AppHandle,
) -> Result<AccessTokenResponse, Error> {
    let client = Client::new();
    let settings_path = app_handle
        .path()
        .config_dir()?
        .join("game-chronicle/settings.toml");
    let mut contents = String::new();
    let mut file = std::fs::File::open(settings_path)?;
    file.read_to_string(&mut contents).unwrap();
    let settings = toml::from_str::<UserSettings>(&contents)?;
    let twitch_client_id;
    let twitch_client_secret;
    match settings.twitch_client_id {
        Some(id) => twitch_client_id = id,
        None => return Err(Error::from("Twitch client ID not found")),
    }
    match settings.twitch_client_secret {
        Some(secret) => twitch_client_secret = secret,
        None => return Err(Error::from("Twitch client secret not found")),
    }
    let response = client
        .post(&format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", twitch_client_id, twitch_client_secret))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let json_response: AccessTokenResponse = serde_json::from_str(response.text().await?.as_str())?;
    Ok(json_response)
}

#[tauri::command]
pub async fn get_games_by_id(
    access_token: String,
    game_ids: Vec<i32>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<IgdbGame>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let body = format!(
        "fields name, cover.image_id; where id = ({});",
        game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let response = send_igdb_request(
        &"games".to_string(),
        &access_token,
        body,
        app_handle
            .path()
            .config_dir()?
            .join("game-chronicle/settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}

#[tauri::command]
pub async fn get_similar_games(
    access_token: String,
    game_ids: Vec<i32>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<SimilarGames>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let body = format!(
        "fields similar_games.name, similar_games.cover.image_id; where id = ({}) & category = 0 & platforms.category = (1, 6); exclude id;",
        game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let response = send_igdb_request(
        &"games".to_string(),
        &access_token,
        body,
        app_handle
            .path()
            .config_dir()?
            .join("game-chronicle/settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}

#[tauri::command]
pub async fn get_random_top_games(
    access_token: String,
    amount: i32,
    app_handle: tauri::AppHandle,
) -> Result<Vec<IgdbGame>, Error> {
    let random_offset = rand::thread_rng().gen_range(0..900);
    let body = format!(
        "fields name, cover.image_id; limit {}; where category = 0 & total_rating >= 85 & platforms.category = (1, 6); offset {};",
        amount,
        random_offset
    );
    let response = send_igdb_request(
        &"games".to_string(),
        &access_token,
        body,
        app_handle
            .path()
            .config_dir()?
            .join("game-chronicle/settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}

#[tauri::command]
pub async fn search_game(
    access_token: String,
    search_query: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<IgdbGame>, Error> {
    let body = format!(
        "fields name, cover.image_id; search \"{}\"; where category = 0 & version_parent = null;",
        search_query
    );
    let response = send_igdb_request(
        &"games".to_string(),
        &access_token,
        body,
        app_handle
            .path()
            .config_dir()?
            .join("game-chronicle/settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}
