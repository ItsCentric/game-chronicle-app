#![allow(dead_code)]
use anyhow::{Context, Result};
use csv::ReaderBuilder;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::{collections::HashMap, io::Cursor};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager};

use crate::helpers::create_dir_if_not_exists;

/// Field configuration with optional aliasing
#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub igdb_field: String,
    pub field_alias: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ImportError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub enum ImportStep {
    Download,
    Import,
}

#[derive(Debug, serde::Serialize, Clone)]
pub enum ImportStatus {
    Started,
    Progress,
    Completed,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ImportProgressPayload {
    pub step: ImportStep,
    pub status: ImportStatus,
    pub progress: Option<usize>, // total items processed
    pub total: Option<usize>,  // total items to process
}

impl From<anyhow::Error> for ImportError {
    fn from(err: anyhow::Error) -> Self {
        ImportError {
            message: err.to_string(),
            code: None,
        }
    }
}

impl FieldConfig {
    pub fn new(igdb_field: impl Into<String>) -> Self {
        Self {
            igdb_field: igdb_field.into(),
            field_alias: None,
        }
    }

    pub fn aliased(igdb_field: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            igdb_field: igdb_field.into(),
            field_alias: Some(alias.into()),
        }
    }

    pub fn get_db_field_name(&self) -> &str {
        self.field_alias.as_ref().unwrap_or(&self.igdb_field)
    }
}

impl From<&str> for FieldConfig {
    fn from(field: &str) -> Self {
        Self::new(field)
    }
}

impl From<String> for FieldConfig {
    fn from(field: String) -> Self {
        Self::new(field)
    }
}

impl From<(&str, &str)> for FieldConfig {
    fn from((igdb_field, alias): (&str, &str)) -> Self {
        Self::aliased(igdb_field, alias)
    }
}

impl From<(String, String)> for FieldConfig {
    fn from((igdb_field, alias): (String, String)) -> Self {
        Self::aliased(igdb_field, alias)
    }
}

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct CsvUrlResponse {
    pub url: String,
    pub version: String,
}

/// Configuration for a single dump import
#[derive(Debug, Clone)]
pub struct DumpConfig {
    pub endpoint: String,
    pub fields: Vec<FieldConfig>,
    pub table_name: Option<String>,
    pub primary_key: Option<String>,
    pub relation_type: RelationType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    /// Regular entity table (games, platforms, etc.)
    Entity,
    /// One-to-many embedded as array in CSV (e.g., platforms: [1,2,3])
    /// Fields: (parent_id_field, array_field_name, child_table, child_id_field)
    EmbeddedArray {
        parent_id_field: String,
        array_field_name: String,
        join_table_name: String,
        foreign_key_1: String, // parent table foreign key
        foreign_key_2: String, // child table foreign key
    },
}

impl DumpConfig {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            fields: Vec::new(),
            table_name: None,
            primary_key: None,
            relation_type: RelationType::Entity,
        }
    }

    /// Configure as embedded array relationship (e.g., games CSV with platforms: [1,2,3])
    pub fn embedded_relation(
        mut self,
        parent_id_field: impl Into<String>,
        array_field_name: impl Into<String>,
        join_table_name: impl Into<String>,
        foreign_key_1: impl Into<String>,
        foreign_key_2: impl Into<String>,
    ) -> Self {
        self.relation_type = RelationType::EmbeddedArray {
            parent_id_field: parent_id_field.into(),
            array_field_name: array_field_name.into(),
            join_table_name: join_table_name.into(),
            foreign_key_1: foreign_key_1.into(),
            foreign_key_2: foreign_key_2.into(),
        };
        self
    }

    /// Select specific fields to import with optional aliases
    pub fn fields<I, F>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: Into<FieldConfig>,
    {
        self.fields = fields.into_iter().map(|f| f.into()).collect();
        self
    }

    /// Add a single field with optional alias
    pub fn add_field<F: Into<FieldConfig>>(mut self, field: F) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Add multiple fields at once
    pub fn add_fields<I, F>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: Into<FieldConfig>,
    {
        self.fields.extend(fields.into_iter().map(|f| f.into()));
        self
    }

    /// Set custom table name (defaults to endpoint name)
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = Some(name.into());
        self
    }

    /// Set primary key field (defaults to "id")
    pub fn primary_key(mut self, key: impl Into<String>) -> Self {
        self.primary_key = Some(key.into());
        self
    }

    fn get_table_name(&self) -> &str {
        self.table_name.as_ref().unwrap_or(&self.endpoint)
    }

    fn get_primary_key(&self) -> &str {
        self.primary_key
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("id")
    }
}

