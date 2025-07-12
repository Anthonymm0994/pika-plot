//! CSV and data import functionality.

use std::path::Path;
use pika_core::{
    error::{PikaError, Result},
    types::{NodeId, ImportOptions, TableInfo, ColumnInfo},
    events::{AppEvent, EventBus},
};
use crate::database::Database;
use std::sync::Arc;

/// Import context for tracking progress and sending events
pub struct ImportContext {
    pub node_id: NodeId,
    pub path: std::path::PathBuf,
    pub options: ImportOptions,
    pub event_sender: Option<EventBus>,
}

impl ImportContext {
    /// Send progress update
    fn send_progress(&self, progress: f32) {
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send_app_event(AppEvent::ImportProgress {
                path: self.path.clone(),
                progress,
            });
        }
    }
    
    /// Send error event
    fn send_error(&self, error: PikaError) {
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send_app_event(AppEvent::ImportError {
                path: self.path.clone(),
                error: error.to_string(),
            });
        }
    }
}

/// Import a file into the database with enhanced validation and progress tracking.
pub async fn import_file(
    db: &Database,
    path: &Path,
    options: ImportOptions,
    event_sender: Option<EventBus>,
) -> Result<TableInfo> {
    // Validate file exists
    if !path.exists() {
        return Err(PikaError::FileNotFound {
            path: path.display().to_string()
        });
    }
    
    // Check file is not a directory
    if path.is_dir() {
        return Err(PikaError::FileReadError {
            error: format!("{} is a directory", path.display()),
        });
    }
    
    // Check file size
    let metadata = std::fs::metadata(path).map_err(|e| PikaError::FileReadError {
        error: e.to_string(),
    })?;
    let size_mb = metadata.len() / (1024 * 1024);
    if size_mb > 2048 {
        return Err(PikaError::FileTooLarge {
            path: path.display().to_string(),
            size: metadata.len(),
        });
    }
    
    let node_id = NodeId::new();
    let context = ImportContext {
        node_id,
        path: path.to_path_buf(),
        options: options.clone(),
        event_sender,
    };
    
    // Send import started event
    if let Some(sender) = &context.event_sender {
        let _ = sender.send_app_event(AppEvent::ImportStarted {
            path: path.to_path_buf(),
        });
    }
    
    // Determine import strategy based on file extension
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let table_name = if options.table_name.is_empty() {
        generate_table_name(&node_id)
    } else {
        sanitize_table_name(&options.table_name)
    };
    
    context.send_progress(0.1); // 10% - Started
    
    let table_info = match extension.to_lowercase().as_str() {
        "csv" => import_csv_enhanced(db, &context, &table_name).await?,
        "parquet" => import_parquet(db, &context, &table_name).await?,
        "json" | "jsonl" => import_json(db, &context, &table_name).await?,
        _ => return Err(PikaError::UnsupportedFormat { 
            format: extension.to_string() 
        }),
    };
    
    context.send_progress(1.0); // 100% - Complete
    
    // Send import complete event
    if let Some(sender) = &context.event_sender {
        let _ = sender.send_app_event(AppEvent::ImportComplete {
            path: path.to_path_buf(),
            table_info: table_info.clone(),
        });
    }
    
    Ok(table_info)
}

/// Enhanced CSV import with type inference and validation
async fn import_csv_enhanced(
    db: &Database,
    context: &ImportContext,
    table_name: &str,
) -> Result<TableInfo> {
    let path = &context.path;
    let options = &context.options;
    
    context.send_progress(0.2); // 20% - Reading file
    
    // First, get a preview and infer types if needed
    let preview_result = if options.infer_schema {
        preview_csv_with_inference(path, options)?
    } else {
        preview_csv_basic(path, options)?
    };
    
    context.send_progress(0.4); // 40% - Type inference complete
    
    // Build the CREATE TABLE statement with proper types
    let create_sql = build_create_table_sql(table_name, &preview_result, options)?;
    
    // Create table
    db.execute_sql(&create_sql)?;
    
    context.send_progress(0.5); // 50% - Table created
    
    // Build COPY statement for efficient bulk loading
    let copy_sql = build_copy_sql(table_name, path, options)?;
    
    // Import data
    let rows_imported = db.execute_sql(&copy_sql)?;
    
    context.send_progress(0.9); // 90% - Data imported
    
    // Get final table info
    let table_info = get_table_info(db, &context.node_id, table_name, path)?;
    
    Ok(table_info)
}

