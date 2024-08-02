// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs::remove_file, path::PathBuf, thread};

use database::update_table_schema;
use serde::Deserialize;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_cli::CliExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::{NotificationExt, PermissionState};

use std::path::Path;

use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::{MouseButton::Left, TrayIconEvent::Click},
    Manager,
};

mod data_import;
mod database;
mod dumps;
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
    #[error(transparent)]
    Csv(#[from] csv::Error),
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
    autostart: bool,
    new: bool,
}

#[derive(serde::Serialize, Debug, Deserialize)]
struct ProcessMonitoringSettings {
    enabled: bool,
    directory_depth: usize,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

struct DatabaseConnections {
    logs_conn: std::sync::Mutex<rusqlite::Connection>,
    igdb_conn: std::sync::Mutex<rusqlite::Connection>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
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
                window.hide().unwrap();
                let windows = app_handle.webview_windows();
                let visible_windows = windows.iter().filter(|(_, window)| window.is_visible().unwrap()).collect::<HashMap<_, _>>();
                if visible_windows.len() == 0 {
                    app_handle.notification().builder().title("Game Chronicle").body("Game Chronicle is still running in the background.").show().unwrap();
                }
                api.prevent_close();
            },
            _ => {}
        })
        .setup(|app| {
            match app.cli().matches() {
                Ok(matches) => match matches.args.get("hidden") {
                    Some(is_hidden_set) => {
                        if is_hidden_set.value.as_bool().unwrap() {
                        match app.get_webview_window("main") {
                            Some(webview_window) => {
                                webview_window.close()?;
                            }
                            None => {}
                        }
                        }
                    },
                    None => {}
                },
                Err(_) => {}
            };
            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap();
            let menu = MenuBuilder::new(app).quit().build().unwrap();
            tauri::tray::TrayIconBuilder::new().title("Game Chronicle").tooltip("Game Chronicle").icon(tray_icon).menu(&menu)
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
            let user_settings = match helpers::get_user_settings(app.handle().clone()) {
                Ok(user_settings) => user_settings,
                Err(_) => {
                    let settings = UserSettings {
                        username: whoami::username(),
                        executable_paths: None,
                        process_monitoring: ProcessMonitoringSettings {
                            enabled: false,
                            directory_depth: 2,
                        },
                        autostart: false,
                        new: true,
                    };
                    match helpers::create_dir_if_not_exists(app.path().config_dir()?.join("game-chronicle").as_path()) {
                        Ok(_) => {}
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::PermissionDenied => {
                                app.dialog().message("Could not create needed files. Please run the application as an administrator.").title("Permission denied").kind(tauri_plugin_dialog::MessageDialogKind::Error).blocking_show();
                                app.handle().exit(0);
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
            let autostart_manager = app.autolaunch();
            if user_settings.autostart && !autostart_manager.is_enabled().unwrap() {
                autostart_manager.enable().unwrap();
            } else if !user_settings.autostart && autostart_manager.is_enabled().unwrap() {
                autostart_manager.disable().unwrap();
            }
            let (logs_conn, igdb_conn) = database::initialize_database(app.handle().clone()).unwrap();
            igdb_conn.execute("INSERT INTO games_fts (games_fts) VALUES ('rebuild')", rusqlite::params![])?;
            app.manage(DatabaseConnections {
                logs_conn: std::sync::Mutex::new(logs_conn),
                igdb_conn: std::sync::Mutex::new(igdb_conn),
            });
            let schema_changes = helpers::get_schema_changes(app.handle())?;
            if let Some(igdb_changes) = schema_changes.igdb {
                for (table_name, changes) in igdb_changes {
                    update_table_schema(&app.state::<DatabaseConnections>().igdb_conn.lock().unwrap(), &table_name, changes)?;
                }
            }
            if let Some(log_changes) = schema_changes.logs {
                for (table_name, changes) in log_changes {
                    update_table_schema(&app.state::<DatabaseConnections>().logs_conn.lock().unwrap(), &table_name, changes)?;
                }
            }
            remove_file(app.path().resource_dir()?.join("resources/schema_changes.toml"))?;
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
            database::get_recent_logs,
            database::get_logs,
            helpers::get_user_settings,
            helpers::save_user_settings,
            database::delete_log,
            database::get_log_by_id,
            database::add_log,
            database::update_log,
            database::add_executable_details,
            igdb::get_popular_games,
            igdb::search_game,
            data_import::get_steam_data,
            data_import::import_igdb_games,
            dumps::get_local_dump_versions,
            dumps::save_local_dump_versions,
            dumps::get_all_dump_info,
            dumps::import_dumps,
            dumps::download_dumps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
