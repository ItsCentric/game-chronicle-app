use std::io::Read;

use csv::Reader;
use reqwest::Client;

use crate::{helpers::get_app_config_directory, Error, UserSettings};

use rand::Rng;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone)]
pub struct IgdbGame {
    pub id: i32,
    #[serde(rename(deserialize = "name"))]
    pub title: String,
    pub cover: Option<Cover>,
    pub websites: Option<Vec<Website>>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone)]
pub struct Cover {
    pub id: i32,
    #[serde(rename(deserialize = "image_id"))]
    pub cover_id: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone)]
pub struct Website {
    pub url: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct SimilarGames {
    pub similar_games: Option<Vec<IgdbGame>>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct MultiQueryResponse<T> {
    pub result: Vec<T>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct NewIgdbGame {
    pub id: i32,
    pub name: String,
    #[serde(rename(deserialize = "cover"))]
    pub cover_id: Option<i32>,
    #[serde(
        rename(deserialize = "websites"),
        deserialize_with = "deserialize_list"
    )]
    pub website_ids: Option<Vec<i32>>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct NewCover {
    pub id: i32,
    pub image_id: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct NewWebsite {
    pub id: i32,
    pub url: String,
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

pub fn parse_csv<T>(csv_path: &std::path::PathBuf) -> Result<Vec<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut rdr = Reader::from_path(csv_path)?;
    let mut items = vec![];
    for result in rdr.deserialize() {
        let item: T = result?;
        items.push(item);
    }
    Ok(items)
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
    let config_dir = get_app_config_directory(&app_handle)?;
    let settings_path = config_dir.join("settings.toml");
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
        get_app_config_directory(&app_handle)?.join("settings.toml"),
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
        get_app_config_directory(&app_handle)?.join("settings.toml"),
    )
    .await;
    let deserialized_response: Vec<SimilarGames> =
        serde_json::from_str(response?.text().await?.as_str())?;
    Ok(deserialized_response
        .into_iter()
        .filter(|s| s.similar_games.is_some())
        .collect())
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
        get_app_config_directory(&app_handle)?.join("settings.toml"),
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
        get_app_config_directory(&app_handle)?.join("settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}

pub async fn multi_search_game_links(
    access_token: String,
    links: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<MultiQueryResponse<IgdbGame>>, Error> {
    let mut body = String::new();
    for (i, link) in links.iter().enumerate() {
        body.push_str(&format!(
            "query games \"Part {}\" {{fields name, cover.image_id, websites.url; where category = 0 & version_parent = null & websites.url ~ *\"{}\"*;}};",
            i, link
        ));
    }
    let response = send_igdb_request(
        &"multiquery".to_string(),
        &access_token,
        body,
        get_app_config_directory(&app_handle)?.join("settings.toml"),
    )
    .await;
    serde_json::from_str(response?.text().await?.as_str()).map_err(Error::from)
}
