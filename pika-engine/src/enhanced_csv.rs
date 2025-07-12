//! Enhanced CSV handling functionality extracted from pebble
//! Provides advanced CSV reading, writing, and analysis capabilities

use csv::{Reader, Writer, StringRecord, ReaderBuilder, WriterBuilder};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufWriter, BufRead};
use pika_core::{Result, PikaError, types::ImportOptions};
use arrow::record_batch::RecordBatch;
use arrow::array::{StringArray, Float64Array, Int64Array, ArrayRef};
use arrow::datatypes::{Schema, Field, DataType};
use std::sync::Arc;
use std::collections::HashMap;

/// Enhanced CSV reader with advanced features
pub struct EnhancedCsvReader {
    path: PathBuf,
    options: ImportOptions,
    reader: Option<Reader<BufReader<File>>>,
    sample_cache: Option<Vec<StringRecord>>,
    headers_cache: Option<Vec<String>>,
}

impl EnhancedCsvReader {
    /// Create a new enhanced CSV reader
    pub fn new<P: AsRef<Path>>(path: P, options: ImportOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(PikaError::internal(format!("CSV file '{}' does not exist", path.display())));
        }
        
        Ok(Self {
            path,
            options,
            reader: None,
            sample_cache: None,
            headers_cache: None,
        })
    }
    
    /// Build a CSV reader with current options
    fn build_reader(&self) -> Result<Reader<BufReader<File>>> {
        let file = File::open(&self.path)
            .map_err(|e| PikaError::internal(format!("Failed to open file: {}", e)))?;
        
        let reader = ReaderBuilder::new()
            .delimiter(self.options.delimiter as u8)
            .has_headers(self.options.has_header)
            .quote(self.options.quote_char.unwrap_or('"') as u8)
            .escape(self.options.escape_char.map(|c| c as u8))
            .from_reader(BufReader::new(file));
        
        Ok(reader)
    }
    
    /// Get column headers
    pub fn headers(&mut self) -> Result<Vec<String>> {
        if let Some(cached) = &self.headers_cache {
            return Ok(cached.clone());
        }
        
        let headers = if self.options.has_header {
            let mut reader = self.build_reader()?;
            if let Some(result) = reader.records().next() {
                result.map_err(|e| PikaError::internal(format!("Failed to read headers: {}", e)))?
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            } else {
                return Err(PikaError::internal("Empty CSV file".to_string()));
            }
        } else {
            // Generate default column names
            let sample = self.sample_records(1)?;
            if sample.is_empty() {
                return Err(PikaError::internal("Empty CSV file".to_string()));
            }
            
            (0..sample[0].len())
                .map(|i| format!("column_{}", i))
                .collect::<Vec<String>>()
        };
        
        self.headers_cache = Some(headers.clone());
        Ok(headers)
    }
    
    /// Get a sample of records for analysis
    pub fn sample_records(&mut self, n: usize) -> Result<Vec<StringRecord>> {
        if let Some(cached) = &self.sample_cache {
            return Ok(cached.iter().take(n).cloned().collect());
        }
        
        let mut reader = self.build_reader()?;
        let mut records = Vec::new();
        
        // Skip header if present
        if self.options.has_header {
            if let Some(result) = reader.records().next() {
                result.map_err(|e| PikaError::internal(format!("Failed to read header: {}", e)))?;
            }
        }
        
        // Skip initial rows if specified
        for _ in 0..self.options.skip_rows {
            if let Some(result) = reader.records().next() {
                result.map_err(|e| PikaError::internal(format!("Failed to skip row: {}", e)))?;
            }
        }
        
        // Read sample records
        for (i, result) in reader.records().enumerate() {
            if i >= n {
                break;
            }
            let record = result.map_err(|e| PikaError::internal(format!("Failed to read record {}: {}", i, e)))?;
            records.push(record);
        }
        
        self.sample_cache = Some(records.clone());
        Ok(records)
    }
    
    /// Analyze column types from sample data
    pub fn analyze_column_types(&mut self, sample_size: usize) -> Result<Vec<DataType>> {
        let sample = self.sample_records(sample_size)?;
        let headers = self.headers()?;
        
        if sample.is_empty() {
            return Ok(vec![DataType::Utf8; headers.len()]);
        }
        
        let mut column_types = vec![DataType::Utf8; headers.len()];
        
        for (col_idx, _header) in headers.iter().enumerate() {
            let mut all_integers = true;
            let mut all_floats = true;
            let mut has_values = false;
            
            for record in &sample {
                if let Some(value) = record.get(col_idx) {
                    if !value.trim().is_empty() {
                        has_values = true;
                        
                        // Try to parse as integer
                        if all_integers && value.parse::<i64>().is_err() {
                            all_integers = false;
                        }
                        
                        // Try to parse as float
                        if all_floats && value.parse::<f64>().is_err() {
                            all_floats = false;
                        }
                        
                        // If neither integer nor float, it's a string
                        if !all_integers && !all_floats {
                            break;
                        }
                    }
                }
            }
            
            if has_values {
                if all_integers {
                    column_types[col_idx] = DataType::Int64;
                } else if all_floats {
                    column_types[col_idx] = DataType::Float64;
                }
            }
        }
        
        Ok(column_types)
    }
    
    /// Read all records and convert to Arrow RecordBatch
    pub fn to_record_batch(&mut self) -> Result<RecordBatch> {
        let headers = self.headers()?;
        let column_types = self.analyze_column_types(1000)?;
        
        // Build schema
        let fields: Vec<Field> = headers.iter()
            .zip(column_types.iter())
            .map(|(name, data_type)| Field::new(name, data_type.clone(), true))
            .collect();
        let schema = Schema::new(fields);
        
        // Read all records
        let mut reader = self.build_reader()?;
        let mut records = Vec::new();
        
        // Skip header if present
        if self.options.has_header {
            if let Some(result) = reader.records().next() {
                result.map_err(|e| PikaError::internal(format!("Failed to read header: {}", e)))?;
            }
        }
        
        // Skip initial rows if specified
        for _ in 0..self.options.skip_rows {
            if let Some(result) = reader.records().next() {
                result.map_err(|e| PikaError::internal(format!("Failed to skip row: {}", e)))?;
            }
        }
        
        // Read data records
        let mut record_count = 0;
        for result in reader.records() {
            if let Some(max_rows) = self.options.max_rows {
                if record_count >= max_rows {
                    break;
                }
            }
            
            let record = result.map_err(|e| PikaError::internal(format!("Failed to read record {}: {}", record_count, e)))?;
            records.push(record);
            record_count += 1;
        }
        
        // Convert to Arrow arrays
        let mut arrays: Vec<ArrayRef> = Vec::new();
        
        for (col_idx, data_type) in column_types.iter().enumerate() {
            match data_type {
                DataType::Int64 => {
                    let values: Vec<Option<i64>> = records.iter()
                        .map(|record| {
                            record.get(col_idx)
                                .and_then(|s| if s.trim().is_empty() { None } else { s.parse().ok() })
                        })
                        .collect();
                    arrays.push(Arc::new(Int64Array::from(values)));
                }
                DataType::Float64 => {
                    let values: Vec<Option<f64>> = records.iter()
                        .map(|record| {
                            record.get(col_idx)
                                .and_then(|s| if s.trim().is_empty() { None } else { s.parse().ok() })
                        })
                        .collect();
                    arrays.push(Arc::new(Float64Array::from(values)));
                }
                _ => {
                    let values: Vec<Option<String>> = records.iter()
                        .map(|record| {
                            record.get(col_idx)
                                .map(|s| if s.trim().is_empty() { None } else { Some(s.to_string()) })
                                .unwrap_or(None)
                        })
                        .collect();
                    arrays.push(Arc::new(StringArray::from(values)));
                }
            }
        }
        
        RecordBatch::try_new(Arc::new(schema), arrays)
            .map_err(|e| PikaError::internal(format!("Failed to create RecordBatch: {}", e)))
    }
    
    /// Get file statistics
    pub fn file_stats(&self) -> Result<CsvFileStats> {
        let metadata = std::fs::metadata(&self.path)
            .map_err(|e| PikaError::internal(format!("Failed to get file metadata: {}", e)))?;
        
        // Simple estimation based on file size (rough approximation)
        let estimated_rows = if metadata.len() > 0 {
            (metadata.len() / 50).max(1) as usize  // Rough estimate: 50 bytes per row
        } else {
            0
        };
        
        let final_estimated_rows = if self.options.has_header {
            estimated_rows.saturating_sub(1)
        } else {
            estimated_rows
        };
        
        Ok(CsvFileStats {
            file_size: metadata.len(),
            estimated_rows: final_estimated_rows,
            encoding: self.options.encoding.clone(),
            delimiter: self.options.delimiter,
            has_header: self.options.has_header,
        })
    }
}

