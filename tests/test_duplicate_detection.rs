use fresh::core::{Database, DuplicateDetector, DuplicateDetectionConfig};
use std::collections::HashSet;
use std::path::Path;

#[test]
fn test_duplicate_detection_basic() {
    // Create a simple test database
    let mut db = Database::open_writable(":memory:").unwrap();
    
    // Create a simple table with duplicates
    db.create_table("test_table", &[
        ("group_id", "VARCHAR"),
        ("name", "VARCHAR"),
        ("value", "INTEGER"),
        ("id", "INTEGER"),
    ]).unwrap();
    
    // Insert test data with duplicates
    let test_data = vec![
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "1".to_string()],
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "2".to_string()],
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "3".to_string()],
        vec!["B".to_string(), "Jane".to_string(), "200".to_string(), "4".to_string()],
        vec!["B".to_string(), "Jane".to_string(), "200".to_string(), "5".to_string()],
        vec!["C".to_string(), "Bob".to_string(), "300".to_string(), "6".to_string()],
    ];
    
    db.batch_insert("test_table", &test_data).unwrap();
    
    // Get the table data
    let batch = db.get_table_arrow_batch("test_table").unwrap();
    assert_eq!(batch.num_rows(), 6);
    
    // Create detector configuration
    let config = DuplicateDetectionConfig {
        group_column: "group_id".to_string(),
        ignore_columns: HashSet::from(["id".to_string()]),
        block_size: 256,
        null_equals_null: true,
    };
    
    let detector = DuplicateDetector::new(config);
    
    // Run detection
    let result = detector.detect_duplicates(&batch).unwrap();
    
    // Verify results
    assert_eq!(result.total_duplicates, 2); // Groups A and B have duplicates
    assert_eq!(result.stats.groups_processed, 3); // A, B, C
    assert_eq!(result.stats.blocks_analyzed, 3);
    assert_eq!(result.stats.unique_blocks, 2);
    
    // Check that we found the expected duplicate blocks
    assert_eq!(result.duplicate_blocks.len(), 2);
    
    // Verify group A has 3 occurrences (1 original + 2 duplicates)
    let group_a_block = result.duplicate_blocks.iter()
        .find(|block| block.group_id == "A")
        .unwrap();
    assert_eq!(group_a_block.row_indices.len(), 3);
    
    // Verify group B has 2 occurrences (1 original + 1 duplicate)
    let group_b_block = result.duplicate_blocks.iter()
        .find(|block| block.group_id == "B")
        .unwrap();
    assert_eq!(group_b_block.row_indices.len(), 2);
    
    // Test creating clean Arrow file
    let output_path = Path::new("test_output.arrow");
    let kept_rows = detector.create_clean_arrow_file(&batch, &result, output_path).unwrap();
    
    // Should keep 4 rows (1 from A, 1 from B, 1 from C)
    assert_eq!(kept_rows, 4);
    
    // Clean up
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn test_duplicate_detection_with_small_blocks() {
    // Create a simple test database
    let mut db = Database::open_writable(":memory:").unwrap();
    
    // Create a simple table with duplicates
    db.create_table("test_table", &[
        ("group_id", "VARCHAR"),
        ("name", "VARCHAR"),
        ("value", "INTEGER"),
        ("id", "INTEGER"),
    ]).unwrap();
    
    // Insert test data with duplicates
    let test_data = vec![
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "1".to_string()],
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "2".to_string()],
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "3".to_string()],
        vec!["A".to_string(), "John".to_string(), "100".to_string(), "4".to_string()],
    ];
    
    db.batch_insert("test_table", &test_data).unwrap();
    
    // Get the table data
    let batch = db.get_table_arrow_batch("test_table").unwrap();
    assert_eq!(batch.num_rows(), 4);
    
    // Create detector configuration with small block size
    let config = DuplicateDetectionConfig {
        group_column: "group_id".to_string(),
        ignore_columns: HashSet::from(["id".to_string()]),
        block_size: 2, // Small block size
        null_equals_null: true,
    };
    
    let detector = DuplicateDetector::new(config);
    
    // Run detection
    let result = detector.detect_duplicates(&batch).unwrap();
    
    // Verify results
    assert_eq!(result.total_duplicates, 1); // Group A has duplicates
    assert_eq!(result.stats.groups_processed, 1); // Only A
    assert_eq!(result.stats.blocks_analyzed, 2); // 2 blocks of size 2
    assert_eq!(result.stats.unique_blocks, 1);
    
    // Check that we found the expected duplicate blocks
    assert_eq!(result.duplicate_blocks.len(), 1);
    
    // Verify group A has 4 occurrences
    let group_a_block = result.duplicate_blocks.iter()
        .find(|block| block.group_id == "A")
        .unwrap();
    assert_eq!(group_a_block.row_indices.len(), 4);
} 