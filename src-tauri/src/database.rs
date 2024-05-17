use rusqlite::{Connection, OptionalExtension};
use std::{fs, sync::Mutex};

use crate::Error;
use chrono::{Datelike, Local, Months};
use tauri::{Manager, State};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecutableDetails {
    pub name: String,
    pub game_id: i32,
    pub minutes_played: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardStatistics {
    pub total_minutes_played: i32,
    pub total_games_played: i32,
    pub total_games_completed: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Log {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
    pub game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogData {
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
    pub game: Game,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogUpdateData {
    id: i32,
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub id: i32,
    pub title: String,
    pub cover_id: String,
}

pub fn initialize_database(app_handle: tauri::AppHandle) -> Result<rusqlite::Connection, Error> {
    let data_dir = app_handle
        .path()
        .data_dir()
        .unwrap()
        .to_str()
        .to_owned()
        .unwrap()
        .to_owned()
        + "/game-chronicle";
    match fs::File::open(data_dir.clone()) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                fs::create_dir(data_dir.clone())?;
            }
            _ => {}
        },
    }
    let conn = Connection::open(data_dir.as_str().to_owned() + "/data.db")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    date TEXT DEFAULT CURRENT_TIMESTAMP,
    rating INTEGER DEFAULT 0,
    notes TEXT,
    status TEXT,
    minutes_played INTEGER DEFAULT 0,
    FOREIGN KEY (game_id) REFERENCES logged_games(id),
    CONSTRAINT valid_rating CHECK (rating >= 0 AND rating <= 5),
    CONSTRAINT valid_status CHECK (status IN ('wishlist', 'backlog', 'playing', 'completed', 'played', 'abandoned', 'retired'))
    CONSTRAINT valid_date CHECK (date(date) IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    username TEXT,
    executable_paths TEXT,
    process_monitoring_enabled INTEGER DEFAULT 0,
    process_monitoring_directory_depth INTEGER DEFAULT 3,
  	CONSTRAINT boolean_enabled CHECK (process_monitoring_enabled IN (0,1))
);

CREATE TABLE IF NOT EXISTS executable_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    executable_name TEXT,
    game_id INTEGER NOT NULL,
    minutes_played INTEGER DEFAULT 0,
    FOREIGN KEY (game_id) REFERENCES logged_games(id),
    CONSTRAINT unique_executable_name UNIQUE (executable_name)
);

CREATE TABLE IF NOT EXISTS logged_games (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    cover_id TEXT,
    UNIQUE (title)
);
",
    )?;
    Ok(conn)
}