/// Enhanced CSV writer with advanced features
pub struct EnhancedCsvWriter {
    writer: Writer<BufWriter<File>>,
    options: CsvWriteOptions,
}

#[derive(Debug, Clone)]
pub struct CsvWriteOptions {
    pub delimiter: char,
    pub quote_char: char,
    pub escape_char: Option<char>,
    pub quote_style: QuoteStyle,
    pub include_headers: bool,
}

#[derive(Debug, Clone)]
pub enum QuoteStyle {
    Always,
    Necessary,
    Never,
}

impl Default for CsvWriteOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote_char: '"',
            escape_char: None,
            quote_style: QuoteStyle::Necessary,
            include_headers: true,
        }
    }
}

impl EnhancedCsvWriter {
    /// Create a new enhanced CSV writer
    pub fn new<P: AsRef<Path>>(path: P, options: CsvWriteOptions) -> Result<Self> {
        let file = File::create(path)
            .map_err(|e| PikaError::internal(format!("Failed to create file: {}", e)))?;
        
        let mut builder = WriterBuilder::new();
        builder.delimiter(options.delimiter as u8);
        builder.quote(options.quote_char as u8);
        if let Some(escape) = options.escape_char {
            builder.escape(escape as u8);
        }
        
        let writer = builder.from_writer(BufWriter::new(file));
        
        Ok(Self { writer, options })
    }
    