/// Preview CSV and infer column types
fn preview_csv_with_inference(
    path: &Path,
    options: &ImportOptions,
) -> Result<CsvPreviewResult> {
    use csv::ReaderBuilder;
    use std::fs::File;
    
    let file = std::fs::File::open(path).map_err(|e| PikaError::FileReadError {
        error: e.to_string(),
    })?;
    
    let mut reader = ReaderBuilder::new()
        .delimiter(options.delimiter)
        .has_headers(options.has_header)
        .quote(options.quote_char as u8)
        .escape(options.escape_char.map(|c| c as u8))
        .flexible(true)
        .from_reader(file);
    
    let headers = if options.has_header {
        reader.headers()
            .map_err(|e| PikaError::CsvImport {
                error: format!("Failed to read headers: {}", e),
                line: Some(0),
            })?
            .iter()
            .map(String::from)
            .collect()
    } else {
        // Generate column names
        let first_record = reader.records().next();
        if let Some(Ok(record)) = first_record {
            (0..record.len()).map(|i| format!("column_{}", i + 1)).collect()
        } else {
            vec![]
        }
    };
    
    // Infer types from sample
    let mut type_inference = TypeInference::new(&headers, &options.null_values);
    
    // Read sample for type inference
    let sample_size = options.sample_size.min(10000);
    let mut sample_rows = Vec::new();
    
    for (idx, result) in reader.records().take(sample_size).enumerate() {
        let record = result.map_err(|e| PikaError::CsvImport {
            error: format!("Failed to parse CSV row {}: {}", idx + 1, e),
            line: Some(idx + 1),
        })?;
        
        if record.len() != headers.len() {
            return Err(PikaError::CsvImport {
                error: format!("Row {} has {} columns, expected {}", idx + 1, record.len(), headers.len()),
                line: Some(idx + 1),
            });
        }
        
        let row: Vec<String> = record.iter().map(String::from).collect();
        type_inference.process_row(&row);
        
        if sample_rows.len() < 100 { // Keep first 100 for preview
            sample_rows.push(row);
        }
    }
    
    let column_types = type_inference.finalize();
    
    Ok(CsvPreviewResult {
        headers,
        column_types,
        sample_rows,
        total_rows_sampled: sample_size,
    })
}

/// Basic CSV preview without type inference
fn preview_csv_basic(
    path: &Path,
    options: &ImportOptions,
) -> Result<CsvPreviewResult> {
    // Use DuckDB's schema detection as fallback
    // This is simpler but less customizable than our inference
    todo!("Implement basic preview using DuckDB's read_csv")
}

/// Type inference engine for CSV columns
struct TypeInference {
    columns: Vec<ColumnTypeInfo>,
    null_values: Vec<String>,
}

#[derive(Clone)]
struct ColumnTypeInfo {
    name: String,
    null_count: usize,
    integer_count: usize,
    float_count: usize,
    date_count: usize,
    boolean_count: usize,
    text_count: usize,
    total_count: usize,
}

impl TypeInference {
    fn new(headers: &[String], null_values: &[String]) -> Self {
        let columns = headers.iter().map(|name| ColumnTypeInfo {
            name: name.clone(),
            null_count: 0,
            integer_count: 0,
            float_count: 0,
            date_count: 0,
            boolean_count: 0,
            text_count: 0,
            total_count: 0,
        }).collect();
        
        TypeInference {
            columns,
            null_values: null_values.to_vec(),
        }
    }
    
    fn process_row(&mut self, row: &[String]) {
        for (idx, value) in row.iter().enumerate() {
            if let Some(col_info) = self.columns.get_mut(idx) {
                col_info.total_count += 1;
                
                let trimmed = value.trim();
                
                // Check for null
                if trimmed.is_empty() || self.null_values.contains(&trimmed.to_string()) {
                    col_info.null_count += 1;
                    continue;
                }
                
                // Try parsing in order of specificity
                if Self::is_boolean(trimmed) {
                    col_info.boolean_count += 1;
                } else if Self::is_integer(trimmed) {
                    col_info.integer_count += 1;
                } else if Self::is_float(trimmed) {
                    col_info.float_count += 1;
                } else if Self::is_date(trimmed) {
                    col_info.date_count += 1;
                } else {
                    col_info.text_count += 1;
                }
            }
        }
    }
    