pub fn get_executable_details(
    conn: &Connection,
    executable_name: &str,
) -> Result<ExecutableDetails, Error> {
    let mut stmt =
        conn.prepare("SELECT name, game_id, minutes_played FROM executables WHERE name = ?")?;
    let executable = stmt.query_row(&[executable_name], |row| {
        Ok(ExecutableDetails {
            name: row.get(0)?,
            game_id: row.get(1)?,
            minutes_played: row.get(2)?,
        })
    })?;
    Ok(executable)
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

#[tauri::command]
pub fn get_dashboard_statistics(
    state: State<Mutex<Connection>>,
) -> Result<(DashboardStatistics, DashboardStatistics), Error> {
    let conn = state.lock().unwrap();
    let this_month = Local::now().date_naive();
    let beginning_of_this_month = this_month.with_day(1).expect("this_month to be valid");
    let end_of_last_month = beginning_of_this_month
        .pred_opt()
        .expect("beginning_of_this_month to not be epoch");
    let this_month_statistics_range = [
        end_of_last_month.to_string(),
        beginning_of_this_month
            .checked_add_months(Months::new(1))
            .expect("Next month to be in range")
            .to_string(),
    ];
    let last_month_statistics_range = [
        beginning_of_this_month
            .checked_sub_months(Months::new(1))
            .expect("Previous month to be in range")
            .to_string(),
        beginning_of_this_month.to_string(),
    ];
    let mut this_minutes_and_games_played_stmt = conn.prepare("SELECT COALESCE(SUM(minutes_played), 0), COUNT(*) FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status != 'wishlist'")?;
    let this_minutes_and_games_played: (i32, i32) = this_minutes_and_games_played_stmt
        .query_row(this_month_statistics_range.clone(), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    let mut this_completed_games_stmt = conn.prepare(
        "SELECT COUNT(*) FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status = 'completed'",
    )?;
    let this_completed_games: i32 =
        this_completed_games_stmt.query_row(this_month_statistics_range, |row| Ok(row.get(0)?))?;
    let mut last_minutes_and_games_played_stmt = conn.prepare("SELECT COALESCE(SUM(minutes_played), 0), COUNT(*) FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status != 'wishlist'")?;
    let last_minutes_and_games_played: (i32, i32) = last_minutes_and_games_played_stmt
        .query_row(last_month_statistics_range.clone(), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    let mut last_completed_games_stmt = conn.prepare(
        "SELECT COUNT(*) FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status = 'completed'",
    )?;
    let last_completed_games: i32 =
        last_completed_games_stmt.query_row(last_month_statistics_range, |row| Ok(row.get(0)?))?;
    Ok((
        DashboardStatistics {
            total_minutes_played: this_minutes_and_games_played.0,
            total_games_played: this_minutes_and_games_played.1,
            total_games_completed: this_completed_games,
        },
        DashboardStatistics {
            total_minutes_played: last_minutes_and_games_played.0,
            total_games_played: last_minutes_and_games_played.1,
            total_games_completed: last_completed_games,
        },
    ))
}

#[tauri::command]
pub fn get_recent_logs(
    state: State<Mutex<Connection>>,
    amount: i32,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    let conn = state.lock().unwrap();
    if filter.len() == 0 {
        let mut stmt = conn.prepare("SELECT * FROM logs JOIN logged_games ON logged_games.id = logs.game_id ORDER BY date DESC LIMIT ?")?;
        let logs = stmt
            .query_map([amount], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    date: row.get(4)?,
                    rating: row.get(5)?,
                    notes: row.get(6)?,
                    status: row.get(7)?,
                    minutes_played: row.get(8)?,
                    game: Game {
                        id: row.get(9)?,
                        title: row.get(10)?,
                        cover_id: row.get(11)?,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(logs);
    }
    let mut stmt =
        conn.prepare("SELECT * FROM logs JOIN logged_games ON logged_games.id = logs.game_id WHERE status IN (?) ORDER BY date DESC LIMIT ?")?;
    let logs = stmt
        .query_map((filter.join(","), amount), |row| {
            Ok(Log {
                id: row.get(0)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                date: row.get(4)?,
                rating: row.get(5)?,
                notes: row.get(6)?,
                status: row.get(7)?,
                minutes_played: row.get(8)?,
                game: Game {
                    id: row.get(9)?,
                    title: row.get(10)?,
                    cover_id: row.get(11)?,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(logs)
}

#[tauri::command]
pub fn get_logs(
    state: State<Mutex<Connection>>,
    sort_by: String,
    sort_order: String,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    let conn = state.lock().unwrap();
    let joined_filter = filter
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM logs JOIN logged_games ON logged_games.id = logs.game_id WHERE status IN ({}) ORDER BY ? {}",
            joined_filter, sort_order
        )
        .as_str(),
    )?;
    let logs = stmt
        .query_map([sort_by], |row| {
            Ok(Log {
                id: row.get(0)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                date: row.get(4)?,
                rating: row.get(5)?,
                notes: row.get(6)?,
                status: row.get(7)?,
                minutes_played: row.get(8)?,
                game: Game {
                    id: row.get(9)?,
                    title: row.get(10)?,
                    cover_id: row.get(11)?,
                },
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(logs)
}

#[tauri::command]
pub fn delete_log(state: State<Mutex<Connection>>, id: i32) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    conn.execute("DELETE FROM logs WHERE id = ?", [id])?;
    Ok(id)
}

#[tauri::command]
pub fn get_log_by_id(state: State<Mutex<Connection>>, id: i32) -> Result<Log, Error> {
    let conn = state.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT * FROM logs JOIN logged_games ON logged_games.id = logs.game_id WHERE logs.id = ?",
    )?;
    let log = stmt.query_row([id], |row| {
        Ok(Log {
            id: row.get(0)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            date: row.get(4)?,
            rating: row.get(5)?,
            notes: row.get(6)?,
            status: row.get(7)?,
            minutes_played: row.get(8)?,
            game: Game {
                id: row.get(9)?,
                title: row.get(10)?,
                cover_id: row.get(11)?,
            },
        })
    })?;
    Ok(log)
}

#[tauri::command]
pub fn add_log(state: State<Mutex<Connection>>, log_data: LogData) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id FROM logged_games WHERE id = ?")?;
    let game = stmt
        .query_row([log_data.game.id.to_string()], |row| Ok(row.get(0)?))
        .optional()?;
    let game_id = match game {
        Some(id) => id,
        None => {
            conn.execute(
                "INSERT INTO logged_games (id, title, cover_id) VALUES (?1, ?2, ?3)",
                [
                    log_data.game.id.to_string(),
                    log_data.game.title,
                    log_data.game.cover_id,
                ],
            )?;
            conn.last_insert_rowid() as i32
        }
    };
    conn.execute(
        "INSERT INTO logs (game_id, date, rating, notes, status, minutes_played) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            game_id.to_string(),
            log_data.date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.minutes_played.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub fn update_log(state: State<Mutex<Connection>>, log_data: LogUpdateData) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    conn.execute(
        "UPDATE logs SET date = ?1, rating = ?2, notes = ?3, status = ?4, minutes_played = ?5 WHERE id = ?6",
        [
            log_data.date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.minutes_played.to_string(),
            log_data.id.to_string(),
        ],
    )?;
    Ok(log_data.id)
}

#[tauri::command]
pub fn add_executable_details(
    state: State<Mutex<Connection>>,
    executable_details: ExecutableDetails,
) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    conn.execute(
        "INSERT INTO executable_details (executable_name, igdb_id, minutes_played) VALUES (?1, ?2, ?3)",
        [
            executable_details.name,
            executable_details.game_id.to_string(),
            executable_details.minutes_played.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}
