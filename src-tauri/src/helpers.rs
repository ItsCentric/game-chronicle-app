use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use tauri::Manager;

use crate::{Error, ProcessMonitoringSettings, UserSettings};

#[derive(serde::Deserialize, Debug)]
pub struct CsvUrlResponse {
    pub url: String,
    pub version: String,
}

pub type DumpVersions = HashMap<String, toml::Value>;

#[tauri::command]
pub fn get_user_settings(app_handle: tauri::AppHandle) -> Result<UserSettings, Error> {
    let config_path = app_handle.path().config_dir().unwrap();
    let mut file = fs::File::open(config_path.join("game-chronicle/settings.toml"))?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let mut settings_map: HashMap<String, toml::Value> = toml::from_str(&file_contents)?;
    let user_settings = UserSettings {
        username: settings_map
            .remove("username")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(whoami::username),
        executable_paths: settings_map.remove("executable_paths").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        }),
        process_monitoring: {
            let process_monitoring_map = settings_map
                .remove("process_monitoring")
                .and_then(|v| v.as_table().cloned())
                .unwrap_or_default();
            ProcessMonitoringSettings {
                enabled: process_monitoring_map
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                directory_depth: process_monitoring_map
                    .get("directory_depth")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(2) as usize,
            }
        },
        autostart: settings_map
            .remove("autostart")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        new: settings_map
            .remove("new")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    };

    Ok(user_settings)
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

pub fn get_app_data_directory(app_handle: &tauri::AppHandle) -> Result<PathBuf, Error> {
    let dir = app_handle.path().data_dir()?;
    Ok(dir.join("game-chronicle"))
}

pub fn get_csv_url_blocking(endpoint: &str) -> Result<CsvUrlResponse, Error> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("https://api.gamechronicle.app/csv/{}", endpoint))
        .send()?
        .json::<CsvUrlResponse>()?;
    Ok(response)
}

pub fn get_csv_data_blocking(url: &str) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;
    Ok(response)
}

pub fn get_dump_versions(app_handle: &tauri::AppHandle) -> Result<DumpVersions, Error> {
    let app_data_dir = get_app_data_directory(app_handle)?;
    let mut file = fs::File::open(app_data_dir.join("dump_versions.toml"))?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let dump_versions: HashMap<String, toml::Value> = toml::from_str(&file_contents)?;

    Ok(dump_versions)
}

pub fn save_dump_versions(
    dump_versions: DumpVersions,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let app_data_dir = get_app_data_directory(app_handle)?;
    let dump_versions_str = toml::to_string(&dump_versions)?;
    fs::write(app_data_dir.join("dump_versions.toml"), dump_versions_str)?;

    Ok(())
}
