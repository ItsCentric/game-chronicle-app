use rusqlite::{Connection, OptionalExtension};

use crate::{
    helpers::{create_dir_if_not_exists, get_app_data_directory},
    DatabaseConnections, Error,
};
use tauri::State;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecutableDetails {
    pub name: String,
    pub game_id: i32,
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
    pub game_id: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogData {
    pub game_id: i32,
    pub date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
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

pub fn initialize_database(
    app_handle: tauri::AppHandle,
) -> Result<(rusqlite::Connection, rusqlite::Connection), Error> {
    let data_dir = get_app_data_directory(&app_handle)?;
    create_dir_if_not_exists(data_dir.as_path())?;
    let logs_conn = Connection::open(data_dir.join("logs.db"))?;
    let sql_file_contents = include_str!("../sql/initialize_database.sql");
    logs_conn.execute_batch(sql_file_contents)?;
    let conn = Connection::open(data_dir.join("igdb.db"))?;
    let igdb_sql_file_contents = include_str!("../sql/initialize_igdb_database.sql");
    conn.execute_batch(igdb_sql_file_contents)?;
    Ok((logs_conn, conn))
}

fn log_from_row(row: &rusqlite::Row) -> Result<Log, rusqlite::Error> {
    Ok(Log {
        id: row.get(0)?,
        game_id: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        date: row.get(4)?,
        rating: row.get(5)?,
        notes: row.get(6)?,
        status: row.get(7)?,
        minutes_played: row.get(8)?,
    })
}

pub fn get_executable_details(
    conn: &Connection,
    executable_name: &str,
) -> Result<ExecutableDetails, Error> {
    let mut stmt = conn.prepare(
        "SELECT executable_name, game_id FROM executable_details WHERE executable_name = ?",
    )?;
    let executable = stmt.query_row(&[executable_name], |row| {
        Ok(ExecutableDetails {
            name: row.get(0)?,
            game_id: row.get(1)?,
        })
    })?;
    Ok(executable)
}

#[tauri::command]
pub fn get_dashboard_statistics(
    state: State<DatabaseConnections>,
    start_date: String,
    end_date: String,
) -> Result<DashboardStatistics, Error> {
    let conn = state.logs_conn.lock().unwrap();
    let minutes_and_games_played_stmt = conn.prepare("SELECT COALESCE(SUM(total_minutes_played), 0), COUNT(*) FROM ( SELECT COALESCE(SUM(minutes_played), 0) AS total_minutes_played FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status != 'wishlist' GROUP BY game_id ) AS subquery;").optional()?;
    let this_minutes_and_games_played: (i32, i32) = match minutes_and_games_played_stmt {
        Some(mut stmt) => stmt.query_row([start_date.clone(), end_date.clone()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?,
        None => (0, 0),
    };
    let mut completed_games_stmt = conn.prepare(
        "SELECT COUNT(*) FROM logs WHERE (date BETWEEN ?1 AND ?2) AND status = 'completed'",
    )?;
    let this_completed_games: i32 =
        completed_games_stmt.query_row([start_date.clone(), end_date.clone()], |row| {
            Ok(row.get(0)?)
        })?;
    Ok(DashboardStatistics {
        total_minutes_played: this_minutes_and_games_played.0,
        total_games_played: this_minutes_and_games_played.1,
        total_games_completed: this_completed_games,
    })
}

#[tauri::command]
pub fn get_recent_logs(
    state: State<DatabaseConnections>,
    amount: i32,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    let conn = state.logs_conn.lock().unwrap();
    if filter.len() == 0 {
        let mut stmt = conn.prepare("SELECT * FROM logs ORDER BY date DESC LIMIT ?")?;
        let logs = stmt
            .query_map([amount], |row| Ok(log_from_row(row)?))?
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(logs);
    }
    let joined_filter = filter
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM logs WHERE status IN ({}) ORDER BY date DESC LIMIT ?",
            joined_filter
        )
        .as_str(),
    )?;
    let logs = stmt
        .query_map([amount], |row| Ok(log_from_row(row)?))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(logs)
}

#[tauri::command]
pub fn get_logs(
    state: State<DatabaseConnections>,
    sort_by: String,
    sort_order: String,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    let conn = state.logs_conn.lock().unwrap();
    let joined_filter = filter
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM logs WHERE status IN ({}) ORDER BY ? {}",
            joined_filter, sort_order
        )
        .as_str(),
    )?;
    let logs = stmt
        .query_map([sort_by], |row| Ok(log_from_row(row)?))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(logs)
}

#[tauri::command]
pub fn delete_log(state: State<DatabaseConnections>, id: i32) -> Result<i32, Error> {
    let conn = state.logs_conn.lock().unwrap();
    conn.execute("DELETE FROM logs WHERE id = ?", [id])?;
    Ok(id)
}

#[tauri::command]
pub fn get_log_by_id(state: State<DatabaseConnections>, id: i32) -> Result<Log, Error> {
    let conn = state.logs_conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT * FROM logs WHERE logs.id = ?")?;
    let log = stmt.query_row([id], |row| Ok(log_from_row(row)?))?;
    Ok(log)
}

#[tauri::command]
pub fn add_log(state: State<DatabaseConnections>, log_data: LogData) -> Result<i32, Error> {
    let conn = state.logs_conn.lock().unwrap();
    conn.execute(
        "INSERT INTO logs (game_id, date, rating, notes, status, minutes_played) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            log_data.game_id.to_string(),
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
pub fn update_log(
    state: State<DatabaseConnections>,
    log_data: LogUpdateData,
) -> Result<i32, Error> {
    let conn = state.logs_conn.lock().unwrap();
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
    state: State<DatabaseConnections>,
    executable_details: ExecutableDetails,
) -> Result<i32, Error> {
    let conn = state.logs_conn.lock().unwrap();
    conn.execute(
        "INSERT INTO executable_details (executable_name, game_id) VALUES (?1, ?2)",
        [
            executable_details.name,
            executable_details.game_id.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}
