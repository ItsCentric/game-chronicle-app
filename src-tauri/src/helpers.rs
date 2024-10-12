use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use tauri::Manager;

use crate::{Error, ProcessMonitoringSettings, UserSettings};

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct CsvUrlResponse {
    pub url: String,
    pub version: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct SchemaFieldUpdate {
    pub new_name: String,
    pub new_type: String,
    pub update_type: i32,
    pub default: Option<String>,
    pub constraint_data: Option<String>,
}

type DatabaseUpdateTable = HashMap<String, HashMap<String, SchemaFieldUpdate>>;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct SchemaUpdate {
    pub igdb: Option<DatabaseUpdateTable>,
    pub logs: Option<DatabaseUpdateTable>,
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
        executable_paths: settings_map
            .remove("executable_paths")
            .and_then(|v| v.as_str().map(String::from)),
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

pub fn get_schema_changes(app_handle: &tauri::AppHandle) -> Result<SchemaUpdate, Error> {
    let resource_path = app_handle.path().resource_dir().unwrap();
    let mut file = match fs::File::open(resource_path.join("resources/schema_changes.toml")) {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                return Ok(SchemaUpdate {
                    igdb: None,
                    logs: None,
                })
            }
            _ => return Err(e.into()),
        },
    };
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let schema_changes: SchemaUpdate = toml::from_str(&file_contents)?;
    Ok(schema_changes)
}