/// Main IGDB dump importer
pub struct IgdbImporter {
    pool: SqlitePool,
    dumps: Vec<DumpConfig>,
    base_url: String,
    download_dir: String,
    app_handle: AppHandle,
}

impl IgdbImporter {
    /// Create a new importer with database connection
    pub async fn new(
        pool: SqlitePool,
        download_dir: String,
        app_handle: AppHandle,
    ) -> Result<Self> {
        Ok(Self {
            pool,
            dumps: Vec::new(),
            base_url: "https://api.gamechronicle.app/csv".to_string(),
            download_dir,
            app_handle,
        })
    }

    /// Set custom base URL for IGDB dumps
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Add a dump configuration - simple version with field selection
    pub fn add_dump<I, F>(&mut self, endpoint: &str, fields: I) -> &mut Self
    where
        I: IntoIterator<Item = F>,
        F: Into<FieldConfig>,
    {
        let config = DumpConfig::new(endpoint).fields(fields);
        self.dumps.push(config);
        self
    }

    /// Add an embedded array relationship (e.g., games CSV with platforms column containing [1,2,3])
    pub fn add_embedded_relationship(
        &mut self,
        endpoint: &str,
        parent_id_field: &str,
        array_field_name: &str,
        join_table_name: &str,
        foreign_key_1: &str,
        foreign_key_2: &str,
    ) -> &mut Self {
        let config = DumpConfig::new(endpoint).embedded_relation(
            parent_id_field,
            array_field_name,
            join_table_name,
            foreign_key_1,
            foreign_key_2,
        );
        self.dumps.push(config);
        self
    }

    /// Add a single field to the last dump configuration
    pub fn add_field<F: Into<FieldConfig>>(&mut self, field: F) -> &mut Self {
        if let Some(last_dump) = self.dumps.last_mut() {
            last_dump.fields.push(field.into());
        }
        self
    }

    /// Add a dump configuration - advanced version with builder pattern
    pub fn add_dump_config(&mut self, config: DumpConfig) -> &mut Self {
        self.dumps.push(config);
        self
    }

