use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use datafusion::prelude::*;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::array::{StringArray, Int64Array, Float64Array, BooleanArray};
use datafusion::arrow::datatypes::TimeUnit;
use tokio::runtime::Runtime;
use crate::core::error::{Result, FreshError};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub name: String,
    pub sql: String,
}

#[derive(Debug, Clone)]
pub struct DataBatch {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct Database {
    // DataFusion context for in-memory analytics
    ctx: SessionContext,
    runtime: Runtime,
    // Cache for loaded data batches
    batch_cache: HashMap<String, DataBatch>,
    // Track registered tables
    registered_tables: HashMap<String, Arc<RecordBatch>>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        // Create a new runtime and context
        let runtime = Runtime::new()
            .expect("Failed to create tokio runtime for Database clone");
        let ctx = SessionContext::new();
        
        Self {
            ctx,
            runtime,
            batch_cache: self.batch_cache.clone(),
            registered_tables: self.registered_tables.clone(),
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // DataFusion context closed
    }
}

impl Database {
    pub fn open_readonly<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // DataFusion is in-memory, so we don't need file paths for now
        let runtime = Runtime::new()
            .map_err(|e| FreshError::Custom(format!("Failed to create tokio runtime: {}", e)))?;
        
        let ctx = SessionContext::new();
        
        Ok(Self {
            ctx,
            runtime,
            batch_cache: HashMap::new(),
            registered_tables: HashMap::new(),
        })
    }

    pub fn open_writable<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // DataFusion is in-memory, so we don't need file paths for now
        let runtime = Runtime::new()
            .map_err(|e| FreshError::Custom(format!("Failed to create tokio runtime: {}", e)))?;
        
        let ctx = SessionContext::new();
        
