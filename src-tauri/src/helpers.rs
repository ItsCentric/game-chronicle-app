use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use tauri::Manager;

use crate::{Error, UserSettings};

#[derive(serde::Deserialize, Debug)]
pub struct CsvUrlResponse {
    url: String,
}

#[tauri::command]
pub fn get_user_settings(app_handle: tauri::AppHandle) -> Result<UserSettings, Error> {
    let config_path = app_handle.path().config_dir().unwrap();
    let mut file = fs::File::open(config_path.join("game-chronicle/settings.toml"))?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(toml::from_str::<UserSettings>(&file_contents)?)
}

#[tauri::command]
pub fn save_user_settings(
    user_settings: UserSettings,
    app_handle: tauri::AppHandle,
) -> Result<UserSettings, Error> {
    let config_path = app_handle.path().config_dir().unwrap();
    let settings_str = toml::to_string(&user_settings)?;
    fs::write(
        config_path.join("game-chronicle/settings.toml"),
        settings_str,
    )?;
    Ok(user_settings)
}

pub fn create_dir_if_not_exists(path: &Path) -> Result<(), std::io::Error> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            e => Err(e.into()),
        },
    }
}

pub fn get_app_config_directory(app_handle: &tauri::AppHandle) -> Result<PathBuf, Error> {
    let dir = app_handle.path().config_dir()?;
    Ok(dir.join("game-chronicle"))
}

pub fn get_app_data_directory(app_handle: &tauri::AppHandle) -> Result<PathBuf, Error> {
    let dir = app_handle.path().data_dir()?;
    Ok(dir.join("game-chronicle"))
}

pub fn get_csv_url_blocking(endpoint: &str) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!(
            "https://game-chronicle-api-e9e3d1c83b7e.herokuapp.com/csv/{}",
            endpoint
        ))
        .send()?
        .json::<CsvUrlResponse>()?;
    Ok(response.url)
}

pub fn get_csv_data_blocking(url: &str) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;
    Ok(response)
}
