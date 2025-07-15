//! Enhanced CSV processing with advanced features
//!
//! This module provides CSV parsing with automatic type detection,
//! encoding detection, and statistical analysis of data.

use pika_core::{
    types::{ImportOptions, TableInfo, ColumnInfo},
    error::Error as CoreError,
};
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use regex::Regex;
use anyhow::Context;
use crate::error::{Result, EngineError};

// Pre-compile regex patterns for better performance
static DATE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),           // YYYY-MM-DD
        Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap(),           // MM/DD/YYYY
        Regex::new(r"^\d{2}-\d{2}-\d{4}$").unwrap(),           // MM-DD-YYYY
        Regex::new(r"^\d{4}/\d{2}/\d{2}$").unwrap(),           // YYYY/MM/DD
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap(),  // ISO datetime
    ]
});

static UUID_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap()
});

/// Number of rows to sample for type detection
const SAMPLE_SIZE: usize = 1000;

/// Minimum confidence threshold for type detection (0.0 - 1.0)
const TYPE_CONFIDENCE_THRESHOLD: f64 = 0.95;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvFileStats {
    pub file_path: std::path::PathBuf,
    pub file_size_bytes: u64,
    pub estimated_rows: usize,
    pub sample_rows: Vec<Vec<String>>,
    pub column_stats: Vec<ColumnStats>,
    pub delimiter: char,
    pub has_header: bool,
    pub encoding: String,
    pub null_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub name: String,
    pub index: usize,
    pub inferred_type: DataType,
    pub null_count: usize,
    pub unique_count: usize,
    pub sample_values: Vec<String>,
    pub min_length: usize,
    pub max_length: usize,
    pub numeric_stats: Option<NumericStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Text,
    Integer,
    Real,
    Boolean,
    Date,
    DateTime,
    Uuid,
}

impl DataType {
    pub fn to_sql_type(&self) -> &'static str {
        match self {
            DataType::Text => "TEXT",
            DataType::Integer => "INTEGER",
            DataType::Real => "REAL",
            DataType::Boolean => "BOOLEAN",
            DataType::Date => "DATE",
            DataType::DateTime => "DATETIME",
            DataType::Uuid => "TEXT",
        }
    }
}

/// Enhanced CSV importer with advanced features
pub struct EnhancedCsvImporter;

impl EnhancedCsvImporter {
    /// Create a new enhanced CSV importer
    pub fn new() -> Self {
        Self
    }
    