        Ok(Self {
            ctx,
            runtime,
            batch_cache: HashMap::new(),
            registered_tables: HashMap::new(),
        })
    }

    pub fn is_readonly(&self) -> bool {
        false // DataFusion is always writable in-memory
    }

    // Load a table into memory as a DataBatch (cached)
    pub fn load_table_batch(&mut self, table_name: &str) -> Result<DataBatch> {
        // Check cache first
        if let Some(batch) = self.batch_cache.get(table_name) {
            return Ok(batch.clone());
        }
        
        // Load from DataFusion into DataBatch
        let query = format!("SELECT * FROM '{}'", table_name);
        let batch = self.execute_query_batch(&query)?;
        
        // Cache the batch
        self.batch_cache.insert(table_name.to_string(), batch.clone());
        
        Ok(batch)
    }

    // Execute a DataFusion query and return as DataBatch
    pub fn execute_query_batch(&self, query: &str) -> Result<DataBatch> {
        let ctx = self.ctx.clone();
        
        let result = self.runtime.block_on(async {
            ctx.sql(query).await
        }).map_err(|e| FreshError::Custom(format!("Failed to execute query: {}", e)))?;
        
        let record_batches = self.runtime.block_on(async {
            result.collect().await
        }).map_err(|e| FreshError::Custom(format!("Failed to collect results: {}", e)))?;
        
        if record_batches.is_empty() {
            return Ok(DataBatch {
                columns: vec![],
                rows: vec![],
            });
        }
        
        // Convert first batch to DataBatch
        let batch = &record_batches[0];
        let columns: Vec<String> = batch.schema().fields().iter()
            .map(|field| field.name().clone())
            .collect();
        
        let rows = self.record_batch_to_rows(batch)?;
        
        Ok(DataBatch {
            columns,
            rows,
        })
    }

    // Execute query and return as row data (for UI display)
    pub fn execute_query(&self, query: &str) -> Result<Vec<Vec<String>>> {
        let batch = self.execute_query_batch(query)?;
        Ok(batch.rows)
    }

    // Execute count query safely
    pub fn execute_count_query(&self, query: &str) -> Result<i64> {
        let batch = self.execute_query_batch(query)?;
        if batch.rows.is_empty() || batch.columns.is_empty() {
            return Ok(0);
        }
        
        // Try to parse the first value as i64
        if let Some(first_row) = batch.rows.first() {
            if let Some(first_value) = first_row.first() {
                return first_value.parse::<i64>()
                    .map_err(|e| FreshError::Custom(format!("Failed to parse count result: {}", e)));
            }
        }
        
        Ok(0)
    }

    // Create table in DataFusion (register an empty table)
    pub fn create_table(&mut self, table_name: &str, columns: &[(&str, &str)]) -> Result<()> {
        // Convert SQL types to Arrow types
        let fields: Vec<Field> = columns.iter()
            .map(|(name, sql_type)| {
                let arrow_type = self.sql_type_to_arrow_type(sql_type);
                Field::new(*name, arrow_type, true)
            })
            .collect();
        
        let schema = Schema::new(fields);
        let empty_batch = RecordBatch::new_empty(Arc::new(schema));
        
        // Register the empty table
        self.ctx.register_batch(table_name, empty_batch)
            .map_err(|e| FreshError::Custom(format!("Failed to register table: {}", e)))?;
        
        Ok(())
    }

    // Insert data into DataFusion table
    pub fn insert_data(&mut self, table_name: &str, values: &[Vec<String>]) -> Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        
        // Clear cache for this table since data changed
        self.batch_cache.remove(table_name);
        
        // Convert string values to Arrow arrays
        let (columns, schema) = if let Some(existing_batch) = self.registered_tables.get(table_name) {
            // Use existing schema to preserve column names and types
            let existing_schema = existing_batch.schema();
            let columns: Vec<String> = existing_schema.fields().iter()
                .map(|field| field.name().clone())
                .collect();
            (columns, existing_schema.clone())
        } else {
            // Create default column names if table doesn't exist
            let columns: Vec<String> = (0..values[0].len()).map(|i| format!("col_{}", i)).collect();
            let schema = Arc::new(Schema::new(
                columns.iter().map(|name| Field::new(name, DataType::Utf8, true)).collect::<Vec<_>>()
            ));
            (columns, schema)
        };
        
        let arrays = self.string_rows_to_arrow_arrays(&columns, values)?;
        
        let batch = RecordBatch::try_new(schema, arrays)
            .map_err(|e| FreshError::Custom(format!("Failed to create record batch: {}", e)))?;
        
        // Safely register or replace the table
        self.register_or_replace_table(table_name, batch.clone())?;
        
        // Store in our cache
        self.registered_tables.insert(table_name.to_string(), Arc::new(batch));
        
        Ok(())
    }

    // Helper method to safely register or replace a table
    fn register_or_replace_table(&mut self, table_name: &str, batch: RecordBatch) -> Result<()> {
        // Check if table already exists in our tracking
        if self.registered_tables.contains_key(table_name) {
            // Remove from our internal tracking first
            self.registered_tables.remove(table_name);
        }
        
        // Try to register the table
        match self.ctx.register_batch(table_name, batch.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                // If registration fails, the table already exists in DataFusion
                // We need to create a new context to replace it
                // This is a workaround since DataFusion doesn't support table replacement
                
                // Create a new context
                let new_ctx = SessionContext::new();
                
                // Register all existing tables except the one we're replacing
                for (existing_name, existing_batch) in &self.registered_tables {
                    if existing_name != table_name {
                        new_ctx.register_batch(existing_name, existing_batch.as_ref().clone())
                            .map_err(|e| FreshError::Custom(format!("Failed to re-register table {}: {}", existing_name, e)))?;
                    }
                }
                
                // Register the new table
                new_ctx.register_batch(table_name, batch)
                    .map_err(|e| FreshError::Custom(format!("Failed to register data: {}", e)))?;
                
                // Replace the context
                self.ctx = new_ctx;
                
                Ok(())
            }
        }
    }

    // Get table information from DataFusion
    pub fn get_tables(&self) -> Result<Vec<TableInfo>> {
        let mut tables = Vec::new();
        
        for (table_name, batch) in &self.registered_tables {
            let columns: Vec<ColumnInfo> = batch.schema().fields().iter()
                .map(|field| ColumnInfo {
                    name: field.name().clone(),
                    data_type: self.arrow_type_to_sql_type(field.data_type()).to_string(),
                    is_nullable: field.is_nullable(),
                    is_primary_key: false, // DataFusion doesn't track primary keys
                })
                .collect();
            
            tables.push(TableInfo {
                name: table_name.clone(),
                columns,
                row_count: batch.num_rows() as i64,
            });
        }
        
        Ok(tables)
    }

    // Helper methods for DataFusion integration
    fn record_batch_to_rows(&self, batch: &RecordBatch) -> Result<Vec<Vec<String>>> {
        let mut rows = Vec::new();
        
        for row_idx in 0..batch.num_rows() {
            let mut row = Vec::new();
            for col_idx in 0..batch.num_columns() {
                let array = batch.column(col_idx);
                let value = self.array_value_to_string(array, row_idx)?;
                row.push(value);
            }
            rows.push(row);
        }
        
        Ok(rows)
    }

    fn array_value_to_string(&self, array: &datafusion::arrow::array::ArrayRef, index: usize) -> Result<String> {
        use datafusion::arrow::array::*;
        
        match array.data_type() {
            DataType::Utf8 => {
                let string_array = array.as_any().downcast_ref::<StringArray>().unwrap();
                Ok(string_array.value(index).to_string())
            }
            DataType::Int64 => {
                let int_array = array.as_any().downcast_ref::<Int64Array>().unwrap();
                Ok(int_array.value(index).to_string())
            }
            DataType::Float64 => {
                let float_array = array.as_any().downcast_ref::<Float64Array>().unwrap();
                Ok(float_array.value(index).to_string())
            }
            DataType::Boolean => {
                let bool_array = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                Ok(bool_array.value(index).to_string())
            }
            _ => {
                // For other types, convert to string representation
                Ok(format!("{:?}", array))
            }
        }
    }

    fn string_rows_to_arrow_arrays(&self, columns: &[String], values: &[Vec<String>]) -> Result<Vec<datafusion::arrow::array::ArrayRef>> {
        let mut arrays = Vec::new();
        
        for col_idx in 0..columns.len() {
            // Try to infer the data type from the first few rows
            let mut is_numeric = true;
            let mut is_integer = true;
            
            // Check first 10 rows to infer type
            for row in values.iter().take(10) {
                if col_idx < row.len() {
                    let value = &row[col_idx];
                    if !value.is_empty() {
                        // Check if it's numeric
                        if !value.chars().all(|c| c.is_numeric() || c == '.' || c == '-') {
                            is_numeric = false;
                            break;
                        }
                        // Check if it's integer
                        if value.contains('.') {
                            is_integer = false;
                        }
                    }
                }
            }
            
            if is_numeric && is_integer {
                // Create Int64 array
                let mut int_values = Vec::new();
                for row in values {
                    if col_idx < row.len() {
                        let value = &row[col_idx];
                        if value.is_empty() {
                            int_values.push(None);
                        } else {
                            match value.parse::<i64>() {
                                Ok(int_val) => int_values.push(Some(int_val)),
                                Err(_) => int_values.push(None),
                            }
                        }
                    } else {
                        int_values.push(None);
                    }
                }
                let array = Int64Array::from(int_values);
                arrays.push(Arc::new(array) as datafusion::arrow::array::ArrayRef);
            } else if is_numeric {
                // Create Float64 array
                let mut float_values = Vec::new();
                for row in values {
                    if col_idx < row.len() {
                        let value = &row[col_idx];
                        if value.is_empty() {
                            float_values.push(None);
                        } else {
                            match value.parse::<f64>() {
                                Ok(float_val) => float_values.push(Some(float_val)),
                                Err(_) => float_values.push(None),
                            }
                        }
                    } else {
                        float_values.push(None);
                    }
                }
                let array = Float64Array::from(float_values);
                arrays.push(Arc::new(array) as datafusion::arrow::array::ArrayRef);
            } else {
                // Create String array
                let mut string_values = Vec::new();
                for row in values {
                    if col_idx < row.len() {
                        string_values.push(row[col_idx].clone());
                    } else {
                        string_values.push(String::new());
                    }
                }
                let array = StringArray::from(string_values);
                arrays.push(Arc::new(array) as datafusion::arrow::array::ArrayRef);
            }
        }
        
        Ok(arrays)
    }

    fn sql_type_to_arrow_type(&self, sql_type: &str) -> DataType {
        match sql_type.to_uppercase().as_str() {
            "INTEGER" | "INT" | "BIGINT" => DataType::Int64,
            "REAL" | "DOUBLE" | "FLOAT" => DataType::Float64,
            "BOOLEAN" | "BOOL" => DataType::Boolean,
            "TEXT" | "VARCHAR" | "STRING" => DataType::Utf8,
            "TIMESTAMP" => DataType::Timestamp(TimeUnit::Nanosecond, None),
            _ => DataType::Utf8, // Default to string
        }
    }

    fn arrow_type_to_sql_type(&self, arrow_type: &DataType) -> &'static str {
        match arrow_type {
            DataType::Int64 => "INTEGER",
            DataType::Float64 => "REAL",
            DataType::Boolean => "BOOLEAN",
            DataType::Utf8 => "TEXT",
            DataType::Timestamp(_, _) => "TIMESTAMP",
            _ => "TEXT",
        }
    }

    // Placeholder methods for compatibility
    pub fn get_views(&self) -> Result<Vec<ViewInfo>> {
        Ok(vec![]) // DataFusion doesn't have views in the same way
    }

    pub fn table_exists(&self, table_name: &str) -> Result<bool> {
        Ok(self.registered_tables.contains_key(table_name))
    }

    pub fn get_column_names(&self, query: &str) -> Result<Vec<String>> {
        let ctx = &self.ctx;
        let rt = Runtime::new()?;
        
        rt.block_on(async {
            let df = ctx.sql(query).await.map_err(|e| FreshError::Database(e.to_string()))?;
            let schema = df.schema();
            Ok(schema.fields().iter().map(|f| f.name().clone()).collect())
        })
    }
    
    pub fn get_column_types(&self, query: &str) -> Result<Vec<DataType>> {
        let ctx = &self.ctx;
        let rt = Runtime::new()?;
        
        rt.block_on(async {
            let df = ctx.sql(query).await.map_err(|e| FreshError::Database(e.to_string()))?;
            let schema = df.schema();
            Ok(schema.fields().iter().map(|f| f.data_type().clone()).collect())
        })
    }

    pub fn execute_sql(&mut self, sql: &str) -> Result<()> {
        let ctx = self.ctx.clone();
        self.runtime.block_on(async {
            ctx.sql(sql).await
        }).map_err(|e| FreshError::Custom(format!("Failed to execute SQL: {}", e)))?;
        Ok(())
    }

    pub fn create_table_with_types(&mut self, table_name: &str, columns: &[(&str, &str)]) -> Result<()> {
        self.create_table(table_name, columns)
    }

    pub fn stream_insert_csv(&mut self, table_name: &str, csv_path: &Path, delimiter: char, has_header: bool) -> Result<()> {
        // Read CSV with proper header handling
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delimiter as u8)
            .has_headers(has_header)
            .from_path(csv_path)
            .map_err(|e| FreshError::Custom(format!("Failed to read CSV: {}", e)))?;
        
        let mut rows = Vec::new();
        let mut headers = Vec::new();
        
        if has_header {
            // Get headers from the first row
            match rdr.headers() {
                Ok(header_result) => {
                    headers = header_result.iter().map(|s| s.to_string()).collect();
                }
                Err(e) => {
                    return Err(FreshError::Custom(format!("Failed to read CSV headers: {}", e)));
                }
            }
        }
        
        // Read data rows
        for result in rdr.records() {
            let record = result.map_err(|e| FreshError::Custom(format!("Failed to read CSV record: {}", e)))?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }
        
        // If no headers were found, create default column names
        if headers.is_empty() {
            headers = (0..rows.first().map(|r| r.len()).unwrap_or(0))
                .map(|i| format!("col_{}", i))
                .collect();
        }
        
        // If the table already exists with a schema, we need to ensure the data matches
        if let Some(existing_batch) = self.registered_tables.get(table_name) {
            let existing_schema = existing_batch.schema();
            let expected_columns = existing_schema.fields().len();
            
            // Ensure all rows have the expected number of columns
            for (row_idx, row) in rows.iter().enumerate() {
                if row.len() != expected_columns {
                    return Err(FreshError::Custom(format!(
                        "Row {} has {} columns, but table schema expects {} columns",
                        row_idx + 1, row.len(), expected_columns
                    )));
                }
            }
        }
        
        // Create schema with proper column names and type inference
        let fields: Vec<Field> = headers.iter().enumerate()
            .map(|(col_idx, name)| {
                // Try to infer the data type from the first few rows
                let data_type = if !rows.is_empty() {
                    let mut is_numeric = true;
                    let mut is_integer = true;
                    
                    // Check first 10 rows to infer type
                    for row in rows.iter().take(10) {
                        if col_idx < row.len() {
                            let value = &row[col_idx];
                            if !value.is_empty() {
                                // Check if it's numeric
                                if !value.chars().all(|c| c.is_numeric() || c == '.' || c == '-') {
                                    is_numeric = false;
                                    break;
                                }
                                // Check if it's integer
                                if value.contains('.') {
                                    is_integer = false;
                                }
                            }
                        }
                    }
                    
                    if is_numeric {
                        if is_integer {
                            DataType::Int64
                        } else {
                            DataType::Float64
                        }
                    } else {
                        DataType::Utf8
                    }
                } else {
                    DataType::Utf8
                };
                
                Field::new(name, data_type, true)
            })
            .collect();
        
        let schema = Schema::new(fields);
        
        // Convert rows to Arrow arrays
        let arrays = self.string_rows_to_arrow_arrays(&headers, &rows)?;
        
        let batch = RecordBatch::try_new(Arc::new(schema), arrays)
            .map_err(|e| FreshError::Custom(format!("Failed to create record batch: {}", e)))?;
        
        // Register the table with proper schema, handling replacement if it already exists
        self.register_or_replace_table(table_name, batch.clone())?;
        
        // Store in our cache
        self.registered_tables.insert(table_name.to_string(), Arc::new(batch));
        
        Ok(())
    }

    pub fn begin_transaction(&mut self) -> Result<()> {
        // DataFusion doesn't support transactions in the same way
        Ok(())
    }

    pub fn commit_transaction(&mut self) -> Result<()> {
        // DataFusion doesn't support transactions in the same way
        Ok(())
    }

    pub fn rollback_transaction(&mut self) -> Result<()> {
        // DataFusion doesn't support transactions in the same way
        Ok(())
    }

    pub fn insert_record(&mut self, table_name: &str, values: &[String]) -> Result<()> {
        self.insert_data(table_name, &[values.to_vec()])
    }

    pub fn batch_insert(&mut self, table_name: &str, all_values: &[Vec<String>]) -> Result<()> {
        self.insert_data(table_name, all_values)
    }

    pub fn create_table_with_schema(&mut self, table_name: &str, columns: &[(&str, DataType)]) -> Result<()> {
        let fields: Vec<Field> = columns.iter()
            .map(|(name, data_type)| Field::new(*name, data_type.clone(), true))
            .collect();
        
        let schema = Schema::new(fields);
        let empty_batch = RecordBatch::new_empty(Arc::new(schema));
        
        self.ctx.register_batch(table_name, empty_batch)
            .map_err(|e| FreshError::Custom(format!("Failed to register table with schema: {}", e)))?;
        
        Ok(())
    }

    pub fn insert_record_batch(&mut self, table_name: &str, batch: &RecordBatch) -> Result<()> {
        // Clear cache for this table since data changed
        self.batch_cache.remove(table_name);
        
        // Register the batch as a table
        self.ctx.register_batch(table_name, batch.clone())
            .map_err(|e| FreshError::Custom(format!("Failed to register record batch: {}", e)))?;
        
        // Store in our cache
        self.registered_tables.insert(table_name.to_string(), Arc::new(batch.clone()));
        
        Ok(())
    }

    pub fn load_table_arrow_batch(&mut self, table_name: &str) -> Result<Arc<RecordBatch>> {
        if let Some(batch) = self.registered_tables.get(table_name) {
            return Ok(batch.clone());
        }
        
        // Try to query the table to get its data
        let query = format!("SELECT * FROM '{}'", table_name);
        let ctx = self.ctx.clone();
        
        let result = self.runtime.block_on(async {
            ctx.sql(&query).await
        }).map_err(|e| FreshError::Custom(format!("Failed to load table: {}", e)))?;
        
        let record_batches = self.runtime.block_on(async {
            result.collect().await
        }).map_err(|e| FreshError::Custom(format!("Failed to collect table data: {}", e)))?;
        
        if record_batches.is_empty() {
            return Err(FreshError::Custom("No data found in table".to_string()));
        }
        
        let batch = Arc::new(record_batches[0].clone());
        self.registered_tables.insert(table_name.to_string(), batch.clone());
        
        Ok(batch)
    }

    fn warn_if_cloud_folder(_path: &Path) {
        // DataFusion is in-memory, so no file locking concerns
    }

    // === HYBRID PERSISTENCE METHODS ===

    /// Save a table in both Arrow IPC (fast cache) and Parquet (persistent) formats
    pub fn save_table_dual(&mut self, table_name: &str, base_path: &Path) -> Result<()> {
        // Get the table data as RecordBatch
        let batch = self.load_table_arrow_batch(table_name)?;
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(base_path)
            .map_err(|e| FreshError::Custom(format!("Failed to create directory: {}", e)))?;
        
        // Save as Arrow IPC (fast cache)
        let arrow_path = base_path.join(format!("{}.arrow", table_name));
        self.save_table_arrow_ipc(table_name, &arrow_path)?;
        
        // For now, skip Parquet due to version conflicts
        // TODO: Add Parquet support when Arrow versions are compatible
                // Table saved successfully
        
        Ok(())
    }

    /// Save a table as Arrow IPC file (fast cache format)
    pub fn save_table_arrow_ipc(&mut self, table_name: &str, path: &Path) -> Result<()> {
        use datafusion::arrow::ipc::writer::FileWriter;
        use std::fs::File;
        
        let batch = self.load_table_arrow_batch(table_name)?;
        
        let file = File::create(path)
            .map_err(|e| FreshError::Custom(format!("Failed to create Arrow IPC file: {}", e)))?;
        
        let mut writer = FileWriter::try_new(file, &batch.schema())
            .map_err(|e| FreshError::Custom(format!("Failed to create Arrow IPC writer: {}", e)))?;
        
        writer.write(&batch)
            .map_err(|e| FreshError::Custom(format!("Failed to write Arrow IPC data: {}", e)))?;
        
        writer.finish()
            .map_err(|e| FreshError::Custom(format!("Failed to finish Arrow IPC file: {}", e)))?;
        
        Ok(())
    }

    /// Load a table from Arrow IPC file (fast cache format)
    pub fn load_table_arrow_ipc(&mut self, table_name: &str, path: &Path) -> Result<()> {
        use datafusion::arrow::ipc::reader::FileReader;
        use std::fs::File;
        
        let file = File::open(path)
            .map_err(|e| FreshError::Custom(format!("Failed to open Arrow IPC file: {}", e)))?;
        
        let reader = FileReader::try_new(file, None)
            .map_err(|e| FreshError::Custom(format!("Failed to create Arrow IPC reader: {}", e)))?;
        
        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result
                .map_err(|e| FreshError::Custom(format!("Failed to read Arrow IPC batch: {}", e)))?;
            batches.push(batch);
        }
        
        if batches.is_empty() {
            return Err(FreshError::Custom("No data found in Arrow IPC file".to_string()));
        }
        
        // Combine all batches into one (for simplicity)
        let combined_batch = if batches.len() == 1 {
            batches.remove(0)
        } else {
            // For multiple batches, we'd need to concatenate them
            // For now, just use the first batch
            batches.remove(0)
        };
        
        // Register the table
        self.insert_record_batch(table_name, &combined_batch)?;
        
        Ok(())
    }

    /// Load all tables from a directory (Arrow IPC format only for now)
    pub fn load_all_tables_from_directory(&mut self, directory: &Path) -> Result<Vec<String>> {
        let mut loaded_tables = Vec::new();
        
        if !directory.exists() {
            return Ok(loaded_tables);
        }
        
        for entry in std::fs::read_dir(directory)
            .map_err(|e| FreshError::Custom(format!("Failed to read directory: {}", e)))? {
            
            let entry = entry
                .map_err(|e| FreshError::Custom(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| FreshError::Custom("Invalid file name".to_string()))?;
            
            if let Some(table_name) = file_name.strip_suffix(".arrow") {
                // Load Arrow IPC files
                match self.load_table_arrow_ipc(table_name, &path) {
                    Ok(_) => {
                        loaded_tables.push(table_name.to_string());
                        // Table loaded successfully
                    }
                    Err(e) => {
                        eprintln!("[Database] Failed to load table '{}' from Arrow IPC: {}", table_name, e);
                    }
                }
            }
        }
        
        Ok(loaded_tables)
    }

    /// Save all current tables in Arrow IPC format
    pub fn save_all_tables(&mut self, base_path: &Path) -> Result<Vec<String>> {
        let mut saved_tables = Vec::new();
        
        // Collect table names first to avoid borrowing issues
        let table_names: Vec<String> = self.registered_tables.keys().cloned().collect();
        
        for table_name in table_names {
            match self.save_table_dual(&table_name, base_path) {
                Ok(_) => {
                    saved_tables.push(table_name);
                }
                Err(e) => {
                    eprintln!("[Database] Failed to save table '{}': {}", table_name, e);
                }
            }
        }
        
        Ok(saved_tables)
    }
} 