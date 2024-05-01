use reqwest::Client;

use crate::Error;

use std::env;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct IgdbGame {
    pub id: i32,
    pub name: String,
    pub cover: Cover,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cover {
    pub id: i32,
    pub image_id: String,
}

pub async fn send_igdb_request(
    endpoint: &String,
    access_token: &String,
    body: String,
) -> Result<reqwest::Response, Error> {
    let client = Client::new();
    let response = client
        .post(&format!("https://api.igdb.com/v4/{}", endpoint))
        .header("Client-ID", env::var("TWITCH_CLIENT_ID")?)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    Ok(response)
}

#[tauri::command]
pub async fn authenticate_with_twitch() -> Result<AccessTokenResponse, Error> {
    let client = Client::new();
    let response = client
        .post(&format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", env::var("TWITCH_CLIENT_ID")?, env::var("TWITCH_CLIENT_SECRET")?))
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
) -> Result<Vec<IgdbGame>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let body = format!(
        "fields name, cover.image_id; where id = ({})",
        game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let response = send_igdb_request(&"games".to_string(), &access_token, body).await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}

#[tauri::command]
pub async fn get_similar_games(
    access_token: String,
    game_ids: Vec<i32>,
) -> Result<Vec<IgdbGame>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let body = format!(
        "fields name, cover.image_id; where id = ({})",
        game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let response = send_igdb_request(&"games".to_string(), &access_token, body).await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}
