use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};

use crate::{
    helpers::{create_dir_if_not_exists, get_app_data_directory, SchemaFieldUpdate},
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
    pub start_date: String,
    pub end_date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
    pub game_id: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogData {
    pub game_id: i32,
    pub start_date: String,
    pub end_date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogUpdateData {
    id: i32,
    pub start_date: String,
    pub end_date: String,
    pub rating: i32,
    pub notes: String,
    pub status: String,
    pub minutes_played: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SchemaModifier {
    table_name: String,
    schema: String,
}

impl SchemaModifier {
    fn load_schema(table_name: &str, conn: &Connection) -> Result<SchemaModifier, Error> {
        let mut stmt =
            conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name=?1")?;

        let mut rows = stmt.query([table_name])?;
        let schema: String;
        if let Some(row) = rows.next()? {
            schema = row.get(0)?;
        } else {
            return Err(Error::from(format!("Table '{}' not found", table_name)));
        }
        Ok(SchemaModifier {
            table_name: table_name.to_string(),
            schema,
        })
    }

    fn change_table_name(&self, new_table_name: &str) -> SchemaModifier {
        let new_schema = self.schema.replacen(&self.table_name, new_table_name, 1);
        SchemaModifier {
            table_name: new_table_name.to_string(),
            schema: new_schema,
        }
    }

    fn add_constraint(&self, constraint_name: &str, new_constraint: &str) -> SchemaModifier {
        let mut schema_lines = self.schema.lines().collect::<Vec<_>>();
        let constraint = format!("CONSTRAINT {} {}", constraint_name, new_constraint);
        schema_lines.insert(schema_lines.len() - 2, &constraint);
        let new_schema = schema_lines.join("\n");
        SchemaModifier {
            table_name: self.table_name.clone(),
            schema: new_schema,
        }
    }

    fn remove_constraint(&self, constraint_name: &str) -> SchemaModifier {
        let schema_lines = self.schema.lines().collect::<Vec<_>>();
        let new_schema_lines = schema_lines
            .iter()
            .filter(|line| !line.contains(constraint_name))
            .map(|line| *line)
            .collect::<Vec<&str>>();
        SchemaModifier {
            table_name: self.table_name.clone(),
            schema: new_schema_lines.join("\n"),
        }
    }

    fn commit(&self) -> String {
        self.schema.clone()
    }
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
        id: row.get("id")?,
        game_id: row.get("game_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        rating: row.get("rating")?,
        notes: row.get("notes")?,
        status: row.get("status")?,
        minutes_played: row.get("minutes_played")?,
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
    let minutes_and_games_played_stmt = conn.prepare("SELECT COALESCE(SUM(total_minutes_played), 0), COUNT(*) FROM ( SELECT COALESCE(SUM(minutes_played), 0) AS total_minutes_played FROM logs WHERE (end_date BETWEEN ?1 AND ?2) AND status != 'wishlist' GROUP BY game_id ) AS subquery;").optional()?;
    let this_minutes_and_games_played: (i32, i32) = match minutes_and_games_played_stmt {
        Some(mut stmt) => stmt.query_row([start_date.clone(), end_date.clone()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?,
        None => (0, 0),
    };
    let mut completed_games_stmt = conn.prepare(
        "SELECT COUNT(*) FROM logs WHERE (end_date BETWEEN ?1 AND ?2) AND status = 'completed'",
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
        let mut stmt = conn.prepare("SELECT * FROM logs ORDER BY end_date DESC LIMIT ?")?;
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
            "SELECT * FROM logs WHERE status IN ({}) ORDER BY end_date DESC LIMIT ?",
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
        "INSERT INTO logs (game_id, start_date, rating, notes, status, minutes_played, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            log_data.game_id.to_string(),
            log_data.start_date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.minutes_played.to_string(),
            log_data.end_date,
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
        "UPDATE logs SET start_date = ?1, end_date = ?7, rating = ?2, notes = ?3, status = ?4, minutes_played = ?5 WHERE id = ?6",
        [
            log_data.start_date,
            log_data.rating.to_string(),
            log_data.notes,
            log_data.status,
            log_data.minutes_played.to_string(),
            log_data.id.to_string(),
            log_data.end_date,
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

pub fn update_table_schema(
    conn: &mut Connection,
    table_name: &str,
    schema_updates: &HashMap<String, SchemaFieldUpdate>,
) -> Result<(), Error> {
    for (field_name, schema_update) in schema_updates {
        match schema_update.update_type {
            1 => {
                let default = match &schema_update.default {
                    Some(d) => d,
                    None => Err(Error::from(format!(
                        "No default value provided for {}",
                        field_name
                    )))?,
                };
                conn.execute(
                    format!(
                        "ALTER TABLE {} ADD COLUMN {} {}",
                        table_name, schema_update.new_name, schema_update.new_type
                    )
                    .as_str(),
                    [],
                )?;
                conn.execute(
                    format!(
                        "UPDATE {} SET {} = {}",
                        table_name, schema_update.new_name, default
                    )
                    .as_str(),
                    [],
                )?;
            }
            2 => {
                conn.execute(
                    format!(
                        "ALTER TABLE {} RENAME COLUMN {} TO {}",
                        table_name, field_name, schema_update.new_name
                    )
                    .as_str(),
                    [],
                )?;
            }
            3 => {
                conn.execute(
                    format!("ALTER TABLE {} DROP COLUMN {}", table_name, field_name).as_str(),
                    [],
                )?;
            }
            4 => {
                let constraint_data = match &schema_update.constraint_data {
                    Some(d) => d,
                    None => Err(Error::from(format!(
                        "No constraint data provided for {}",
                        field_name
                    )))?,
                };
                let temp_table_name = "new_".to_owned() + table_name;
                let new_schema = SchemaModifier::load_schema(table_name, &conn)?
                    .change_table_name(&temp_table_name)
                    .add_constraint(&field_name, constraint_data)
                    .commit();
                let transaction = conn.transaction()?;
                transaction.execute(&new_schema, [])?;
                transaction.execute(
                    &format!(
                        "INSERT INTO {} SELECT * FROM {}",
                        temp_table_name, table_name
                    ),
                    [],
                )?;
                transaction.execute(&format!("DROP TABLE {}", table_name), [])?;
                transaction.execute(
                    &format!("ALTER TABLE {} RENAME TO {}", temp_table_name, table_name),
                    [],
                )?;
                transaction.commit()?;
            }
            5 => {
                let constraint_data = match &schema_update.constraint_data {
                    Some(d) => d,
                    None => Err(Error::from(format!(
                        "No constraint data provided for {}",
                        field_name
                    )))?,
                };
                let temp_table_name = "new_".to_owned() + table_name;
                let new_schema = SchemaModifier::load_schema(table_name, &conn)?
                    .change_table_name(&temp_table_name)
                    .remove_constraint(&field_name)
                    .add_constraint(&schema_update.new_name, constraint_data)
                    .commit();
                let transaction = conn.transaction()?;
                transaction.execute(&new_schema, [])?;
                transaction.execute(
                    &format!(
                        "INSERT INTO {} SELECT * FROM {}",
                        temp_table_name, table_name
                    ),
                    [],
                )?;
                transaction.execute(&format!("DROP TABLE {}", table_name), [])?;
                transaction.execute(
                    &format!("ALTER TABLE {} RENAME TO {}", temp_table_name, table_name),
                    [],
                )?;
                transaction.commit()?;
            }
            6 => {
                let temp_table_name = "new_".to_owned() + table_name;
                let new_schema = SchemaModifier::load_schema(table_name, &conn)?
                    .change_table_name(&temp_table_name)
                    .remove_constraint(&field_name)
                    .commit();
                let transaction = conn.transaction()?;
                transaction.execute(&new_schema, [])?;
                transaction.execute(
                    &format!(
                        "INSERT INTO {} SELECT * FROM {}",
                        temp_table_name, table_name
                    ),
                    [],
                )?;
                transaction.execute(&format!("DROP TABLE {}", table_name), [])?;
                transaction.execute(
                    &format!("ALTER TABLE {} RENAME TO {}", temp_table_name, table_name),
                    [],
                )?;
                transaction.commit()?;
            }
            _ => {}
        }
    }
    Ok(())
}