    fn finalize(self) -> Vec<InferredType> {
        self.columns.into_iter().map(|col| {
            let non_null = col.total_count - col.null_count;
            
            if non_null == 0 {
                InferredType::Text // Default for all nulls
            } else if col.boolean_count > non_null * 9 / 10 {
                InferredType::Boolean
            } else if col.integer_count > non_null * 9 / 10 {
                InferredType::Integer
            } else if col.integer_count + col.float_count > non_null * 9 / 10 {
                InferredType::Float
            } else if col.date_count > non_null * 9 / 10 {
                InferredType::Date
            } else {
                InferredType::Text
            }
        }).collect()
    }
    
    fn is_boolean(s: &str) -> bool {
        matches!(s.to_lowercase().as_str(), "true" | "false" | "t" | "f" | "yes" | "no" | "y" | "n" | "1" | "0")
    }
    
    fn is_integer(s: &str) -> bool {
        s.parse::<i64>().is_ok()
    }
    
    fn is_float(s: &str) -> bool {
        s.parse::<f64>().is_ok()
    }
    
    fn is_date(s: &str) -> bool {
        // Simple date patterns - can be enhanced
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() ||
        chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y").is_ok() ||
        chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok()
    }
}

#[derive(Debug, Clone)]
enum InferredType {
    Boolean,
    Integer,
    Float,
    Date,
    Text,
}

impl InferredType {
    fn to_sql_type(&self) -> &'static str {
        match self {
            InferredType::Boolean => "BOOLEAN",
            InferredType::Integer => "BIGINT",
            InferredType::Float => "DOUBLE",
            InferredType::Date => "DATE",
            InferredType::Text => "VARCHAR",
        }
    }
}

struct CsvPreviewResult {
    headers: Vec<String>,
    column_types: Vec<InferredType>,
    sample_rows: Vec<Vec<String>>,
    total_rows_sampled: usize,
}

/// Build CREATE TABLE SQL with inferred types
fn build_create_table_sql(
    table_name: &str,
    preview: &CsvPreviewResult,
    options: &ImportOptions,
) -> Result<String> {
    let mut columns = Vec::new();
    
    for (idx, header) in preview.headers.iter().enumerate() {
        let col_type = preview.column_types.get(idx)
            .map(|t| t.to_sql_type())
            .unwrap_or("VARCHAR");
        
        columns.push(format!("\"{}\" {}", sanitize_column_name(header), col_type));
    }
    
    Ok(format!(
        "CREATE TABLE {} ({})",
        table_name,
        columns.join(", ")
    ))
}

/// Build COPY SQL for efficient data loading
fn build_copy_sql(
    table_name: &str,
    path: &Path,
    options: &ImportOptions,
) -> Result<String> {
    let mut copy_options = Vec::new();
    
    copy_options.push(format!("FORMAT CSV"));
    copy_options.push(format!("DELIMITER '{}'", options.delimiter as char));
    copy_options.push(format!("QUOTE '{}'", options.quote_char));
    
    if options.has_header {
        copy_options.push("HEADER".to_string());
    }
    
    if let Some(escape) = options.escape_char {
        copy_options.push(format!("ESCAPE '{}'", escape));
    }
    
    if !options.null_values.is_empty() {
        let null_str = options.null_values.join(",");
        copy_options.push(format!("NULL '{}'", null_str));
    }
    
    Ok(format!(
        "COPY {} FROM '{}' ({})",
        table_name,
        path.display(),
        copy_options.join(", ")
    ))
}

/// Import Parquet file.
async fn import_parquet(
    db: &Database,
    context: &ImportContext,
    table_name: &str,
) -> Result<TableInfo> {
    context.send_progress(0.3);
    
    let sql = format!(
        "CREATE TABLE {} AS SELECT * FROM read_parquet('{}')",
        table_name,
        context.path.display()
    );
    
    db.execute_sql(&sql)?;
    
    context.send_progress(0.8);
    
    get_table_info(db, &context.node_id, table_name, &context.path)
}

