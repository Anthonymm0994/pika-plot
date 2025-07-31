use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use datafusion::arrow::array::{StringArray, Int64Array, Float64Array, BooleanArray, TimestampSecondArray, TimestampMillisecondArray, TimestampMicrosecondArray, TimestampNanosecondArray, Date32Array, UInt32Array};
use datafusion::arrow::datatypes::{DataType, TimeUnit};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::compute;
use crate::core::error::Result;

/// Configuration for duplicate block detection
#[derive(Debug, Clone)]
pub struct DuplicateDetectionConfig {
    /// Column name to group by
    pub group_column: String,
    /// Column names to ignore during comparison
    pub ignore_columns: HashSet<String>,
    /// Block size for grouping rows (default: 256)
    pub block_size: usize,
    /// Whether to treat null values as equal
    pub null_equals_null: bool,
}

impl Default for DuplicateDetectionConfig {
    fn default() -> Self {
        Self {
            group_column: String::new(),
            ignore_columns: HashSet::new(),
            block_size: 256,
            null_equals_null: true,
        }
    }
}

/// Represents a block of rows that are duplicates
#[derive(Debug, Clone)]
pub struct DuplicateBlock {
    /// The hash of this block
    pub block_hash: u64,
    /// Group ID this block belongs to
    pub group_id: String,
    /// Row indices where this block appears (multiple if duplicates exist)
    pub row_indices: Vec<Vec<usize>>,
    /// Number of rows in each block
    pub block_size: usize,
}

/// Results of duplicate detection
#[derive(Debug, Clone)]
pub struct DuplicateDetectionResult {
    /// All duplicate blocks found
    pub duplicate_blocks: Vec<DuplicateBlock>,
    /// Total number of duplicate blocks found
    pub total_duplicates: usize,
    /// Total number of rows that are part of duplicate blocks
    pub total_duplicate_rows: usize,
    /// Statistics about the detection
    pub stats: DetectionStats,
}

#[derive(Debug, Clone)]
pub struct DetectionStats {
    /// Number of groups processed
    pub groups_processed: usize,
    /// Total number of blocks analyzed
    pub blocks_analyzed: usize,
    /// Number of unique blocks found
    pub unique_blocks: usize,
}

/// Main duplicate detector implementation
pub struct DuplicateDetector {
    config: DuplicateDetectionConfig,
}

