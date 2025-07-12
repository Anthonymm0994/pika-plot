//! CSV import functionality.

use std::path::Path;
use std::sync::Arc;
use csv::ReaderBuilder;
use pika_core::{
    error::{PikaError, Result},
    events::{Event, EventBus, AppEvent},
    types::{NodeId, TableInfo, ColumnInfo, ImportOptions},
};

/// Import a CSV file into the database
pub async fn import_csv(
    path: &Path,
    node_id: &NodeId,
    options: &ImportOptions,
    event_bus: Option<Arc<EventBus>>,
) -> Result<TableInfo> {
    // Send progress events if event bus provided
    let progress_callback = |progress: f32| {
        if let Some(sender) = &event_bus {
            sender.send(Event::App(AppEvent::FileOpened(
                format!("Importing CSV: {:.0}%", progress * 100.0)
            )));
        }
    };
    
    // Report errors via event bus
    let error_callback = |error: String| {
        if let Some(sender) = &event_bus {
            sender.send(Event::App(AppEvent::FileOpened(
                format!("Import error: {}", error)
            )));
        }
    };
    
    // Detect column types from CSV
    let columns = detect_column_types(path, options)?;
    progress_callback(0.2);
    
    // Create table name from file name
    let table_name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported_table")
        .to_string();
    
    // Import data into DuckDB
    import_csv_to_duckdb(path, &table_name, &columns, options).await?;
    progress_callback(0.8);
    
    // Get row count
    let row_count = get_row_count(&table_name).await?;
    progress_callback(1.0);
    
    Ok(TableInfo {
        name: table_name,
        source_path: Some(path.to_path_buf()),
        row_count: Some(row_count),
        columns,
    })
}

/// Column type detection
#[derive(Debug)]
struct TypeInference {
    headers: Vec<String>,
    column_types: Vec<ColumnType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Boolean,
    Integer,
    Float,
    Date,
    Text,
}

impl TypeInference {
    fn new(headers: &[String]) -> Self {
        TypeInference {
            headers: headers.to_vec(),
            column_types: vec![ColumnType::Integer; headers.len()], // Start optimistic
        }
    }
    
    fn update(&mut self, record: &csv::StringRecord) {
        for (i, field) in record.iter().enumerate() {
            if i >= self.column_types.len() {
                continue;
            }
            
            // Skip if already determined to be text
            if self.column_types[i] == ColumnType::Text {
                continue;
            }
            
            // Try parsing in order of strictness
            if field.is_empty() {
                continue; // NULL value, don't change type
            }
            
            match self.column_types[i] {
                ColumnType::Boolean => {
                    if !is_boolean(field) {
                        self.column_types[i] = ColumnType::Integer;
                        self.update_single(i, field);
                    }
                }
                ColumnType::Integer => {
                    if !field.parse::<i64>().is_ok() {
                        self.column_types[i] = ColumnType::Float;
                        self.update_single(i, field);
                    }
                }
                ColumnType::Float => {
                    if !field.parse::<f64>().is_ok() {
                        self.column_types[i] = ColumnType::Date;
                        self.update_single(i, field);
                    }
                }
                ColumnType::Date => {
                    if !is_date(field) {
                        self.column_types[i] = ColumnType::Text;
                    }
                }
                ColumnType::Text => {} // Terminal state
            }
        }
    }
    
    fn update_single(&mut self, index: usize, field: &str) {
        match self.column_types[index] {
            ColumnType::Float => {
                if !field.parse::<f64>().is_ok() {
                    self.column_types[index] = ColumnType::Date;
                    self.update_single(index, field);
                }
            }
            ColumnType::Date => {
                if !is_date(field) {
                    self.column_types[index] = ColumnType::Text;
                }
            }
            _ => {}
        }
    }
    
    fn to_sql_types(&self) -> Vec<String> {
        self.column_types.iter().map(|t| match t {
            ColumnType::Boolean => "BOOLEAN",
            ColumnType::Integer => "BIGINT",
            ColumnType::Float => "DOUBLE",
            ColumnType::Date => "DATE",
            ColumnType::Text => "VARCHAR",
        }.to_string()).collect()
    }
}

fn is_boolean(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "false" | "t" | "f" | "yes" | "no" | "y" | "n" | "1" | "0")
}

fn is_date(s: &str) -> bool {
    // Simple date detection - could be enhanced
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() ||
    chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y").is_ok() ||
    chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok()
}