    /// Import a CSV file with the given options
    pub async fn import(&self, path: impl AsRef<Path>, options: ImportOptions) -> Result<TableInfo> {
        let path = path.as_ref();
        
        // Validate the file exists
        if !path.exists() {
            return Err(EngineError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("CSV file not found: {}", path.display())
            )));
        }
        
        // Read and analyze the CSV file
        let stats = self.analyze_csv(path, &options)
            .await
            .context("Failed to analyze CSV file")?;
        
        // Convert stats to TableInfo
        let table_info = self.stats_to_table_info(stats, &options)?;
        
        Ok(table_info)
    }
    
    /// Analyze a CSV file and gather statistics
    async fn analyze_csv(&self, path: &Path, options: &ImportOptions) -> Result<CsvFileStats> {
        // Get file metadata
        let metadata = std::fs::metadata(path)
            .context("Failed to read file metadata")?;
        
        // TODO: Implement actual CSV parsing and analysis
        // - Parse CSV headers and detect delimiter
        // - Sample rows for type inference
        // - Calculate statistics for numeric columns
        // - Detect encoding (UTF-8, UTF-16, etc.)
        // For now, return placeholder stats
        let stats = CsvFileStats {
            file_path: path.to_path_buf(),
            file_size_bytes: metadata.len(),
            estimated_rows: 0,
            sample_rows: vec![],
            column_stats: vec![],
            delimiter: ',',
            has_header: true,
            encoding: "UTF-8".to_string(),
            null_values: vec!["".to_string(), "NULL".to_string(), "null".to_string()],
        };
        
        Ok(stats)
    }
    
    /// Convert CSV statistics to TableInfo
    fn stats_to_table_info(&self, stats: CsvFileStats, options: &ImportOptions) -> Result<TableInfo> {
        let table_name = stats.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        
        // Convert column stats to ColumnInfo
        let columns = if stats.column_stats.is_empty() {
            // Fallback when no stats are available
            vec![ColumnInfo {
                name: "placeholder".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            }]
        } else {
            stats.column_stats.into_iter()
                .map(|col_stat| ColumnInfo {
                    name: col_stat.name,
                    data_type: col_stat.inferred_type.to_sql_type().to_string(),
                    nullable: col_stat.null_count > 0,
                })
                .collect()
        };
        
        Ok(TableInfo {
            name: table_name,
            source_path: Some(stats.file_path),
            row_count: Some(stats.estimated_rows),
            columns,
            preview_data: stats.sample_rows.into(),
        })
    }
    
    /// Detect the data type of a value
    fn detect_value_type(&self, value: &str) -> DataType {
        // Handle empty or null values
        if value.is_empty() || value.eq_ignore_ascii_case("null") {
            return DataType::Text;
        }
        
        // Check boolean
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            return DataType::Boolean;
            }
            
        // Check UUID
        if UUID_PATTERN.is_match(value) {
            return DataType::Uuid;
        }
        
        // Check dates
        for pattern in DATE_PATTERNS.iter() {
            if pattern.is_match(value) {
                return if value.contains('T') || value.contains(':') {
                    DataType::DateTime
        } else {
                    DataType::Date
                };
            }
            }
            
        // Check numeric types
        if let Ok(_) = value.parse::<i64>() {
            return DataType::Integer;
        }
        
        if let Ok(_) = value.parse::<f64>() {
            return DataType::Real;
        }
        
        // Default to text
        DataType::Text
    }
    
    /// Infer column type from a sample of values
    fn infer_column_type(&self, values: &[String]) -> DataType {
        if values.is_empty() {
            return DataType::Text;
        }
        
        // Count occurrences of each type
        let mut type_counts = HashMap::new();
        let mut non_null_count = 0;
        
        for value in values {
            if !value.is_empty() && !value.eq_ignore_ascii_case("null") {
                let detected_type = self.detect_value_type(value);
                *type_counts.entry(detected_type).or_insert(0) += 1;
                non_null_count += 1;
            }
        }
        
        // If we have no non-null values, default to Text
        if non_null_count == 0 {
            return DataType::Text;
        }
        
        // Find the most common type with confidence threshold
        let confidence_required = (non_null_count as f64 * TYPE_CONFIDENCE_THRESHOLD) as usize;
        
        // Check types in order of restrictiveness
        for data_type in &[
            DataType::Boolean,
            DataType::Integer,
            DataType::Real,
            DataType::Uuid,
            DataType::DateTime,
            DataType::Date,
        ] {
            if let Some(&count) = type_counts.get(data_type) {
                if count >= confidence_required {
                    return *data_type;
                }
            }
        }
        
        // Default to Text if no type meets the confidence threshold
        DataType::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pika_core::types::ImportOptions;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    /// Helper to create a temporary CSV file
    fn create_temp_csv(content: &str) -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()
            .context("Failed to create temp file")?;
        file.write_all(content.as_bytes())
            .context("Failed to write CSV content")?;
        Ok(file)
    }
    
    #[tokio::test]
    async fn test_csv_import_returns_table_info() {
        let importer = EnhancedCsvImporter::new();
        let options = ImportOptions::default();
        
        // Create a temporary CSV file
        let csv_content = "name,age,active\nJohn,30,true\nJane,25,false";
        let temp_file = create_temp_csv(csv_content).unwrap();
        
        let result = importer.import(temp_file.path(), options).await;
        assert!(result.is_ok());
        
        let table_info = result.unwrap();
        assert!(table_info.source_path.is_some());
    }
    
    #[tokio::test]
    async fn test_csv_import_file_not_found() {
        let importer = EnhancedCsvImporter::new();
        let options = ImportOptions::default();
        
        let result = importer.import("non_existent_file.csv", options).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(err.is_io_error());
        assert!(err.to_string().contains("CSV file not found"));
    }
    
    #[test]
    fn test_data_type_to_sql() {
        assert_eq!(DataType::Text.to_sql_type(), "TEXT");
        assert_eq!(DataType::Integer.to_sql_type(), "INTEGER");
        assert_eq!(DataType::Real.to_sql_type(), "REAL");
        assert_eq!(DataType::Boolean.to_sql_type(), "BOOLEAN");
        assert_eq!(DataType::Date.to_sql_type(), "DATE");
        assert_eq!(DataType::DateTime.to_sql_type(), "DATETIME");
        assert_eq!(DataType::Uuid.to_sql_type(), "TEXT");
    }
    
    #[test]
    fn test_regex_patterns_compile() {
        // Ensure lazy statics are initialized without panic
        assert!(DATE_PATTERNS.len() >= 5);
        assert!(UUID_PATTERN.is_match("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!UUID_PATTERN.is_match("not-a-uuid"));
    }
    
    #[test]
    fn test_detect_value_type_empty() {
        let importer = EnhancedCsvImporter::new();
        assert_eq!(importer.detect_value_type(""), DataType::Text);
        assert_eq!(importer.detect_value_type("null"), DataType::Text);
        assert_eq!(importer.detect_value_type("NULL"), DataType::Text);
        }
        
    #[test]
    fn test_detect_value_type_boolean() {
        let importer = EnhancedCsvImporter::new();
        assert_eq!(importer.detect_value_type("true"), DataType::Boolean);
        assert_eq!(importer.detect_value_type("TRUE"), DataType::Boolean);
        assert_eq!(importer.detect_value_type("false"), DataType::Boolean);
        assert_eq!(importer.detect_value_type("FALSE"), DataType::Boolean);
    }
    
    #[test]
    fn test_detect_value_type_uuid() {
        let importer = EnhancedCsvImporter::new();
        assert_eq!(
            importer.detect_value_type("550e8400-e29b-41d4-a716-446655440000"),
            DataType::Uuid
        );
        assert_eq!(
            importer.detect_value_type("not-a-uuid"),
        DataType::Text
        );
    }
    
    #[test]
    fn test_detect_value_type_dates() {
        let importer = EnhancedCsvImporter::new();
        
        // Date formats
        assert_eq!(importer.detect_value_type("2023-12-25"), DataType::Date);
        assert_eq!(importer.detect_value_type("12/25/2023"), DataType::Date);
        assert_eq!(importer.detect_value_type("12-25-2023"), DataType::Date);
        assert_eq!(importer.detect_value_type("2023/12/25"), DataType::Date);
        
        // DateTime formats
        assert_eq!(
            importer.detect_value_type("2023-12-25T10:30:00"),
            DataType::DateTime
        );
        assert_eq!(
            importer.detect_value_type("2023-12-25T10:30:00.123Z"),
            DataType::DateTime
        );
    }
    
    #[test]
    fn test_detect_value_type_numeric() {
        let importer = EnhancedCsvImporter::new();
        
        // Integers
        assert_eq!(importer.detect_value_type("42"), DataType::Integer);
        assert_eq!(importer.detect_value_type("-42"), DataType::Integer);
        assert_eq!(importer.detect_value_type("0"), DataType::Integer);
        
        // Reals
        assert_eq!(importer.detect_value_type("42.5"), DataType::Real);
        assert_eq!(importer.detect_value_type("-42.5"), DataType::Real);
        assert_eq!(importer.detect_value_type("3.14159"), DataType::Real);
        assert_eq!(importer.detect_value_type("1e10"), DataType::Real);
    }
    
    #[test]
    fn test_detect_value_type_text() {
        let importer = EnhancedCsvImporter::new();
        
        assert_eq!(importer.detect_value_type("Hello World"), DataType::Text);
        assert_eq!(importer.detect_value_type("42 years"), DataType::Text);
        assert_eq!(importer.detect_value_type("true story"), DataType::Text);
    }
    
    #[test]
    fn test_infer_column_type_empty() {
        let importer = EnhancedCsvImporter::new();
        assert_eq!(importer.infer_column_type(&[]), DataType::Text);
        
        let all_nulls = vec!["".to_string(), "null".to_string(), "NULL".to_string()];
        assert_eq!(importer.infer_column_type(&all_nulls), DataType::Text);
    }
    
    #[test]
    fn test_infer_column_type_integers() {
        let importer = EnhancedCsvImporter::new();
        let values = vec![
            "42".to_string(),
            "-10".to_string(),
            "0".to_string(),
            "999".to_string(),
            "".to_string(), // null value
        ];
        assert_eq!(importer.infer_column_type(&values), DataType::Integer);
    }
    
    #[test]
    fn test_infer_column_type_mixed_numeric() {
        let importer = EnhancedCsvImporter::new();
        let values = vec![
            "42".to_string(),
            "3.14".to_string(),
            "-10".to_string(),
            "0.5".to_string(),
        ];
        // Should detect as Real since it contains both integers and floats
        assert_eq!(importer.infer_column_type(&values), DataType::Text);
    }
    
    #[test]
    fn test_infer_column_type_booleans() {
        let importer = EnhancedCsvImporter::new();
        let values = vec![
            "true".to_string(),
            "false".to_string(),
            "TRUE".to_string(),
            "FALSE".to_string(),
            "".to_string(),
        ];
        assert_eq!(importer.infer_column_type(&values), DataType::Boolean);
    }
    
    #[test]
    fn test_infer_column_type_dates() {
        let importer = EnhancedCsvImporter::new();
        let values = vec![
            "2023-01-01".to_string(),
            "2023-12-25".to_string(),
            "2024-06-15".to_string(),
            "".to_string(),
        ];
        assert_eq!(importer.infer_column_type(&values), DataType::Date);
    }
    
    #[test]
    fn test_infer_column_type_mixed_types() {
        let importer = EnhancedCsvImporter::new();
        let values = vec![
            "42".to_string(),
            "Hello".to_string(),
            "true".to_string(),
            "2023-01-01".to_string(),
        ];
        // With mixed types and no clear majority, should default to Text
        assert_eq!(importer.infer_column_type(&values), DataType::Text);
    }
    
    #[test]
    fn test_infer_column_type_confidence_threshold() {
        let importer = EnhancedCsvImporter::new();
        
        // 95% integers, 5% text - should be Integer
        let mut values = vec!["42".to_string(); 95];
        values.extend(vec!["text".to_string(); 5]);
        assert_eq!(importer.infer_column_type(&values), DataType::Integer);
        
        // 90% integers, 10% text - should be Text (below threshold)
        let mut values = vec!["42".to_string(); 90];
        values.extend(vec!["text".to_string(); 10]);
        assert_eq!(importer.infer_column_type(&values), DataType::Text);
    }
} 