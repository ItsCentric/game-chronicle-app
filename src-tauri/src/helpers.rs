use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tauri::{Manager, State};

use crate::{Error, UserSettings};

use rusqlite::Connection;

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

#[tauri::command]
pub fn get_current_username(state: State<Mutex<Connection>>) -> Result<String, Error> {
    let conn = state.lock().unwrap();
    let mut stmt = conn.prepare("SELECT username FROM user_settings")?;
    match stmt.query_row([], |row| Ok(row.get(0)?)) {
        Ok(username) => Ok(username),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(whoami::username()),
        Err(e) => Err(Error::from(e)),
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
