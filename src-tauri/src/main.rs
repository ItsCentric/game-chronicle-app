// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, path::PathBuf, thread};

use helpers::{get_app_data_directory, get_csv_data_blocking, get_csv_url_blocking};
use igdb::parse_csv;
use serde::Deserialize;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
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
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
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
            let app_data_dir = get_app_data_directory(app.handle())?;
            if app_data_dir.join("igdb.db").exists() {
                std::fs::remove_file(get_app_data_directory(app.handle())?.join("igdb.db"))?;
            };
            let (logs_conn, mut igdb_conn) = database::initialize_database(app.handle().clone()).unwrap();
            let temp_dir = app.handle().path().temp_dir()?.join("game-chronicle");
            std::fs::create_dir_all(&temp_dir)?;
            let endpoints = ["covers", "websites", "platforms", "games"];
            for endpoint in endpoints.iter() {
                let csv_url = get_csv_url_blocking(endpoint)?;
                let csv_data = get_csv_data_blocking(&csv_url)?;
                let mut csv_file = File::create(temp_dir.join(format!("{}.csv", endpoint)))?;
                std::io::copy(&mut csv_data.as_bytes(), &mut csv_file)?;
            }
            let covers: Vec<igdb::Cover> = parse_csv(&temp_dir.join("covers.csv"))?;
            let cover_transaction = igdb_conn.transaction()?;
            {
                let mut cover_statement = cover_transaction.prepare("INSERT INTO covers (id, image_id) VALUES (?1, ?2)")?;
                for cover in covers {
                    cover_statement.execute(
                        (cover.id, cover.image_id)
                    )?;
                }
            }
            cover_transaction.commit()?;
            let websites: Vec<igdb::Website> = parse_csv(&temp_dir.join("websites.csv"))?;
            let website_transaction = igdb_conn.transaction()?;
            {
                let mut website_statement = website_transaction.prepare("INSERT INTO websites (id, url) VALUES (?1, ?2)")?;
                for website in websites {
                    website_statement.execute(
                        (website.id, website.url)
                    )?;
                }
            }
            website_transaction.commit()?;
            let platforms: Vec<igdb::Platform> = parse_csv(&temp_dir.join("platforms.csv"))?;
            let platform_transaction = igdb_conn.transaction()?;
            {
                let mut platform_statement = platform_transaction.prepare("INSERT INTO platforms (id, name, category) VALUES (?1, ?2, ?3)")?;
                for platform in platforms {
                    platform_statement.execute(
                        (platform.id, platform.name, platform.category)
                    )?;
                }
            }
            platform_transaction.commit()?;
            let games: Vec<igdb::Game> = parse_csv(&temp_dir.join("games.csv"))?;
            let game_transaction = igdb_conn.transaction()?;
            {
                let mut game_statement = game_transaction.prepare("INSERT INTO games (id, name, cover_id, category, version_parent, total_rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")?;
                let mut game_websites_statement = game_transaction.prepare("INSERT INTO game_websites (game_id, website_id) VALUES (?1, ?2)")?;
                let mut similar_games_statement = game_transaction.prepare("INSERT INTO similar_games (game_id, similar_game_id) VALUES (?1, ?2)")?;
                let mut game_platforms_statement = game_transaction.prepare("INSERT INTO game_platforms (game_id, platform_id) VALUES (?1, ?2)")?;
                for game in &games {
                    match game_statement.execute(
                        (game.id, game.name.clone(), game.cover_id, game.category, game.version_parent, game.total_rating)
                    ) {
                        Ok(_) => {}
                        Err(_) => {
                        }
                    };
                    match &game.website_ids {
                        Some(website_ids) => {
                            for website_id in website_ids {
                                match game_websites_statement.execute(
                                    (game.id, website_id)
                                ) {
                                    Ok(_) => {}
                                    Err(_) => {
                                    }
                                };
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                    match &game.similar_games {
                        Some(similar_games) => {
                            for similar_game_id in similar_games {
                                match similar_games_statement.execute(
                                    (game.id, similar_game_id)
                                ) {
                                    Ok(_) => {}
                                    Err(_) => {
                                    }
                                };
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                    match &game.platform_ids {
                        Some(platform_ids) => {
                            for platform_id in platform_ids {
                                match game_platforms_statement.execute(
                                    (game.id, platform_id)
                                ) {
                                    Ok(_) => {},
                                    Err(_) => {
                                    }
                                };
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }
            game_transaction.commit()?;
            igdb_conn.execute("INSERT INTO games_fts (rowid, name) SELECT id, name FROM games;", [])?;
            std::fs::remove_dir_all(temp_dir)?;
            app.manage(DatabaseConnections {
                logs_conn: std::sync::Mutex::new(logs_conn),
                igdb_conn: std::sync::Mutex::new(igdb_conn),
            });
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
            igdb::get_random_top_games,
            igdb::search_game,
            data_import::get_steam_data,
            data_import::import_igdb_games,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