    /// Write headers
    pub fn write_headers(&mut self, headers: &[String]) -> Result<()> {
        if self.options.include_headers {
            self.writer.write_record(headers)
                .map_err(|e| PikaError::internal(format!("Failed to write headers: {}", e)))?;
        }
        Ok(())
    }
    
    /// Write a single record
    pub fn write_record(&mut self, record: &[String]) -> Result<()> {
        self.writer.write_record(record)
            .map_err(|e| PikaError::internal(format!("Failed to write record: {}", e)))?;
        Ok(())
    }
    
    /// Write a RecordBatch to CSV
    pub fn write_record_batch(&mut self, batch: &RecordBatch) -> Result<()> {
        let schema = batch.schema();
        
        // Write headers if enabled
        if self.options.include_headers {
            let headers: Vec<String> = schema.fields()
                .iter()
                .map(|field| field.name().clone())
                .collect();
            self.write_headers(&headers)?;
        }
        
        // Write data rows
        for row_idx in 0..batch.num_rows() {
            let mut row_data = Vec::new();
            
            for col_idx in 0..batch.num_columns() {
                let array = batch.column(col_idx);
                let value = if array.is_null(row_idx) {
                    String::new()
                } else {
                    arrow::util::display::array_value_to_string(array, row_idx)
                        .unwrap_or_else(|_| String::new())
                };
                row_data.push(value);
            }
            
            self.write_record(&row_data)?;
        }
        
        Ok(())
    }
    
    /// Flush the writer
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
            .map_err(|e| PikaError::internal(format!("Failed to flush writer: {}", e)))?;
        Ok(())
    }
}

/// CSV file statistics
#[derive(Debug, Clone)]
pub struct CsvFileStats {
    pub file_size: u64,
    pub estimated_rows: usize,
    pub encoding: String,
    pub delimiter: char,
    pub has_header: bool,
}

/// CSV analysis utilities
pub struct CsvAnalyzer;

impl CsvAnalyzer {
    /// Detect delimiter from sample data
    pub fn detect_delimiter<P: AsRef<Path>>(path: P) -> Result<char> {
        let file = File::open(path)
            .map_err(|e| PikaError::internal(format!("Failed to open file: {}", e)))?;
        
        let mut reader = BufReader::new(file);
        let mut sample = String::new();
        
        // Read first few lines for analysis
        for _ in 0..5 {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)
                .map_err(|e| PikaError::internal(format!("Failed to read line: {}", e)))?;
            if bytes_read == 0 {
                break;
            }
            sample.push_str(&line);
        }
        
        // Count occurrences of common delimiters
        let delimiters = [',', ';', '\t', '|'];
        let mut counts = HashMap::new();
        
        for &delimiter in &delimiters {
            counts.insert(delimiter, sample.matches(delimiter).count());
        }
        
        // Return the most common delimiter
        Ok(counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(delimiter, _)| delimiter)
            .unwrap_or(','))
    }
    
    /// Detect if file has headers
    pub fn detect_headers<P: AsRef<Path>>(path: P, delimiter: char) -> Result<bool> {
        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .has_headers(false)
            .from_path(path)
            .map_err(|e| PikaError::internal(format!("Failed to open file: {}", e)))?;
        
        let mut records = reader.records();
        
        // Get first two records
        let first_record = records.next()
            .ok_or_else(|| PikaError::internal("Empty file".to_string()))?
            .map_err(|e| PikaError::internal(format!("Failed to read first record: {}", e)))?;
        
        let second_record = records.next()
            .ok_or_else(|| PikaError::internal("File has only one record".to_string()))?
            .map_err(|e| PikaError::internal(format!("Failed to read second record: {}", e)))?;
        
        // Check if first record looks like headers
        let first_numeric_count = first_record.iter()
            .filter(|field| field.parse::<f64>().is_ok())
            .count();
        
        let second_numeric_count = second_record.iter()
            .filter(|field| field.parse::<f64>().is_ok())
            .count();
        
        // If first record has significantly fewer numeric values, it's likely headers
        Ok(first_numeric_count < second_numeric_count / 2)
    }
} 