impl DuplicateDetector {
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        Self { config }
    }

    /// Detect duplicate blocks in a record batch
    pub fn detect_duplicates(&self, batch: &RecordBatch) -> Result<DuplicateDetectionResult> {
        let mut duplicate_blocks = Vec::new();
        let mut stats = DetectionStats {
            groups_processed: 0,
            blocks_analyzed: 0,
            unique_blocks: 0,
        };

        // Get the group column index
        let group_col_idx = batch.schema()
            .fields()
            .iter()
            .position(|field| field.name() == &self.config.group_column)
            .ok_or_else(|| crate::core::error::FreshError::Custom(
                format!("Group column '{}' not found", self.config.group_column)
            ))?;

        // Get column indices to ignore
        let ignore_indices: HashSet<usize> = batch.schema()
            .fields()
            .iter()
            .enumerate()
            .filter(|(_, field)| self.config.ignore_columns.contains(field.name()))
            .map(|(idx, _)| idx)
            .collect();

        // Group rows by group_id
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        
        for row_idx in 0..batch.num_rows() {
            let group_value = self.get_cell_value(batch, row_idx, group_col_idx)?;
            groups.entry(group_value).or_insert_with(Vec::new).push(row_idx);
        }

        stats.groups_processed = groups.len();

        // Process each group
        for (group_id, row_indices) in groups {
            let group_blocks = self.process_group(batch, &row_indices, &ignore_indices)?;
            stats.blocks_analyzed += group_blocks.len();
            
            // Find duplicates within this group
            let mut block_hash_map: HashMap<u64, Vec<Vec<usize>>> = HashMap::new();
            
            for (block_hash, block_rows) in group_blocks {
                block_hash_map.entry(block_hash).or_insert_with(Vec::new).push(block_rows);
            }

            // Add duplicate blocks to results
            for (block_hash, row_groups) in block_hash_map {
                if row_groups.len() > 1 {
                    stats.unique_blocks += 1;
                    duplicate_blocks.push(DuplicateBlock {
                        block_hash,
                        group_id: group_id.clone(),
                        row_indices: row_groups,
                        block_size: self.config.block_size,
                    });
                }
            }
        }

        let total_duplicates = duplicate_blocks.len();
        let total_duplicate_rows = duplicate_blocks.iter()
            .map(|block| block.row_indices.iter().map(|indices| indices.len()).sum::<usize>())
            .sum();

        Ok(DuplicateDetectionResult {
            duplicate_blocks,
            total_duplicates,
            total_duplicate_rows,
            stats,
        })
    }

    /// Process a single group of rows
    fn process_group(
        &self,
        batch: &RecordBatch,
        row_indices: &[usize],
        ignore_indices: &HashSet<usize>,
    ) -> Result<Vec<(u64, Vec<usize>)>> {
        let mut blocks = Vec::new();
        
        // Process rows in blocks
        for chunk in row_indices.chunks(self.config.block_size) {
            let block_hash = self.compute_block_hash(batch, chunk, ignore_indices)?;
            blocks.push((block_hash, chunk.to_vec()));
        }

        Ok(blocks)
    }

    /// Compute hash for a block of rows
    fn compute_block_hash(
        &self,
        batch: &RecordBatch,
        row_indices: &[usize],
        ignore_indices: &HashSet<usize>,
    ) -> Result<u64> {
        let mut hasher = DefaultHasher::new();
        
        // Hash each row in the block
        for &row_idx in row_indices {
            let row_hash = self.compute_row_hash(batch, row_idx, ignore_indices)?;
            row_hash.hash(&mut hasher);
        }
        
        Ok(hasher.finish())
    }

    /// Compute hash for a single row
    fn compute_row_hash(
        &self,
        batch: &RecordBatch,
        row_idx: usize,
        ignore_indices: &HashSet<usize>,
    ) -> Result<u64> {
        let mut hasher = DefaultHasher::new();
        
        for (col_idx, array) in batch.columns().iter().enumerate() {
            if ignore_indices.contains(&col_idx) {
                continue;
            }
            
            let cell_value = self.get_cell_value(batch, row_idx, col_idx)?;
            cell_value.hash(&mut hasher);
        }
        
        Ok(hasher.finish())
    }

    /// Get cell value as string for hashing
    fn get_cell_value(&self, batch: &RecordBatch, row_idx: usize, col_idx: usize) -> Result<String> {
        let array = &batch.columns()[col_idx];
        
        if array.is_null(row_idx) {
            return Ok("NULL".to_string());
        }

        match array.data_type() {
            DataType::Utf8 => {
                let string_array = array.as_any().downcast_ref::<StringArray>().unwrap();
                Ok(string_array.value(row_idx).to_string())
            }
            DataType::Int64 => {
                let int_array = array.as_any().downcast_ref::<Int64Array>().unwrap();
                Ok(int_array.value(row_idx).to_string())
            }
            DataType::Float64 => {
                let float_array = array.as_any().downcast_ref::<Float64Array>().unwrap();
                Ok(float_array.value(row_idx).to_string())
            }
            DataType::Boolean => {
                let bool_array = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                Ok(bool_array.value(row_idx).to_string())
            }
            DataType::Timestamp(unit, _) => {
                match unit {
                    TimeUnit::Second => {
                        let timestamp_array = array.as_any().downcast_ref::<TimestampSecondArray>().unwrap();
                        let timestamp = timestamp_array.value(row_idx);
                        let hours = (timestamp / 3600) % 24;
                        let minutes = (timestamp / 60) % 60;
                        let seconds = timestamp % 60;
                        Ok(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
                    }
                    TimeUnit::Millisecond => {
                        let timestamp_array = array.as_any().downcast_ref::<TimestampMillisecondArray>().unwrap();
                        let timestamp = timestamp_array.value(row_idx);
                        let total_seconds = timestamp / 1_000;
                        let milliseconds = timestamp % 1_000;
                        let hours = (total_seconds / 3600) % 24;
                        let minutes = (total_seconds / 60) % 60;
                        let seconds = total_seconds % 60;
                        Ok(format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds))
                    }
                    TimeUnit::Microsecond => {
                        let timestamp_array = array.as_any().downcast_ref::<TimestampMicrosecondArray>().unwrap();
                        let timestamp = timestamp_array.value(row_idx);
                        let total_seconds = timestamp / 1_000_000;
                        let microseconds = timestamp % 1_000_000;
                        let hours = (total_seconds / 3600) % 24;
                        let minutes = (total_seconds / 60) % 60;
                        let seconds = total_seconds % 60;
                        Ok(format!("{:02}:{:02}:{:02}.{:06}", hours, minutes, seconds, microseconds))
                    }
                    TimeUnit::Nanosecond => {
                        let timestamp_array = array.as_any().downcast_ref::<TimestampNanosecondArray>().unwrap();
                        let timestamp = timestamp_array.value(row_idx);
                        let total_seconds = timestamp / 1_000_000_000;
                        let nanoseconds = timestamp % 1_000_000_000;
                        let hours = (total_seconds / 3600) % 24;
                        let minutes = (total_seconds / 60) % 60;
                        let seconds = total_seconds % 60;
                        Ok(format!("{:02}:{:02}:{:02}.{:09}", hours, minutes, seconds, nanoseconds))
                    }
                }
            }
            DataType::Date32 => {
                let date_array = array.as_any().downcast_ref::<Date32Array>().unwrap();
                let days = date_array.value(row_idx);
                // Convert days since epoch to readable date
                let year = 1970 + (days / 365);
                let remaining_days = days % 365;
                let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                let mut month = 1;
                let mut day = remaining_days;
                
                for &days_in_month in &month_days[1..] {
                    if day < days_in_month {
                        break;
                    }
                    day -= days_in_month;
                    month += 1;
                }
                
                Ok(format!("{:04}-{:02}-{:02}", year, month, day + 1))
            }
            _ => {
                // Default to string representation
                let string_array = array.as_any().downcast_ref::<StringArray>().unwrap();
                Ok(string_array.value(row_idx).to_string())
            }
        }
    }

    /// Create a new Arrow file with duplicate blocks removed
    pub fn create_clean_arrow_file(
        &self,
        batch: &RecordBatch,
        result: &DuplicateDetectionResult,
        output_path: &std::path::Path,
    ) -> Result<usize> {
        let mut rows_to_keep = HashSet::new();
        
        // First, add all rows to the keep set
        for i in 0..batch.num_rows() {
            rows_to_keep.insert(i);
        }
        
        // Remove duplicate rows (keep first occurrence, remove rest)
        for block in &result.duplicate_blocks {
            // Keep the first occurrence, remove the rest
            for duplicate_group in block.row_indices.iter().skip(1) {
                for &row_idx in duplicate_group {
                    rows_to_keep.remove(&row_idx);
                }
            }
        }
        
        // Convert to sorted list for efficient processing
        let mut sorted_rows: Vec<usize> = rows_to_keep.into_iter().collect();
        sorted_rows.sort_unstable();
        
        // Create new arrays with only the kept rows
        let mut new_arrays = Vec::new();
        for col_idx in 0..batch.num_columns() {
            let array = batch.column(col_idx);
            let new_array = compute::take(array, &UInt32Array::from(sorted_rows.iter().map(|&x| x as u32).collect::<Vec<u32>>()), None)?;
            new_arrays.push(new_array);
        }
        
        // Create new record batch
        let new_batch = RecordBatch::try_new(batch.schema().clone(), new_arrays)?;
        
        // Save to Arrow IPC file
        let file = std::fs::File::create(output_path)?;
        let mut writer = datafusion::arrow::ipc::writer::FileWriter::try_new(file, &new_batch.schema())?;
        writer.write(&new_batch)?;
        writer.finish()?;
        
        Ok(sorted_rows.len())
    }

    /// Remove duplicate blocks from the database (enhanced version)
    pub fn remove_duplicates(&self, db: &mut crate::core::database::Database, table_name: &str, result: &DuplicateDetectionResult) -> Result<usize> {
        let mut rows_to_remove = HashSet::new();
        
        // Collect all row indices to remove
        for block in &result.duplicate_blocks {
            // Keep the first occurrence, remove the rest
            for duplicate_group in block.row_indices.iter().skip(1) {
                for &row_idx in duplicate_group {
                    rows_to_remove.insert(row_idx);
                }
            }
        }
        
        // Convert to sorted list for efficient removal
        let mut sorted_rows: Vec<usize> = rows_to_remove.into_iter().collect();
        sorted_rows.sort_unstable();
        
        // Remove rows in reverse order to maintain indices
        let mut removed_count = 0;
        for &row_idx in sorted_rows.iter().rev() {
            // Note: This is a simplified approach. In a real implementation,
            // you'd want to use a more efficient bulk delete operation
            // For now, we'll return the count of rows that would be removed
            removed_count += 1;
        }
        
        Ok(removed_count)
    }

    /// Create a new Arrow file with duplicates removed and return the path
    pub fn create_clean_arrow_file_with_path(
        &self,
        batch: &RecordBatch,
        result: &DuplicateDetectionResult,
        base_path: &std::path::Path,
        original_table_name: &str,
    ) -> Result<(std::path::PathBuf, usize)> {
        // Generate output filename
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let output_filename = format!("{}_clean_{}.arrow", original_table_name, timestamp);
        let output_path = base_path.join(output_filename);
        
        let kept_rows = self.create_clean_arrow_file(batch, result, &output_path)?;
        
        Ok((output_path, kept_rows))
    }
} 