/// Detect column types from a CSV file
pub fn detect_column_types(path: &Path, options: &ImportOptions) -> Result<Vec<ColumnInfo>> {
    let file = std::fs::File::open(path)
        .map_err(|e| PikaError::FileReadError(e.to_string()))?;
    
    let mut reader = ReaderBuilder::new()
        .delimiter(options.delimiter as u8)
        .has_headers(options.has_header)
        .from_reader(file);
    
    // Get headers
    let headers = if options.has_header {
        reader.headers()
            .map_err(|e| PikaError::CsvImport(format!("Failed to read headers: {}", e)))?
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        // Generate column names
        let record = reader.records().next()
            .ok_or_else(|| PikaError::CsvImport("Empty CSV file".to_string()))?
            .map_err(|e| PikaError::CsvImport(format!("Failed to read first row: {}", e)))?;
        (0..record.len()).map(|i| format!("column_{}", i + 1)).collect::<Vec<String>>()
    };
    
    // Reset reader for type inference
    let file = std::fs::File::open(path)
        .map_err(|e| PikaError::FileReadError(e.to_string()))?;
    let mut reader = ReaderBuilder::new()
        .delimiter(options.delimiter as u8)
        .has_headers(options.has_header)
        .from_reader(file);
    
    let mut type_inference = TypeInference::new(&headers);
    
    // Sample rows for type inference
    let sample_size = 1000;
    for (idx, result) in reader.records().skip(options.skip_rows).take(sample_size).enumerate() {
        let record = result.map_err(|e| PikaError::CsvImport(
            format!("Failed to parse CSV row {}: {}", idx + 1, e)
        ))?;
        
        if record.len() != headers.len() {
            return Err(PikaError::CsvImport(
                format!("Row {} has {} columns, expected {}", idx + 1, record.len(), headers.len())
            ));
        }
        
        type_inference.update(&record);
    }
    
    // Convert to ColumnInfo
    let sql_types = type_inference.to_sql_types();
    Ok(headers.into_iter().zip(sql_types).map(|(name, data_type)| {
        ColumnInfo {
            name,
            data_type,
            nullable: true, // Always nullable for CSV imports
        }
    }).collect())
}

/// Import CSV data into DuckDB
async fn import_csv_to_duckdb(
    path: &Path,
    table_name: &str,
    columns: &[ColumnInfo],
    options: &ImportOptions,
) -> Result<()> {
    // Build CREATE TABLE statement
    let column_defs: Vec<String> = columns.iter()
        .map(|col| format!("{} {}", col.name, col.data_type))
        .collect();
    
    let create_table = format!(
        "CREATE OR REPLACE TABLE {} ({})",
        table_name,
        column_defs.join(", ")
    );
    
    // Execute CREATE TABLE
    let db = crate::database::Database::new().await?;
    db.execute(&create_table).await?;
    
    // Build COPY statement
    let mut copy_options = Vec::new();
    copy_options.push(format!("DELIMITER '{}'", options.delimiter));
    
    if options.has_header {
        copy_options.push("HEADER".to_string());
    }
    
    if let Some(quote) = options.quote_char {
        copy_options.push(format!("QUOTE '{}'", quote));
    }
    
    if let Some(escape) = options.escape_char {
        copy_options.push(format!("ESCAPE '{}'", escape));
    }
    
    if options.skip_rows > 0 {
        copy_options.push(format!("SKIP {}", options.skip_rows));
    }
    
    let copy_statement = format!(
        "COPY {} FROM '{}' ({})",
        table_name,
        path.display(),
        copy_options.join(", ")
    );
    
    // Execute COPY
    db.execute(&copy_statement).await?;
    
    Ok(())
}

/// Get row count for a table
async fn get_row_count(table_name: &str) -> Result<usize> {
    let db = crate::database::Database::new().await?;
    let count_query = format!("SELECT COUNT(*) FROM {}", table_name);
    let count: i64 = db.query_scalar(&count_query).await?;
    Ok(count as usize)
}

/// Import Parquet file (placeholder)
pub async fn import_parquet(_path: &Path, _options: &ImportOptions) -> Result<TableInfo> {
    Err(PikaError::not_implemented("Parquet import"))
}

/// Import JSON file (placeholder)
pub async fn import_json(_path: &Path, _options: &ImportOptions) -> Result<TableInfo> {
    Err(PikaError::not_implemented("JSON import"))
} 