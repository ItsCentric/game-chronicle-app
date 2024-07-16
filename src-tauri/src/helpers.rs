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