    /// Download all configured dumps
    pub async fn download_dumps(&self) -> Result<()> {
        // Create download directory
        create_dir_if_not_exists(Path::new(&self.download_dir))
            .context("Failed to create download directory")?;

        let mut join_set = tokio::task::JoinSet::new();

        for dump in &self.dumps {
            if dump.relation_type != RelationType::Entity {
                continue;
            }
            let url = format!("{}/{}", self.base_url, dump.endpoint);
            let file_path = format!("{}/{}.csv", self.download_dir, dump.endpoint);
            let endpoint = dump.endpoint.clone();

            join_set.spawn(async move {
                println!("Downloading {} dump...", endpoint);
                Self::download_file(&url, &file_path)
                    .await
                    .with_context(|| format!("Failed to download {} dump", endpoint))
            });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(())) => 
                    self.app_handle.emit(
                        "import_progress",
                        ImportProgressPayload {
                            step: ImportStep::Download,
                            status: ImportStatus::Progress,
                            progress: Some(1), // Increment progress for each successful download
                            total: None,
                        },
                    )?,
                Ok(Err(e)) => return Err(e),
                Err(join_err) => return Err(anyhow::anyhow!(join_err)),
            }
        }

        println!("All dumps downloaded successfully.");
        Ok(())
    }

    /// Import all downloaded dumps to database
    pub async fn import_dumps(&self) -> Result<()> {
        for dump in &self.dumps {
            let file_path = format!("{}/{}.csv", self.download_dir, dump.endpoint);

            if !Path::new(&file_path).exists() {
                println!("Warning: {} not found, skipping...", file_path);
                continue;
            }

            println!("Importing {} dump...", dump.endpoint);
            self.import_dump(dump, &file_path).await?;
        }
        println!("All dumps imported successfully.");

        Ok(())
    }

    /// Download and import in one step
    pub async fn run(&self, app_handle: &AppHandle) -> Result<()> {
        app_handle.emit(
            "import_progress",
            &ImportProgressPayload {
                step: ImportStep::Download,
                status: ImportStatus::Started,
                progress: None,
                total: Some(self.dumps.len()),
            },
        )?;
        self.download_dumps().await?;
        app_handle.emit(
            "import_progress",
            &ImportProgressPayload {
                step: ImportStep::Download,
                status: ImportStatus::Completed,
                progress: None,
                total: None,
            },
        )?;
        app_handle.emit(
            "import_progress",
            &ImportProgressPayload {
                step: ImportStep::Import,
                status: ImportStatus::Started,
                progress: None,
                total: None,
            },
        )?;
        self.import_dumps().await?;
        app_handle.emit(
            "import_progress",
            &ImportProgressPayload {
                step: ImportStep::Import,
                status: ImportStatus::Completed,
                progress: None,
                total: None,
            },
        )?;
        Ok(())
    }

    async fn download_file(url: &str, file_path: &str) -> Result<()> {
        let url_response = reqwest::get(url).await?.json::<CsvUrlResponse>().await?;
        let csv_response = reqwest::get(&url_response.url).await?;
        let content = csv_response.bytes().await?;
        fs::write(file_path, content)?;
        Ok(())
    }

    async fn import_dump(&self, config: &DumpConfig, file_path: &str) -> Result<()> {
        match &config.relation_type {
            RelationType::Entity => {
                self.import_entity_dump(config, file_path).await?;
            }
            RelationType::EmbeddedArray { .. } => {
                self.import_embedded_array_dump(config, file_path).await?;
            }
        }

        Ok(())
    }

    fn normalize_csv_value(value: &str) -> Option<&str> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn extract_row_data_with_aliases(
        &self,
        headers: &csv::StringRecord,
        record: &csv::StringRecord,
        fields: &[FieldConfig],
    ) -> Result<HashMap<String, String>> {
        let mut row_data = HashMap::new();

        for field_config in fields {
            if let Some(pos) = headers.iter().position(|h| h == &field_config.igdb_field) {
                let value = record.get(pos).unwrap_or("").to_string();
                let db_field_name = field_config.get_db_field_name().to_string();
                row_data.insert(db_field_name, value);
            }
        }

        Ok(row_data)
    }

    async fn import_entity_dump(&self, config: &DumpConfig, file_path: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut reader = ReaderBuilder::new()
            .buffer_capacity(8 * 1024 * 1024)
            .from_reader(Cursor::new(content.as_bytes())); // 8MB buffer for large files

        let headers = reader.headers()?.clone();

        // Validate requested fields exist and build mapping
        let mut valid_fields = Vec::new();
        let mut field_indices = Vec::new();

        for field_config in &config.fields {
            if let Some(index) = headers.iter().position(|h| h == &field_config.igdb_field) {
                valid_fields.push(field_config.clone());
                field_indices.push(index);
            } else {
                println!(
                    "Warning: Field '{}' not found in {} dump",
                    field_config.igdb_field, config.endpoint
                );
            }
        }

        if valid_fields.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid fields found for {}",
                config.endpoint
            ));
        }

        // Import data
        self.entity_batched_insert(config, &valid_fields, &field_indices, reader)
            .await?;
        Ok(())
    }

    async fn embedded_batched_insert(
        &self,
        table_name: &str,
        foreign_key_1: &str,
        foreign_key_2: &str,
        batch_values: &[(String, String)],
    ) -> Result<()> {
        if batch_values.is_empty() {
            return Ok(());
        }

        let placeholders = vec!["(?, ?)"; batch_values.len()].join(", ");
        let sql = format!(
            "INSERT OR IGNORE INTO {} ({}, {}) VALUES {}",
            table_name, foreign_key_1, foreign_key_2, placeholders
        );

        let mut query = sqlx::query(&sql);
        for (parent_id, child_id) in batch_values {
            query = query.bind(parent_id).bind(child_id);
        }

        query.execute(&self.pool).await?;
        Ok(())
    }

    /// Import embedded array relationships (e.g., games with platforms: [1,2,3])
    async fn import_embedded_array_dump(&self, config: &DumpConfig, file_path: &str) -> Result<()> {
        let RelationType::EmbeddedArray {
            parent_id_field,
            array_field_name,
            join_table_name,
            foreign_key_1,
            foreign_key_2,
        } = &config.relation_type
        else {
            return Err(anyhow::anyhow!(
                "Invalid relation type for embedded array import"
            ));
        };

        let content = fs::read_to_string(file_path)?;
        let mut reader = ReaderBuilder::new()
            .buffer_capacity(8 * 1024 * 1024)
            .from_reader(Cursor::new(content.as_bytes()));
        let beginning_pos = reader.position().clone();
        let total_records = reader.records().count();
        reader.records().reader_mut().seek(beginning_pos)?;
        let headers = reader.headers()?.clone();

        // Find column indices
        let parent_id_idx = headers
            .iter()
            .position(|h| h == parent_id_field)
            .ok_or_else(|| anyhow::anyhow!("Parent ID field '{}' not found", parent_id_field))?;

        let array_field_idx = headers
            .iter()
            .position(|h| h == array_field_name)
            .ok_or_else(|| anyhow::anyhow!("Array field '{}' not found", array_field_name))?;

        // Create join table
        self.create_join_table(join_table_name, foreign_key_1, foreign_key_2)
            .await?;

        const BATCH_SIZE: usize = 900; // Smaller batches due to multiple inserts per record
        let mut batch_values = Vec::with_capacity(BATCH_SIZE);
        let mut imported_count = 0;

        for result in reader.records() {
            let record = result?;

            let parent_id = record.get(parent_id_idx).unwrap_or("");
            let array_data = record.get(array_field_idx).unwrap_or("");

            if parent_id.is_empty() || array_data.is_empty() {
                continue;
            }

            // Parse array data - handle different formats
            let child_ids = self.parse_array_field(array_data)?;

            for child_id in child_ids {
                batch_values.push((parent_id.to_string(), child_id));
                if batch_values.len() >= BATCH_SIZE {
                    self.embedded_batched_insert(
                        join_table_name,
                        foreign_key_1,
                        foreign_key_2,
                        &batch_values,
                    )
                    .await?;
                    imported_count += batch_values.len();
                self.app_handle.emit(
                    "import_progress",
                    ImportProgressPayload {
                        step: ImportStep::Import,
                        status: ImportStatus::Progress,
                        progress: Some(imported_count),
                        total: Some(total_records * 10), // Estimate 20 relations per record
                    },
                )?;
                    println!(
                        "Inserted {} relationships into for {} ({} out of {})",
                        batch_values.len(),
                        config.endpoint,
                        imported_count,
                        total_records * 5
                    );
                    batch_values.clear();
                }
            }
        }

        if !batch_values.is_empty() {
            self.embedded_batched_insert(
                join_table_name,
                foreign_key_1,
                foreign_key_2,
                &batch_values,
            )
            .await?;
            imported_count += batch_values.len();
            println!(
                "Inserted remaining {} relationships into {}",
                batch_values.len(),
                config.endpoint
            );
        }

        println!(
            "Successfully imported {} relationship records for {}",
            imported_count, config.endpoint
        );
        Ok(())
    }

    /// Create a join table for relations
    async fn create_join_table(
        &self,
        table_name: &str,
        foreign_key_1: &str,
        foreign_key_2: &str,
    ) -> Result<()> {
        let create_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} INTEGER NOT NULL,
                {} INTEGER NOT NULL,
                PRIMARY KEY ({}, {}),
                FOREIGN KEY ({}) REFERENCES games(id) ON DELETE CASCADE,
                FOREIGN KEY ({}) REFERENCES platforms(id) ON DELETE CASCADE
            )",
            table_name,
            foreign_key_1,
            foreign_key_2,
            foreign_key_1,
            foreign_key_2,
            foreign_key_1,
            foreign_key_2
        );

        sqlx::query(&create_sql).execute(&self.pool).await?;
        Ok(())
    }

    /// Parse array field from CSV (handles [1,2,3] or "1,2,3" or JSON array)
    fn parse_array_field(&self, array_data: &str) -> Result<Vec<String>> {
        let trimmed = array_data.trim();

        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        // Handle CSV array format: {1,2,3}
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.is_empty() {
                return Ok(Vec::new());
            }
            return Ok(inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect());
        }

        // Handle JSON array with Value parsing
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(array) = json_value.as_array() {
                return Ok(array
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n.to_string()))
                    .collect());
            }
        }

        // Handle comma-separated values: "1,2,3"
        Ok(trimmed
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn execute_batch_insert(
        &self,
        table_name: &str,
        field_names: &[String],
        batch_values: &[Vec<Option<String>>],
        single_row_placeholders: &str,
    ) -> Result<()> {
        if batch_values.is_empty() {
            return Ok(());
        }

        let placeholders = vec![single_row_placeholders; batch_values.len()].join(", ");
        let sql = format!(
            "INSERT OR REPLACE INTO {} ({}) VALUES {}",
            table_name,
            field_names.join(", "),
            placeholders
        );

        let mut query = sqlx::query(&sql);
        for row in batch_values {
            for value in row {
                query = query.bind(value.as_deref());
            }
        }

        query.execute(&self.pool).await?;
        Ok(())
    }

    /// Batched import for entity relations
    async fn entity_batched_insert(
        &self,
        config: &DumpConfig,
        fields: &[FieldConfig],
        field_indices: &[usize],
        mut reader: csv::Reader<Cursor<&[u8]>>,
    ) -> Result<()> {
        let table_name = config.get_table_name();

        let db_field_names: Vec<String> = fields
            .iter()
            .map(|f| f.get_db_field_name().to_string())
            .collect();

        // Batch insert into temp table
        const BATCH_SIZE: usize = 900; // Larger batches for temp table
        let mut batch_values = Vec::with_capacity(BATCH_SIZE);
        let mut imported_count = 0;
        let single_row_placeholders = format!("({})", vec!["?"; fields.len()].join(","));
        let beginning_pos = reader.position().clone();
        let total_records = reader.records().count();
        reader.records().reader_mut().seek(beginning_pos)?;

        for result in reader.records() {
            let record = result?;

            let mut row_values = Vec::with_capacity(fields.len());
            for &index in field_indices {
                let raw_value = record.get(index).unwrap_or("");
                let normalized_value = Self::normalize_csv_value(raw_value);
                row_values.push(normalized_value.map(|s| s.to_string()));
            }

            batch_values.push(row_values);

            if batch_values.len() >= BATCH_SIZE {
                self.execute_batch_insert(
                    table_name,
                    &db_field_names,
                    &batch_values,
                    &single_row_placeholders,
                )
                .await?;
                imported_count += batch_values.len();
                self.app_handle.emit(
                    "import_progress",
                    ImportProgressPayload {
                        step: ImportStep::Import,
                        status: ImportStatus::Progress,
                        progress: Some(imported_count),
                        total: Some(total_records),
                    },
                )?;
                println!(
                    "Batch inserted {} records into {} ({} total records)",
                    batch_values.len(),
                    config.endpoint,
                    imported_count
                );
                batch_values.clear();
            }
        }

        if !batch_values.is_empty() {
            self.execute_batch_insert(
                table_name,
                &db_field_names,
                &batch_values,
                &single_row_placeholders,
            )
            .await?;
            imported_count += batch_values.len();
            println!(
                "Inserted remaining {} records into {}",
                batch_values.len(),
                config.endpoint
            );
        }

        println!(
            "Finished batch inserting {} records for {}",
            imported_count, config.endpoint
        );
        Ok(())
    }
}

