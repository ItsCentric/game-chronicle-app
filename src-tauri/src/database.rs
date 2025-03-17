use crate::{
    db::LogsDb,
    DatabasePools, Error,
};
use tauri::State;

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
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

pub async fn get_executable_details(
    logs_pool: &LogsDb,
    executable_name: &str,
) -> Result<ExecutableDetails, Error> {
    let executable = sqlx::query_as::<_, ExecutableDetails>(
        "SELECT executable_name, game_id FROM executable_details WHERE executable_name = $1",
    )
    .bind(executable_name)
    .fetch_one(logs_pool).await?;
    Ok(executable)
}

#[tauri::command]
pub async fn get_dashboard_statistics(
    state: State<'_, DatabasePools>,
    start_date: String,
    end_date: String,
) -> Result<DashboardStatistics, Error> {
    let this_minutes_and_games_played: (i32, i32) = sqlx::query_as("SELECT COALESCE(SUM(total_minutes_played), 0), COUNT(*) FROM ( SELECT COALESCE(SUM(minutes_played), 0) AS total_minutes_played FROM logs WHERE (end_date BETWEEN $1 AND $2) AND status != 'wishlist' GROUP BY game_id ) AS subquery;").bind(start_date.clone()).bind(end_date.clone()).fetch_one(&state.logs_pool).await?;
    let completed_games: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM logs WHERE (end_date BETWEEN $1 AND $2) AND status = 'completed'").bind(start_date).bind(end_date).fetch_one(&state.logs_pool).await?;
    Ok(DashboardStatistics {
        total_minutes_played: this_minutes_and_games_played.0,
        total_games_played: this_minutes_and_games_played.1,
        total_games_completed: completed_games,
    })
}

#[tauri::command]
pub async fn get_recent_logs(
    state: State<'_, DatabasePools>,
    amount: i32,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    if filter.is_empty() {
        let logs: Vec<Log> = sqlx::query_as("SELECT id, game_id, created_at, updated_at, start_date, end_date, rating, notes, status, minutes_played FROM logs ORDER BY end_date DESC LIMIT $1").bind(amount).fetch_all(&state.logs_pool).await?;
        return Ok(logs);
    }
    let joined_filter = filter
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let logs: Vec<Log> = sqlx::query_as(format!("SELECT id, game_id, created_at, updated_at, start_date, end_date, rating, notes, status, minutes_played FROM logs WHERE status IN ({}) ORDER BY end_date DESC LIMIT $1", joined_filter).as_str()).bind(amount).fetch_all(&state.logs_pool).await?;
    Ok(logs)
}

#[tauri::command]
pub async fn get_logs(
    state: State<'_, DatabasePools>,
    sort_by: String,
    sort_order: String,
    filter: Vec<String>,
) -> Result<Vec<Log>, Error> {
    let joined_filter = filter
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(",");
    let logs: Vec<Log> = sqlx::query_as(format!("SELECT id, game_id, created_at, updated_at, start_date, end_date, rating, notes, status, minutes_played FROM logs WHERE status IN ({}) ORDER BY $1 {}", joined_filter, sort_order).as_str()).bind(sort_by).fetch_all(&state.logs_pool).await?;
    Ok(logs)
}

#[tauri::command]
pub async fn delete_log(state: State<'_, DatabasePools>, id: i32) -> Result<i32, Error> {
    sqlx::query("DELETE FROM logs WHERE id = $1").bind(id).execute(&state.logs_pool).await?;
    Ok(id)
}

#[tauri::command]
pub async fn get_log_by_id(state: State<'_, DatabasePools>, id: i32) -> Result<Log, Error> {
    let log: Log = sqlx::query_as("SELECT id, game_id, created_at, updated_at, start_date, end_date, rating, notes, status, minutes_played FROM logs WHERE logs.id = $1").bind(id).fetch_one(&state.logs_pool).await?;
    Ok(log)
}

#[tauri::command]
pub async fn add_log(state: State<'_, DatabasePools>, log_data: LogData) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar("INSERT INTO logs (game_id, start_date, rating, notes, status, minutes_played, end_date) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id AS id").bind(log_data.game_id).bind(log_data.start_date).bind(log_data.rating).bind(log_data.notes).bind(log_data.status).bind(log_data.minutes_played).bind(log_data.end_date).fetch_one(&state.logs_pool).await?;
    Ok(id)
}

#[tauri::command]
pub async fn update_log(
    state: State<'_, DatabasePools>,
    log_data: LogUpdateData,
) -> Result<i32, Error> {
    sqlx::query("UPDATE logs SET start_date = $1, end_date = $7, rating = $2, notes = $3, status = $4, minutes_played = $5 WHERE id = $6").bind(log_data.start_date).bind(log_data.rating).bind(log_data.notes).bind(log_data.status).bind(log_data.minutes_played).bind(log_data.id).bind(log_data.end_date).execute(&state.logs_pool).await?;
    Ok(log_data.id)
}

#[tauri::command]
pub async fn add_executable_details(
    state: State<'_, DatabasePools>,
    executable_details: ExecutableDetails,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar("INSERT INTO executable_details (executable_name, game_id) VALUES ($1, $2) RETURNING id AS id").bind(executable_details.name).bind(executable_details.game_id).fetch_one(&state.logs_pool).await?;
    Ok(id)
}
