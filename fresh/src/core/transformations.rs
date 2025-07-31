use datafusion::arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray, TimestampNanosecondArray, Array};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::compute;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::path::PathBuf;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, Clone, PartialEq)]
pub enum TransformationType {
    Delta,
    DeltaMultiple,
    TimeBin,
    RowId,
}

#[derive(Debug, Clone)]
pub struct TransformationConfig {
    pub transformation_type: TransformationType,
    pub selected_columns: Vec<String>,
    pub output_column_name: String,
    pub bin_size: Option<String>,
    pub time_column: Option<String>,
    pub grouping_columns: Option<Vec<String>>,
}

pub struct DataTransformer;

impl DataTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Apply delta transformation to compute differences between consecutive rows
    pub fn apply_delta(&self, batch: &RecordBatch, column_name: &str, output_name: &str) -> Result<RecordBatch> {
        let schema = batch.schema();
        let column_idx = schema.column_with_name(column_name)
            .ok_or_else(|| anyhow!("Column '{}' not found", column_name))?.0;

        let array = batch.column(column_idx);
        let delta_array = self.compute_delta(array)?;

        // Create new schema with additional column
        let mut new_fields = schema.fields().to_vec();
        new_fields.push(Arc::new(Field::new(output_name, delta_array.data_type().clone(), true)));
        let new_schema = Arc::new(Schema::new(new_fields));

        // Create new arrays with the delta column
        let mut new_arrays = batch.columns().to_vec();
        new_arrays.push(delta_array);

        Ok(RecordBatch::try_new(new_schema, new_arrays)?)
    }

    /// Apply delta transformation to multiple columns at once
    pub fn apply_delta_multiple(&self, batch: &RecordBatch, columns: &[String], output_prefix: &str) -> Result<RecordBatch> {
        let mut current_batch = batch.clone();
        let schema = current_batch.schema();

        for (i, column_name) in columns.iter().enumerate() {
            let output_name = format!("{}_{}", output_prefix, column_name);
            current_batch = self.apply_delta(&current_batch, column_name, &output_name)?;
        }

        Ok(current_batch)
    }

    /// Apply time binning transformation
    pub fn apply_time_bin(&self, batch: &RecordBatch, time_column: &str, bin_size_seconds: f64, output_name: &str) -> Result<RecordBatch> {
        let schema = batch.schema();
        let time_column_idx = schema.column_with_name(time_column)
            .ok_or_else(|| anyhow!("Time column '{}' not found", time_column))?.0;

        let time_array = batch.column(time_column_idx);
        let bin_array = self.compute_time_bins(time_array, bin_size_seconds)?;

        // Create new schema with bin column
        let mut new_fields = schema.fields().to_vec();
        new_fields.push(Arc::new(Field::new(output_name, DataType::Int64, true)));
        let new_schema = Arc::new(Schema::new(new_fields));

        // Create new arrays with the bin column
        let mut new_arrays = batch.columns().to_vec();
        new_arrays.push(bin_array);

        Ok(RecordBatch::try_new(new_schema, new_arrays)?)
    }

    /// Apply row ID transformation
    pub fn apply_row_id(&self, batch: &RecordBatch, output_name: &str, grouping_columns: Option<&[String]>) -> Result<RecordBatch> {
        let schema = batch.schema();
        let row_count = batch.num_rows();
        
        // Create row ID array
        let row_ids: Vec<i64> = (0..row_count as i64).collect();
        let row_id_array = Arc::new(Int64Array::from(row_ids));

        // Create new schema with row ID column
        let mut new_fields = schema.fields().to_vec();
        new_fields.push(Arc::new(Field::new(output_name, DataType::Int64, false)));
        let new_schema = Arc::new(Schema::new(new_fields));

        // Create new arrays with the row ID column
        let mut new_arrays = batch.columns().to_vec();
        new_arrays.push(row_id_array);

        let mut result_batch = RecordBatch::try_new(new_schema, new_arrays)?;

        // If grouping columns are specified, create group IDs
        if let Some(grouping_cols) = grouping_columns {
            let group_id_name = format!("{}_group", output_name);
            result_batch = self.apply_group_id(&result_batch, grouping_cols, &group_id_name)?;
        }

        Ok(result_batch)
    }

    /// Compute delta between consecutive values in an array
    fn compute_delta(&self, array: &ArrayRef) -> Result<ArrayRef> {
        match array.data_type() {
            DataType::Int64 => {
                let int_array = array.as_any().downcast_ref::<Int64Array>().unwrap();
                let mut deltas = Vec::with_capacity(int_array.len());
                
                for i in 0..int_array.len() {
                    if i == 0 {
                        deltas.push(None); // First row has no previous value
                    } else {
                        let current = int_array.value(i);
                        let previous = int_array.value(i - 1);
                        deltas.push(Some(current - previous));
                    }
                }
                
                Ok(Arc::new(Int64Array::from(deltas)))
            }
            DataType::Float64 => {
                let float_array = array.as_any().downcast_ref::<Float64Array>().unwrap();
                let mut deltas = Vec::with_capacity(float_array.len());
                
                for i in 0..float_array.len() {
                    if i == 0 {
                        deltas.push(None); // First row has no previous value
                    } else {
                        let current = float_array.value(i);
                        let previous = float_array.value(i - 1);
                        deltas.push(Some(current - previous));
                    }
                }
                
                Ok(Arc::new(Float64Array::from(deltas)))
            }
            _ => Err(anyhow!("Unsupported data type for delta computation: {:?}", array.data_type())),
        }
    }

    /// Compute time bins based on timestamp values
    fn compute_time_bins(&self, time_array: &ArrayRef, bin_size_seconds: f64) -> Result<ArrayRef> {
        match time_array.data_type() {
            DataType::Timestamp(_, _) => {
                let timestamp_array = time_array.as_any().downcast_ref::<TimestampNanosecondArray>().unwrap();
                let mut bins = Vec::with_capacity(timestamp_array.len());
                
                for i in 0..timestamp_array.len() {
                    if timestamp_array.is_null(i) {
                        bins.push(None);
                    } else {
                        let timestamp_nanos = timestamp_array.value(i);
                        let timestamp_seconds = timestamp_nanos as f64 / 1_000_000_000.0;
                        let bin = (timestamp_seconds / bin_size_seconds).floor() as i64;
                        bins.push(Some(bin));
                    }
                }
                
                Ok(Arc::new(Int64Array::from(bins)))
            }
            _ => Err(anyhow!("Unsupported data type for time binning: {:?}", time_array.data_type())),
        }
    }

    /// Apply group ID transformation based on grouping columns
    fn apply_group_id(&self, batch: &RecordBatch, grouping_columns: &[String], output_name: &str) -> Result<RecordBatch> {
        let schema = batch.schema();
        let mut group_ids = Vec::with_capacity(batch.num_rows());
        let mut current_group_id = 0i64;
        let mut group_key = String::new();
        let mut previous_group_key = String::new();

        for row_idx in 0..batch.num_rows() {
            // Build group key from grouping columns
            group_key.clear();
            for col_name in grouping_columns {
                let col_idx = schema.column_with_name(col_name)
                    .ok_or_else(|| anyhow!("Grouping column '{}' not found", col_name))?.0;
                let array = batch.column(col_idx);
                
                let value = self.format_array_value(array, row_idx);
                group_key.push_str(&value);
                group_key.push('|');
            }

            // Check if this is a new group
            if row_idx == 0 || group_key != previous_group_key {
                current_group_id += 1;
                previous_group_key = group_key.clone();
            }

            group_ids.push(current_group_id);
        }

        let group_id_array = Arc::new(Int64Array::from(group_ids));

        // Create new schema with group ID column
        let mut new_fields = schema.fields().to_vec();
        new_fields.push(Arc::new(Field::new(output_name, DataType::Int64, false)));
        let new_schema = Arc::new(Schema::new(new_fields));

        // Create new arrays with the group ID column
        let mut new_arrays = batch.columns().to_vec();
        new_arrays.push(group_id_array);

        Ok(RecordBatch::try_new(new_schema, new_arrays)?)
    }

    /// Format array value as string for grouping
    fn format_array_value(&self, array: &ArrayRef, row_idx: usize) -> String {
        if row_idx >= array.len() || array.is_null(row_idx) {
            return "null".to_string();
        }

        match array.data_type() {
            DataType::Utf8 => {
                let string_array = array.as_any().downcast_ref::<StringArray>().unwrap();
                string_array.value(row_idx).to_string()
            }
            DataType::Int64 => {
                let int_array = array.as_any().downcast_ref::<Int64Array>().unwrap();
                int_array.value(row_idx).to_string()
            }
            DataType::Float64 => {
                let float_array = array.as_any().downcast_ref::<Float64Array>().unwrap();
                format!("{:.2}", float_array.value(row_idx))
            }
            DataType::Boolean => {
                let bool_array = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                bool_array.value(row_idx).to_string()
            }
            _ => format!("{:?}", array.data_type()),
        }
    }

    /// Save transformed data to a new Arrow file
    pub fn save_transformed_data(&self, batch: &RecordBatch, output_path: &PathBuf) -> Result<()> {
        use datafusion::arrow::ipc::writer::FileWriter;
        use std::fs::File;

        let file = File::create(output_path)?;
        let mut writer = FileWriter::try_new(file, batch.schema().as_ref())?;
        writer.write(batch)?;
        writer.finish()?;
        
        Ok(())
    }

    /// Get available numeric columns from a batch
    pub fn get_numeric_columns(&self, batch: &RecordBatch) -> Vec<String> {
        let schema = batch.schema();
        let mut numeric_columns = Vec::new();

        for field in schema.fields() {
            match field.data_type() {
                DataType::Int64 | DataType::Float64 => {
                    numeric_columns.push(field.name().to_string());
                }
                _ => {}
            }
        }

        numeric_columns
    }

    /// Get available timestamp columns from a batch
    pub fn get_timestamp_columns(&self, batch: &RecordBatch) -> Vec<String> {
        let schema = batch.schema();
        let mut timestamp_columns = Vec::new();

        for field in schema.fields() {
            match field.data_type() {
                DataType::Timestamp(_, _) => {
                    timestamp_columns.push(field.name().to_string());
                }
                _ => {}
            }
        }

        timestamp_columns
    }
} 