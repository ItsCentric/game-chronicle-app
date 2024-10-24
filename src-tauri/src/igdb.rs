use tauri::State;

use crate::{DatabaseConnections, Error};

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Game {
    pub id: i32,
    pub name: String,
    #[serde(rename(deserialize = "cover"))]
    pub cover_id: Option<i32>,
    #[serde(
        rename(deserialize = "websites"),
        deserialize_with = "deserialize_list"
    )]
    pub website_ids: Option<Vec<i32>>,
    #[serde(deserialize_with = "deserialize_list")]
    pub similar_games: Option<Vec<i32>>,
    pub category: i32,
    pub version_parent: Option<i32>,
    pub total_rating: Option<f32>,
    #[serde(
        rename(deserialize = "platforms"),
        deserialize_with = "deserialize_list"
    )]
    pub platform_ids: Option<Vec<i32>>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cover {
    pub id: i32,
    pub image_id: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Website {
    pub id: i32,
    pub url: String,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Platform {
    pub id: i32,
    pub name: String,
    pub category: Option<i32>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, Clone)]
pub struct GameInfo {
    pub id: i32,
    pub title: String,
    pub cover_image_id: Option<String>,
    pub websites: Option<Vec<String>>,
    pub similar_games: Option<Vec<i32>>,
    pub category: i32,
    pub version_parent: Option<i32>,
    pub total_rating: Option<f32>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct PopularityPrimitive {
    pub id: i32,
    pub game_id: i32,
    pub popularity_type: i32,
    pub value: f32,
}

fn deserialize_list<'de, D>(deserializer: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    let s = s.trim_start_matches('{').trim_end_matches('}');
    if s.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        s.split(',')
            .map(|item| match item.trim().parse::<i32>() {
                Ok(id) => id,
                Err(_) => 0,
            })
            .collect(),
    ))
}

fn game_info_from_row(row: &rusqlite::Row) -> Result<GameInfo, rusqlite::Error> {
    let websites_string: Option<String> = row.get("websites")?;
    let similar_games_string: Option<String> = row.get("similar_game_ids")?;
    let websites = match websites_string {
        Some(string) => Some(string.split(',').map(|s| s.to_string()).collect()),
        None => None,
    };
    let similar_games = match similar_games_string {
        Some(string) => Some(
            string
                .split(',')
                .map(|s| s.parse::<i32>().unwrap())
                .collect(),
        ),
        None => None,
    };
    Ok(GameInfo {
        id: row.get("id")?,
        title: row.get("name")?,
        cover_image_id: row.get("image_id")?,
        websites,
        similar_games,
        category: row.get("category")?,
        version_parent: row.get("version_parent")?,
        total_rating: row.get("total_rating")?,
    })
}

fn game_info_columns() -> &'static str {
    "g.id, g.name, c.image_id, GROUP_CONCAT(w.url, ',') websites, GROUP_CONCAT(sg.similar_game_id, ',') similar_game_ids, g.category, g.version_parent, total_rating FROM games g LEFT JOIN covers c ON g.cover_id = c.id LEFT JOIN game_websites gw ON g.id = gw.game_id LEFT JOIN websites w ON gw.website_id = w.id LEFT JOIN similar_games sg ON sg.game_id = g.id LEFT JOIN game_platforms gp ON g.id = gp.game_id LEFT JOIN platforms p ON p.id = gp.platform_id LEFT JOIN popularity_primitives pp ON g.id = pp.game_id"
}

#[tauri::command]
pub fn get_games_by_id(
    state: State<'_, DatabaseConnections>,
    game_ids: Vec<i32>,
) -> Result<Vec<GameInfo>, Error> {
    if game_ids.is_empty() {
        return Ok(vec![]);
    }
    let conn = state.igdb_conn.lock().unwrap();
    let query = format!(
        "SELECT {} WHERE g.id IN ({}) AND g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id;",
        game_info_columns(), game_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let mut stmt = conn.prepare(&query)?;
    let games = stmt
        .query_map([], game_info_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}

#[tauri::command]
pub async fn get_popular_games(
    state: State<'_, DatabaseConnections>,
    amount: i32,
) -> Result<Vec<GameInfo>, Error> {
    let conn = state.igdb_conn.lock().unwrap();
    let games = conn.prepare(&format!("SELECT {} WHERE g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id ORDER BY pp.value DESC LIMIT ?;", game_info_columns()).to_string())?
        .query_map([amount], game_info_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}

#[tauri::command]
pub fn search_game(
    state: State<'_, DatabaseConnections>,
    search_query: String,
) -> Result<Vec<GameInfo>, Error> {
    let results: Vec<i32>;
    {
        let conn = state.igdb_conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT rowid FROM games_fts WHERE name MATCH ?;")?;
        results = stmt
            .query_map([search_query.replace("'", " ")], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
    }
    let games = get_games_by_id(state, results)?;
    Ok(games)
}

pub fn get_games_from_links(
    state: State<'_, DatabaseConnections>,
    links: Vec<String>,
) -> Result<Vec<GameInfo>, Error> {
    let conn = state.igdb_conn.lock().unwrap();
    let formatted_links = links
        .iter()
        .map(|l| format!("'{}'", l))
        .collect::<Vec<String>>();
    let mut stmt = conn.prepare(
        format!(
            "SELECT {} WHERE w.url IN ({}) AND g.category IN (0, 4, 8, 9) AND p.name NOT IN ('Android', 'iOS') AND g.version_parent IS NULL GROUP BY g.id;",
            game_info_columns(),
            formatted_links.join(",").as_str()
        )
        .as_str(),
    )?;
    let games = stmt
        .query_map([], game_info_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}