/// Import JSON/JSONL file.
async fn import_json(
    db: &Database,
    context: &ImportContext,
    table_name: &str,
) -> Result<TableInfo> {
    context.send_progress(0.3);
    
    let sql = format!(
        "CREATE TABLE {} AS SELECT * FROM read_json_auto('{}')",
        table_name,
        context.path.display()
    );
    
    db.execute_sql(&sql)?;
    
    context.send_progress(0.8);
    
    get_table_info(db, &context.node_id, table_name, &context.path)
}

/// Get table information after import
fn get_table_info(
    db: &Database,
    node_id: &NodeId,
    table_name: &str,
    path: &Path,
) -> Result<TableInfo> {
    // Get row count
    let count_sql = format!("SELECT COUNT(*) FROM {}", table_name);
    let row_count = db.query_scalar::<i64>(&count_sql)? as usize;
    
    // Get column information
    let schema_sql = format!(
        "SELECT column_name, data_type 
         FROM information_schema.columns 
         WHERE table_name = '{}' 
         ORDER BY ordinal_position",
        table_name
    );
    
    let columns = db.query_map(&schema_sql, |row| {
        Ok(ColumnInfo {
            name: row.get(0)?,
            data_type: row.get(1)?,
        })
    })?;
    
    // Estimate size (rough)
    let estimated_size = estimate_table_size(row_count, &columns);
    
    Ok(TableInfo {
        id: *node_id,
        name: path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_data")
            .to_string(),
        table_name: table_name.to_string(),
        columns,
        row_count,
        estimated_size,
    })
}

/// Estimate table size in bytes
fn estimate_table_size(row_count: usize, columns: &[ColumnInfo]) -> u64 {
    let avg_bytes_per_column = 8; // Conservative estimate
    (row_count * columns.len() * avg_bytes_per_column) as u64
}

/// Generate a unique table name for imported data.
fn generate_table_name(node_id: &NodeId) -> String {
    format!("data_{}", node_id.0.as_simple())
}

/// Sanitize table name to be SQL-safe
fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

/// Sanitize column name to be SQL-safe
fn sanitize_column_name(name: &str) -> String {
    let sanitized = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>();
    
    // Ensure it doesn't start with a number
    if sanitized.chars().next().map_or(false, |c| c.is_numeric()) {
        format!("col_{}", sanitized)
    } else {
        sanitized
    }
}

/// Export table data to various formats
pub async fn export_table(
    db: &Database,
    table_name: &str,
    output_path: &Path,
    format: ExportFormat,
) -> Result<()> {
    let sql = match format {
        ExportFormat::Csv => {
            format!("COPY {} TO '{}' (FORMAT CSV, HEADER)", table_name, output_path.display())
        }
        ExportFormat::Parquet => {
            format!("COPY {} TO '{}' (FORMAT PARQUET)", table_name, output_path.display())
        }
        ExportFormat::Json => {
            format!("COPY {} TO '{}' (FORMAT JSON)", table_name, output_path.display())
        }
    };
    
    db.execute_sql(&sql)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Parquet,
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_table_name() {
        let node_id = NodeId::new();
        let table_name = generate_table_name(&node_id);
        assert!(table_name.starts_with("data_"));
    }
    
    #[test]
    fn test_sanitize_table_name() {
        assert_eq!(sanitize_table_name("My Table!"), "my_table_");
        assert_eq!(sanitize_table_name("123_start"), "123_start");
        assert_eq!(sanitize_table_name("váłîđ"), "v____");
    }
    
    #[test]
    fn test_sanitize_column_name() {
        assert_eq!(sanitize_column_name("My Column!"), "My_Column_");
        assert_eq!(sanitize_column_name("123_start"), "col_123_start");
        assert_eq!(sanitize_column_name("_valid"), "_valid");
    }
    
    #[test]
    fn test_type_inference_boolean() {
        assert!(TypeInference::is_boolean("true"));
        assert!(TypeInference::is_boolean("FALSE"));
        assert!(TypeInference::is_boolean("yes"));
        assert!(TypeInference::is_boolean("1"));
        assert!(!TypeInference::is_boolean("maybe"));
    }
    
    #[test]
    fn test_type_inference_numeric() {
        assert!(TypeInference::is_integer("123"));
        assert!(TypeInference::is_integer("-456"));
        assert!(!TypeInference::is_integer("123.45"));
        
        assert!(TypeInference::is_float("123.45"));
        assert!(TypeInference::is_float("-456.78"));
        assert!(TypeInference::is_float("123"));
    }
} 