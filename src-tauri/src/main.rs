// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, thread};

use serde::Deserialize;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::{NotificationExt, PermissionState};

use std::path::Path;

use tauri::{image::Image, tray::MouseButton::Left, tray::TrayIconEvent::Click, Manager};

mod data_import;
mod database;
mod helpers;
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
    executable_paths: Option<String>,
    process_monitoring: ProcessMonitoringSettings,
    twitch_client_id: Option<String>,
    twitch_client_secret: Option<String>,
    new: bool,
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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, ..} => {
                let app_handle = window.app_handle();
                let mut notification_permission_state = app_handle.notification().permission_state().unwrap();
                if notification_permission_state != PermissionState::Granted {
                    notification_permission_state = app_handle.notification().request_permission().unwrap();
                    if notification_permission_state != PermissionState::Granted {
                        return;
                    }
                }
                app_handle.notification().builder().title("Game Chronicle").body("Game Chronicle is still running in the background.").show().unwrap();
                window.hide().unwrap();
                api.prevent_close();
            },
            _ => {}
        })
        .setup(|app| {
            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap();
            tauri::tray::TrayIconBuilder::new().title("Game Chronicle").tooltip("Game Chronicle").icon(tray_icon)
            .on_tray_icon_event(|tray, event| {
                match event {
                    Click { id: _, position: _, rect: _, button: mouse_button, .. } => {
                        if mouse_button == Left {
                            let app = tray.app_handle();
                            if let Some(webview_window) = app.get_webview_window("main") {
                                let _ = webview_window.show();
                                let _ = webview_window.set_focus();
                            }
                        }
                    },
                    _ => {}
                }
            })
            .build(app)?;
            let conn = database::initialize_database(app.handle().clone()).unwrap();
            app.manage(std::sync::Mutex::new(conn));
            let user_settings = match helpers::get_user_settings(app.handle().clone()) {
                Ok(user_settings) => user_settings,
                Err(_) => {
                    let settings = UserSettings {
                        username: whoami::username(),
                        executable_paths: None,
                        process_monitoring: ProcessMonitoringSettings {
                            enabled: false,
                            directory_depth: 3,
                        },
                        twitch_client_id: None,
                        twitch_client_secret: None,
                        new: true,
                    };
                    match helpers::create_dir_if_not_exists(app.path().config_dir()?.join("game-chronicle").as_path()) {
                        Ok(_) => {}
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::PermissionDenied => {
                                app.dialog().message("Could not create needed files. Please run the application as an administrator.").title("Permission denied").kind(tauri_plugin_dialog::MessageDialogKind::Error).blocking_show();
                                app.handle().exit(1);
                            }
                            e => {
                                panic!("{}", e)
                            }
                        },
                    }
                    let saved_settings = helpers::save_user_settings(settings, app.handle().clone())?;
                    saved_settings
                }
            };
            if !user_settings.process_monitoring.enabled || user_settings.executable_paths.is_none() {
                return Ok(());
            }
            let executable_paths = user_settings.executable_paths.expect("None check to not fail");
            let executable_paths_vec = executable_paths.split(";");
            let mut paths_to_monitor: Vec<PathBuf> = Vec::new();
            for path in executable_paths_vec {
                let path = Path::new(path);
                if path.is_dir() {
                    let walker = walkdir::WalkDir::new(path)
                        .max_depth(user_settings.process_monitoring.directory_depth as usize);
                    for entry in walker {
                        let entry = entry.unwrap();
                        if entry.file_type().is_file() && !entry.path().to_string_lossy().contains("CrashHandler") {
                            let path = entry.path().to_string_lossy().to_string();
                            paths_to_monitor.push(path.into());
                        }
                    }
                } else {
                    paths_to_monitor.push(path.to_string_lossy().to_string().into());
                }
            }
            let app_handle = app.handle().clone();
            thread::spawn(move || {
                let mut process_monitor = process_monitor::ProcessMonitor::new();
                loop {
                    process_monitor
                        .monitor_processes(paths_to_monitor.clone(), &app_handle)
                        .unwrap();
                    thread::sleep(std::time::Duration::from_secs(1));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            database::get_dashboard_statistics,
            igdb::get_games_by_id,
            igdb::get_similar_games,
            igdb::authenticate_with_twitch,
            database::get_recent_logs,
            database::get_logs,
            helpers::get_user_settings,
            helpers::save_user_settings,
            database::delete_log,
            database::get_log_by_id,
            database::add_log,
            database::update_log,
            database::add_executable_details,
            igdb::get_random_top_games,
            igdb::search_game,
            database::get_logged_game,
            data_import::get_steam_data,
            data_import::import_igdb_games,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
