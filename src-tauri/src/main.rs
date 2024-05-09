// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, io::Read, path::PathBuf, thread};

use serde::Deserialize;

use std::path::Path;

use tauri::Manager;

mod database;
mod igdb;
mod process_monitor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Process not found")]
    ProcessNotFound,
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDer(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error("Error: {0}")]
    Custom(String),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Custom(err.to_string())
    }
}

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct UserSettings {
    username: String,
    executable_paths: String,
    process_monitoring: ProcessMonitoringSettings,
    twitch_client_id: Option<String>,
    twitch_client_secret: Option<String>,
}

#[derive(serde::Serialize, Debug, Deserialize)]
struct ProcessMonitoringSettings {
    enabled: bool,
    directory_depth: i32,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let _conn = database::initialize_database().unwrap();
            let config_path = app.path().config_dir().unwrap();
            let user_settings: UserSettings;
            match fs::File::open(config_path.join("settings.toml")) {
                Ok(mut file) => {
                    let mut file_contents = String::new();
                    file.read_to_string(&mut file_contents).unwrap();
                    user_settings = toml::from_str::<UserSettings>(&file_contents).unwrap();
                }
                Err(_) => {
                    let settings = UserSettings {
                        username: whoami::username(),
                        executable_paths: "".to_string(),
                        process_monitoring: ProcessMonitoringSettings {
                            enabled: false,
                            directory_depth: 3,
                        },
                        twitch_client_id: None,
                        twitch_client_secret: None,
                    };
                    let settings_str = toml::to_string(&settings).unwrap();
                    fs::write(config_path.join("settings.toml"), settings_str).unwrap();
                    user_settings = settings;
                }
            }
            if !user_settings.process_monitoring.enabled {
                return Ok(());
            }
            let executable_paths = user_settings.executable_paths.split(";");
            let mut paths_to_monitor: Vec<PathBuf> = Vec::new();
            for path in executable_paths {
                let path = Path::new(path);
                if path.is_dir() {
                    let walker = walkdir::WalkDir::new(path)
                        .max_depth(user_settings.process_monitoring.directory_depth as usize);
                    for entry in walker {
                        let entry = entry.unwrap();
                        if entry.file_type().is_file() {
                            let path = entry.path().to_string_lossy().to_string();
                            paths_to_monitor.push(path.into());
                        }
                    }
                } else {
                    paths_to_monitor.push(path.to_string_lossy().to_string().into());
                }
            }
            let app_handle = app.handle().clone();
            let handle = thread::spawn(move || {
                let mut process_monitor = process_monitor::ProcessMonitor::new();
                loop {
                    process_monitor
                        .monitor_processes(paths_to_monitor.clone(), &app_handle)
                        .unwrap();
                    thread::sleep(std::time::Duration::from_secs(1));
                }
            });
            handle.join().unwrap();
            Ok(())
        })
        .manage(std::sync::Mutex::new(
            database::initialize_database().unwrap(),
        ))
        .invoke_handler(tauri::generate_handler![
            database::get_current_username,
            database::get_dashboard_statistics,
            igdb::get_games_by_id,
            igdb::get_similar_games,
            igdb::authenticate_with_twitch,
            database::get_recent_logs,
            database::get_logs,
            get_user_settings,
            save_user_settings,
            database::delete_log,
            database::get_log_by_id,
            database::add_log,
            database::update_log,
            database::add_executable_details,
            igdb::get_random_top_games,
            igdb::search_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_user_settings(app_handle: tauri::AppHandle) -> Result<UserSettings, Error> {
    let config_path = app_handle.path().config_dir().unwrap();
    let mut file = fs::File::open(config_path.join("settings.toml"))?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    Ok(toml::from_str::<UserSettings>(&file_contents)?)
}

#[tauri::command]
fn save_user_settings(
    user_settings: UserSettings,
    app_handle: tauri::AppHandle,
) -> Result<UserSettings, Error> {
    let config_path = app_handle.path().config_dir().unwrap();
    let settings_str = toml::to_string(&user_settings)?;
    fs::write(config_path.join("settings.toml"), settings_str)?;
    Ok(user_settings)
}
