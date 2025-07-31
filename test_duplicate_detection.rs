use fresh::core::{DuplicateDetector, DuplicateDetectionConfig};
use std::collections::HashSet;
use datafusion::arrow::array::{StringArray, Int64Array};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn main() {
    // Create test data where two groups have identical content but different group IDs
    let group_ids = vec!["group1", "group1", "group2", "group2"];
    let names = vec!["Alice", "Bob", "Alice", "Bob"];
    let ages = vec![25, 30, 25, 30];
    let jobs = vec!["Engineer", "Manager", "Engineer", "Manager"];
    let dates = vec!["2023-01-01", "2023-01-02", "2023-01-01", "2023-01-02"]; // Same dates!
    
    // Create Arrow arrays
    let group_id_array = StringArray::from(group_ids);
    let name_array = StringArray::from(names);
    let age_array = Int64Array::from(ages);
    let job_array = StringArray::from(jobs);
    let date_array = StringArray::from(dates);
    
    // Create schema
    let schema = Schema::new(vec![
        Field::new("group_id", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int64, false),
        Field::new("job", DataType::Utf8, false),
        Field::new("date", DataType::Utf8, false),
    ]);
    
    // Create RecordBatch
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(group_id_array),
            Arc::new(name_array),
            Arc::new(age_array),
            Arc::new(job_array),
            Arc::new(date_array),
        ]
    ).expect("Failed to create RecordBatch");
    
    println!("Created test data with {} rows", batch.num_rows());
    println!("Schema: {:?}", batch.schema());
    
    // Test 1: Detect duplicates ignoring the group_id column
    println!("=== Test 1: Ignoring group_id column ===");
    let config = DuplicateDetectionConfig {
        group_column: "group_id".to_string(),
        ignore_columns: {
            let mut set = HashSet::new();
            set.insert("group_id".to_string()); // Ignore the group_id column itself
            set
        },
        null_equals_null: true,
    };
    
    let detector = DuplicateDetector::new(config);
    let result = detector.detect_duplicates(&batch).expect("Detection failed");
    
    println!("Total duplicate groups: {}", result.total_duplicates);
    println!("Total duplicate rows: {}", result.total_duplicate_rows);
    println!("Groups processed: {}", result.stats.groups_processed);
    println!("Groups analyzed: {}", result.stats.groups_analyzed);
    println!("Unique groups found: {}", result.stats.unique_groups);
    
    for (i, group) in result.duplicate_groups.iter().enumerate() {
        println!("Duplicate group {}: ID={}, Size={}, Occurrences={}", 
                i + 1, group.group_id, group.group_size, group.row_indices.len());
    }
    
    // Test 2: Detect duplicates without ignoring any columns
    println!("\n=== Test 2: Not ignoring any columns ===");
    let config2 = DuplicateDetectionConfig {
        group_column: "group_id".to_string(),
        ignore_columns: HashSet::new(),
        null_equals_null: true,
    };
    
    let detector2 = DuplicateDetector::new(config2);
    let result2 = detector2.detect_duplicates(&batch).expect("Detection failed");
    
    println!("Total duplicate groups: {}", result2.total_duplicates);
    println!("Total duplicate rows: {}", result2.total_duplicate_rows);
    println!("Groups processed: {}", result2.stats.groups_processed);
    println!("Groups analyzed: {}", result2.stats.groups_analyzed);
    println!("Unique groups found: {}", result2.stats.unique_groups);
    
    for (i, group) in result2.duplicate_groups.iter().enumerate() {
        println!("Duplicate group {}: ID={}, Size={}, Occurrences={}", 
                i + 1, group.group_id, group.group_size, group.row_indices.len());
    }
    
    // Test 3: Create clean Arrow file
    if result.total_duplicates > 0 {
        println!("\n=== Test 3: Creating clean Arrow file ===");
        // Create the directory if it doesn't exist
        let output_dir = std::path::Path::new("test_duplicates");
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).expect("Failed to create test_duplicates directory");
        }
        let output_path = output_dir.join("clean_test_table.arrow");
        let kept_rows = detector.create_clean_arrow_file(&batch, &result, &output_path)
            .expect("Failed to create clean Arrow file");
        println!("Created clean Arrow file with {} rows kept", kept_rows);
    }
    
    println!("\nTest completed!");
} 