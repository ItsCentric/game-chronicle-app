use rusqlite::Connection;
use std::sync::Mutex;

use crate::Error;
use chrono::{Datelike, Local, Months};
use tauri::State;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecutableDetails {
    pub name: String,
    pub igdb_id: i32,
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
    pub title: String,
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub completed: bool,
    pub minutes_played: i32,
    pub igdb_id: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogData {
    pub title: String,
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub completed: bool,
    pub minutes_played: i32,
    pub igdb_id: i32,
}

pub fn initialize_database() -> Result<rusqlite::Connection, Error> {
    let tmp_dir = std::env::temp_dir();
    let conn = Connection::open(tmp_dir.to_str().unwrap().to_owned() + "/logs.db")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    title TEXT,
    date TEXT DEFAULT CURRENT_TIMESTAMP,
    rating INTEGER DEFAULT 0,
    notes TEXT,
    status TEXT,
    completed BOOLEAN DEFAULT FALSE,
    minutes_played INTEGER DEFAULT 0,
    igdb_id INTEGER,
    CONSTRAINT valid_rating CHECK (rating >= 0 AND rating <= 5),
    CONSTRAINT valid_status CHECK (status IN ('backlog', 'playing', 'completed', 'dropped'))
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
    igdb_id INTEGER,
    minutes_played INTEGER DEFAULT 0,
    CONSTRAINT unique_executable_name UNIQUE (executable_name)
);",
    )?;
    Ok(conn)
}

pub fn get_executable_details(
    conn: &Connection,
    executable_name: &str,
) -> Result<ExecutableDetails, Error> {
    let mut stmt =
        conn.prepare("SELECT name, igdb_id, minutes_played FROM executables WHERE name = ?")?;
    let executable = stmt.query_row(&[executable_name], |row| {
        Ok(ExecutableDetails {
            name: row.get(0)?,
            igdb_id: row.get(1)?,
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
        let mut stmt = conn.prepare("SELECT * FROM logs ORDER BY date DESC LIMIT ?")?;
        let logs = stmt
            .query_map([amount], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    updated_at: row.get(2)?,
                    title: row.get(3)?,
                    date: row.get(4)?,
                    rating: row.get(5)?,
                    notes: row.get(6)?,
                    status: row.get(7)?,
                    completed: row.get(8)?,
                    minutes_played: row.get(9)?,
                    igdb_id: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(logs);
    }
    let mut stmt =
        conn.prepare("SELECT * FROM logs WHERE status IN (?) ORDER BY date DESC LIMIT ?")?;
    let logs = stmt
        .query_map((filter.join(","), amount), |row| {
            Ok(Log {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
                date: row.get(4)?,
                rating: row.get(5)?,
                notes: row.get(6)?,
                status: row.get(7)?,
                completed: row.get(8)?,
                minutes_played: row.get(9)?,
                igdb_id: row.get(10)?,
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
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM logs WHERE status IN (?) ORDER BY ? {}",
            sort_order
        )
        .as_str(),
    )?;
    let logs = stmt
        .query_map([filter.join(","), sort_by], |row| {
            Ok(Log {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
                date: row.get(4)?,
                rating: row.get(5)?,
                notes: row.get(6)?,
                status: row.get(7)?,
                completed: row.get(8)?,
                minutes_played: row.get(9)?,
                igdb_id: row.get(10)?,
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
    let mut stmt = conn.prepare("SELECT * FROM logs WHERE id = ?")?;
    let log = stmt.query_row([id], |row| {
        Ok(Log {
            id: row.get(0)?,
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            title: row.get(3)?,
            date: row.get(4)?,
            rating: row.get(5)?,
            notes: row.get(6)?,
            status: row.get(7)?,
            completed: row.get(8)?,
            minutes_played: row.get(9)?,
            igdb_id: row.get(10)?,
        })
    })?;
    Ok(log)
}

#[tauri::command]
pub fn add_log(state: State<Mutex<Connection>>, log_data: LogData) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    conn.execute(
        "INSERT INTO logs (title, date, rating, notes, status, completed, minutes_played, igdb_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            log_data.title,
            log_data.date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.completed.to_string(),
            log_data.minutes_played.to_string(),
            log_data.igdb_id.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
pub fn update_log(
    state: State<Mutex<Connection>>,
    id: i32,
    log_data: LogData,
) -> Result<i32, Error> {
    let conn = state.lock().unwrap();
    conn.execute(
        "UPDATE logs SET title = ?1, date = ?2, rating = ?3, notes = ?4, status = ?5, completed = ?6, minutes_played = ?7, igdb_id = ?8 WHERE id = ?9",
        [
            log_data.title,
            log_data.date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.completed.to_string(),
            log_data.minutes_played.to_string(),
            log_data.igdb_id.to_string(),
            id.to_string(),
        ],
    )?;
    Ok(id)
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
            executable_details.igdb_id.to_string(),
            executable_details.minutes_played.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        let this_month = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
        let end_of_last_month = this_month.pred_opt().expect("valid date");
        println!("{:?}", this_month);
        println!("{:?}", end_of_last_month);
        assert_eq!(end_of_last_month.month(), 12);
    }
}
