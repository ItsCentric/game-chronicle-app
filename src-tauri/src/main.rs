// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, path::PathBuf, thread};

use db::{igdb::init_igdb_db, logs::init_logs_db, IgdbDb, LogsDb};
use serde::Deserialize;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_cli::CliExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::{NotificationExt, PermissionState};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

use std::path::Path;

use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::{MouseButton::Left, TrayIconEvent::Click},
    Manager, Url,
};

mod data_import;
mod db;
mod dumps;
mod helpers;
mod process_monitor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Env(#[from] std::env::VarError),
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
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
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
    beta: bool,
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

struct DatabasePools {
    logs_pool: LogsDb,
    igdb_pool: IgdbDb,
}

fn main() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();
    let mut tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().with_state_flags(StateFlags::all() & !StateFlags::VISIBLE).build())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_window_event(|window, event| if let tauri::WindowEvent::CloseRequested { api, ..} = event {
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
            if visible_windows.is_empty() {
                app_handle.notification().builder().title("Game Chronicle").body("Game Chronicle is still running in the background.").show().unwrap();
            }
            api.prevent_close();
        })
        .setup(move |app| {
            if let Ok(matches) = app.cli().matches() {
                if let Some(is_hidden_set) = matches.args.get("hidden") {
                    if is_hidden_set.value.as_bool().unwrap() {
                    if let Some(webview_window) = app.get_webview_window("main") {
                        webview_window.close()?;
                    }
                    }
                }
            };
            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap();
            let menu = MenuBuilder::new(app).quit().build().unwrap();
            tauri::tray::TrayIconBuilder::new().title("Game Chronicle").tooltip("Game Chronicle").icon(tray_icon).menu(&menu)
            .on_tray_icon_event(|tray, event| {
                if let Click { id: _, position: _, rect: _, button: mouse_button, .. } = event {
                    if mouse_button == Left {
                        let app = tray.app_handle();
                        if let Some(webview_window) = app.get_webview_window("main") {
                            let _ = webview_window.show();
                            let _ = webview_window.set_focus();
                        }
                    }
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
                        beta: false,
                    };
                    match helpers::create_dir_if_not_exists(app.path().app_config_dir()?.as_path()) {
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
                    helpers::save_user_settings(settings, app.handle().clone())?
                }
            };
            let version = if user_settings.beta { "{{version}}" } else { "latest" };
            app.updater_builder().endpoints(vec![Url::parse(&format!("https://github.com/ItsCentric/game-chronicle-app/releases/tag/v{}", version))?])?.build()?;
            let autostart_manager = app.autolaunch();
            if user_settings.autostart && !autostart_manager.is_enabled().unwrap() {
                autostart_manager.enable().unwrap();
            } else if !user_settings.autostart && autostart_manager.is_enabled().unwrap() {
                autostart_manager.disable().unwrap();
            }

            let app_data_dir = if tauri::is_dev() { 
                let dev_path = app.path().app_data_dir()?.join("dev");
                let data_path = app.path().app_data_dir()?;
                helpers::create_dir_if_not_exists(dev_path.as_path())?;
                if std::fs::exists(data_path.join("logs.db"))? {
                    std::fs::copy(data_path.join("logs.db"), dev_path.join("logs.db"))?;
                }
                dev_path
            } else {
                app.path().app_data_dir()?
            };
            let app_data_path = app_data_dir.as_path();
            let logs_pool = tauri::async_runtime::block_on(init_logs_db(app_data_path))?;
            let igdb_pool = tauri::async_runtime::block_on(init_igdb_db(app_data_path))?;
            tauri::async_runtime::block_on(
                sqlx::query("INSERT INTO games_fts (games_fts) VALUES ('rebuild')").execute(&igdb_pool)
            )?;
            app.manage(DatabasePools {
                logs_pool,
                igdb_pool,
            });
            if !user_settings.process_monitoring.enabled || user_settings.executable_paths.is_none() {
                return Ok(());
            }
            let executable_paths = user_settings.executable_paths.expect("None check to not fail");
            let executable_paths_vec = executable_paths.split(';');
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
            tauri::async_runtime::spawn(async move {
                let mut process_monitor = process_monitor::ProcessMonitor::new();
                loop {
                    process_monitor
                        .monitor_processes(paths_to_monitor.clone(), &app_handle).await
                        .unwrap();
                    thread::sleep(std::time::Duration::from_secs(1));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db::logs::get_dashboard_statistics,
            db::igdb::get_games_by_id,
            db::logs::get_recent_logs,
            db::logs::get_logs,
            helpers::get_user_settings,
            helpers::save_user_settings,
            db::logs::delete_log,
            db::logs::get_log_by_id,
            db::logs::add_log,
            db::logs::update_log,
            db::logs::add_executable_details,
            db::igdb::get_popular_games,
            db::igdb::search_game,
            data_import::get_steam_data,
            data_import::import_igdb_games,
            dumps::import_igdb_dumps,
        ]);
    #[cfg(debug_assertions)]
    {
        tauri_builder = tauri_builder.plugin(devtools);
    }

    tauri_builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
