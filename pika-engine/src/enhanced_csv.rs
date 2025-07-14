//! Enhanced CSV processing with advanced features

use pika_core::{
    types::{ImportOptions, TableInfo, ColumnInfo},
    error::Result,
};
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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

/// Enhanced CSV analyzer with fast statistical processing
pub struct CsvAnalyzer {
    sample_size: usize,
    max_unique_values: usize,
}

impl Default for CsvAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CsvAnalyzer {
    pub fn new() -> Self {
        Self {
            sample_size: 10000,
            max_unique_values: 1000,
        }
    }
    
    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }
    
    /// Fast analysis of CSV file with statistical inference
    pub async fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<CsvFileStats> {
        let path = path.as_ref();
        let file_size = std::fs::metadata(path)
            .map_err(|e| pika_core::error::PikaError::Io(e))?
            .len();
        
        // Fast delimiter detection
        let delimiter = self.detect_delimiter(path)?;
        
        // Read sample for analysis
        let (sample_rows, has_header) = self.read_sample(path, delimiter)?;
        
        if sample_rows.is_empty() {
            return Err(pika_core::error::PikaError::DataProcessing("Empty CSV file".to_string()));
        }
        
        let column_count = sample_rows[0].len();
        let estimated_rows = self.estimate_total_rows(file_size, &sample_rows);
        
        // Generate column statistics
        let column_stats = self.analyze_columns(&sample_rows, has_header);
        
        Ok(CsvFileStats {
            file_path: path.to_path_buf(),
            file_size_bytes: file_size,
            estimated_rows,
            sample_rows,
            column_stats,
            delimiter,
            has_header,
            encoding: "UTF-8".to_string(),
            null_values: vec!["".to_string(), "NULL".to_string(), "null".to_string(), "N/A".to_string()],
        })
    }
    
    fn detect_delimiter<P: AsRef<Path>>(&self, path: P) -> Result<char> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| pika_core::error::PikaError::Io(e))?;
        
        let sample = content.lines().take(5).collect::<Vec<_>>().join("\n");
        
        let delimiters = [',', ';', '\t', '|'];
        let mut best_delimiter = ',';
        let mut best_score = 0;
        
        for &delimiter in &delimiters {
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(delimiter as u8)
                .has_headers(false)
                .from_reader(sample.as_bytes());
            
            let mut consistent_columns = true;
            let mut column_count = None;
            let mut row_count = 0;
            
            for result in reader.records() {
                if let Ok(record) = result {
                    let current_count = record.len();
                    
                    if let Some(expected) = column_count {
                        if current_count != expected {
                            consistent_columns = false;
                            break;
                        }
                    } else {
                        column_count = Some(current_count);
                    }
                    
                    row_count += 1;
                    if row_count >= 5 { break; }
                }
            }
            
            if consistent_columns {
                let score = column_count.unwrap_or(0) * row_count;
                if score > best_score {
                    best_score = score;
                    best_delimiter = delimiter;
                }
            }
        }
        
        Ok(best_delimiter)
    }
    
    fn read_sample<P: AsRef<Path>>(&self, path: P, delimiter: char) -> Result<(Vec<Vec<String>>, bool)> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .has_headers(false)
            .from_path(path)
            .map_err(|e| pika_core::error::PikaError::DataProcessing(format!("Failed to read CSV: {}", e)))?;
        
        let mut rows = Vec::new();
        
        for (i, result) in reader.records().enumerate() {
            if i >= self.sample_size {
                break;
            }
            
            match result {
                Ok(record) => {
                    let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                    rows.push(row);
                }
                Err(e) => {
                    eprintln!("Warning: Skipping malformed row {}: {}", i, e);
                    continue;
                }
            }
        }
        
        // Detect if first row is header
        let has_header = if rows.len() >= 2 {
            self.detect_header(&rows[0], &rows[1])
        } else {
            false
        };
        
        Ok((rows, has_header))
    }
    
    fn detect_header(&self, first_row: &[String], second_row: &[String]) -> bool {
        if first_row.len() != second_row.len() {
            return false;
        }
        
        let mut header_indicators = 0;
        let total_columns = first_row.len();
        
        for (header_val, data_val) in first_row.iter().zip(second_row.iter()) {
            // Check if header looks like a name (contains letters, no pure numbers)
            let header_has_letters = header_val.chars().any(|c| c.is_alphabetic());
            let header_is_number = header_val.parse::<f64>().is_ok();
            
            // Check if data looks different from header
            let data_is_number = data_val.parse::<f64>().is_ok();
            
            if header_has_letters && !header_is_number {
                header_indicators += 1;
            }
            
            if header_has_letters && data_is_number {
                header_indicators += 1;
            }
        }
        
        // If more than half the columns look like headers, assume first row is header
        header_indicators > total_columns / 2
    }
    
    fn estimate_total_rows(&self, file_size: u64, sample_rows: &[Vec<String>]) -> usize {
        if sample_rows.is_empty() {
            return 0;
        }
        
        // Estimate average row size in bytes
        let sample_size_bytes: usize = sample_rows.iter()
            .map(|row| row.iter().map(|cell| cell.len() + 1).sum::<usize>()) // +1 for delimiter
            .sum();
        
        let avg_row_size = sample_size_bytes as f64 / sample_rows.len() as f64;
        
        if avg_row_size > 0.0 {
            (file_size as f64 / avg_row_size) as usize
        } else {
            sample_rows.len()
        }
    }
    
    fn analyze_columns(&self, rows: &[Vec<String>], has_header: bool) -> Vec<ColumnStats> {
        if rows.is_empty() {
            return Vec::new();
        }
        
        let column_count = rows[0].len();
        let data_start = if has_header { 1 } else { 0 };
        let data_rows = &rows[data_start..];
        
        let mut column_stats = Vec::new();
        
        for col_idx in 0..column_count {
            let column_name = if has_header && !rows[0].is_empty() && col_idx < rows[0].len() {
                rows[0][col_idx].clone()
            } else {
                format!("column_{}", col_idx + 1)
            };
            
            let column_values: Vec<&String> = data_rows.iter()
                .filter_map(|row| row.get(col_idx))
                .collect();
            
            let stats = self.analyze_column(&column_values, &column_name, col_idx);
            column_stats.push(stats);
        }
        
        column_stats
    }
    
    fn analyze_column(&self, values: &[&String], name: &str, index: usize) -> ColumnStats {
        let mut null_count = 0;
        let mut unique_values = HashMap::new();
        let mut numeric_values = Vec::new();
        let mut min_length = usize::MAX;
        let mut max_length = 0;
        
        for &value in values {
            let trimmed = value.trim();
            
            // Count nulls
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") || trimmed.eq_ignore_ascii_case("n/a") {
                null_count += 1;
                continue;
            }
            
            // Track lengths
            min_length = min_length.min(trimmed.len());
            max_length = max_length.max(trimmed.len());
            
            // Track unique values (up to limit)
            if unique_values.len() < self.max_unique_values {
                *unique_values.entry(trimmed.to_string()).or_insert(0) += 1;
            }
            
            // Try to parse as number
            if let Ok(num) = trimmed.parse::<f64>() {
                numeric_values.push(num);
            }
        }
        
        if min_length == usize::MAX {
            min_length = 0;
        }
        
        // Infer data type
        let inferred_type = self.infer_data_type(values, &numeric_values);
        
        // Calculate numeric statistics if applicable
        let numeric_stats = if !numeric_values.is_empty() && numeric_values.len() > values.len() / 2 {
            Some(self.calculate_numeric_stats(&numeric_values))
        } else {
            None
        };
        
        // Get sample values
        let sample_values: Vec<String> = unique_values.keys()
            .take(10)
            .cloned()
            .collect();
        
        ColumnStats {
            name: name.to_string(),
            index,
            inferred_type,
            null_count,
            unique_count: unique_values.len(),
            sample_values,
            min_length,
            max_length,
            numeric_stats,
        }
    }
    
    fn infer_data_type(&self, values: &[&String], numeric_values: &[f64]) -> DataType {
        let non_null_count = values.iter()
            .filter(|v| !v.trim().is_empty() && !v.trim().eq_ignore_ascii_case("null"))
            .count();
        
        if non_null_count == 0 {
            return DataType::Text;
        }
        
        // Check if most values are numeric
        let numeric_ratio = numeric_values.len() as f64 / non_null_count as f64;
        
        if numeric_ratio > 0.8 {
            // Check if all numeric values are integers
            let all_integers = numeric_values.iter()
                .all(|&n| n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64);
            
            if all_integers {
                return DataType::Integer;
            } else {
                return DataType::Real;
            }
        }
        
        // Check for boolean values
        let boolean_count = values.iter()
            .filter(|v| {
                let trimmed = v.trim().to_lowercase();
                trimmed == "true" || trimmed == "false" || 
                trimmed == "yes" || trimmed == "no" ||
                trimmed == "1" || trimmed == "0"
            })
            .count();
        
        if boolean_count as f64 / non_null_count as f64 > 0.8 {
            return DataType::Boolean;
        }
        
        // Check for date patterns (basic)
        let date_count = values.iter()
            .filter(|v| self.looks_like_date(v))
            .count();
        
        if date_count as f64 / non_null_count as f64 > 0.8 {
            return DataType::Date;
        }
        
        // Check for UUID patterns
        let uuid_count = values.iter()
            .filter(|v| self.looks_like_uuid(v))
            .count();
        
        if uuid_count as f64 / non_null_count as f64 > 0.8 {
            return DataType::Uuid;
        }
        
        DataType::Text
    }
    
    fn looks_like_date(&self, value: &str) -> bool {
        let trimmed = value.trim();
        
        // Basic date patterns
        let date_patterns = [
            r"^\d{4}-\d{2}-\d{2}$",           // YYYY-MM-DD
            r"^\d{2}/\d{2}/\d{4}$",           // MM/DD/YYYY
            r"^\d{2}-\d{2}-\d{4}$",           // MM-DD-YYYY
            r"^\d{4}/\d{2}/\d{2}$",           // YYYY/MM/DD
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}",  // ISO datetime
        ];
        
        for pattern in &date_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(trimmed) {
                return true;
            }
        }
        
        false
    }
    
    fn looks_like_uuid(&self, value: &str) -> bool {
        let trimmed = value.trim();
        
        // UUID pattern: 8-4-4-4-12 hex digits
        let uuid_pattern = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
        
        regex::Regex::new(uuid_pattern).unwrap().is_match(trimmed)
    }
    
    fn calculate_numeric_stats(&self, values: &[f64]) -> NumericStats {
        if values.is_empty() {
            return NumericStats {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                std_dev: 0.0,
            };
        }
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        NumericStats {
            min,
            max,
            mean,
            std_dev,
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
        
        // For now, return a basic TableInfo
        // This will be enhanced with actual CSV parsing later
        Ok(TableInfo {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string(),
            source_path: Some(path.to_path_buf()),
            row_count: None,
            columns: vec![
                ColumnInfo {
                    name: "placeholder".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                }
            ],
            preview_data: None,
        })
    }
} 