/// Macro for creating field configurations with clean syntax
#[macro_export]
macro_rules! fields {
    // Handle mixed field types: fields!["id", name => title, "summary", rating => score]
    ($($field:expr),* $(,)?) => {
        vec![$($field.into()),*]
    };
}

/// Macro for creating aliased fields with arrow syntax
#[macro_export]
macro_rules! alias {
    ($igdb_field:expr => $alias:expr) => {
        FieldConfig::aliased($igdb_field, $alias)
    };
}

/// Helper trait for more intuitive field building
pub trait FieldExt {
    fn as_field(self) -> FieldConfig;
    fn aliased_as(self, alias: impl Into<String>) -> FieldConfig;
}

impl FieldExt for &str {
    fn as_field(self) -> FieldConfig {
        FieldConfig::new(self)
    }

    fn aliased_as(self, alias: impl Into<String>) -> FieldConfig {
        FieldConfig::aliased(self, alias)
    }
}

impl FieldExt for String {
    fn as_field(self) -> FieldConfig {
        FieldConfig::new(self)
    }

    fn aliased_as(self, alias: impl Into<String>) -> FieldConfig {
        FieldConfig::aliased(self, alias)
    }
}

#[tauri::command]
pub async fn import_igdb_dumps(app_handle: AppHandle) -> Result<(), ImportError> {
    let db_path = app_handle
        .path()
        .app_data_dir()
        .context("Could not get data directory")?
        .join("igdb.db");

    let mut importer = IgdbImporter::new(
        match sqlx::SqlitePool::connect_with(
            SqliteConnectOptions::from_str(&db_path.to_string_lossy().to_string())
                .context("Could not connect to igdb database for import")?
                .foreign_keys(false)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Off)
                .page_size(65536) // 64KB page size
                .pragma("cache_size", "-262144") // 256MB cache size
                .pragma("temp_store", "MEMORY")
                .pragma("mmap_size", "1073741824"), // 1GB mmap size
        )
        .await
        {
            Ok(pool) => pool,
            Err(e) => {
                println!("Failed to connect to database: {}", e);
                return Err(anyhow::anyhow!(e).into());
            }
        },
        app_handle
            .path()
            .temp_dir()
            .context("Could not get temp directory for dumps")?
            .to_string_lossy()
            .to_string(),
        app_handle.clone(),
    )
    .await
    .context("Failed to create IGDB importer")?;

    importer
        .add_dump("platforms", ["id", "name", "category"])
        .add_dump("websites", ["id", "url"])
        .add_dump("covers", ["id", "image_id"])
        .add_dump(
            "games",
            [
                "id".as_field(),
                "name".as_field(),
                "cover".aliased_as("cover_id"),
                "category".as_field(),
                "version_parent".as_field(),
                "total_rating".as_field(),
            ],
        )
        .add_dump(
            "popularity_primitives",
            ["id", "game_id", "popularity_type", "value"],
        )
        .add_embedded_relationship(
            "games",
            "id",
            "platforms",
            "game_platforms",
            "game_id",
            "platform_id",
        )
        .add_embedded_relationship(
            "games",
            "id",
            "websites",
            "game_websites",
            "game_id",
            "website_id",
        )
        .add_embedded_relationship(
            "games",
            "id",
            "similar_games",
            "similar_games",
            "game_id",
            "similar_game_id",
        )
        .run(&app_handle)
        .await?;
    app_handle
        .emit("import_finished", ())
        .context("Failed to emit import finished event")?;
    Ok(())
}
