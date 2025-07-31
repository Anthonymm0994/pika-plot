use std::sync::Arc;
use fresh::core::{Database, DuplicateDetector, DuplicateDetectionConfig};
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Duplicate Detection Feature");
    println!("=====================================");

    // Test 1: Simple duplicates
    println!("\n📁 Test 1: Simple duplicates (simple_duplicates.csv)");
    test_file("test_data/simple_duplicates.csv")?;

    // Test 2: Mixed data types
    println!("\n📁 Test 2: Mixed data types (mixed_data.csv)");
    test_file("test_data/mixed_data.csv")?;

    // Test 3: Large dataset
    println!("\n📁 Test 3: Large dataset (large_dataset.csv)");
    test_file("test_data/large_dataset.csv")?;

    println!("\n✅ All tests completed successfully!");
    Ok(())
}

fn test_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create database and load CSV
    let mut db = Database::new()?;
    db.load_csv_file(file_path)?;

    // Get table info
    let tables = db.get_tables()?;
    if tables.is_empty() {
        println!("❌ No tables found in {}", file_path);
        return Ok(());
    }

    let table_name = &tables[0].name;
    println!("📊 Table: {} ({} columns)", table_name, tables[0].columns.len());

    // Get available columns
    let available_columns: Vec<String> = tables[0].columns.iter()
        .map(|col| col.name.clone())
        .collect();
    
    println!("📋 Available columns: {:?}", available_columns);

    // Test with different configurations
    let test_configs = vec![
        ("Basic test (group by group_id, ignore id)", 
         DuplicateDetectionConfig {
             group_column: "group_id".to_string(),
             ignore_columns: HashSet::from(["id".to_string()]),
             block_size: 256,
             null_equals_null: true,
         }),
        ("Test with smaller block size", 
         DuplicateDetectionConfig {
             group_column: "group_id".to_string(),
             ignore_columns: HashSet::from(["id".to_string()]),
             block_size: 2,
             null_equals_null: true,
         }),
        ("Test ignoring timestamp columns", 
         DuplicateDetectionConfig {
             group_column: "group_id".to_string(),
             ignore_columns: HashSet::from(["id".to_string(), "timestamp".to_string(), "last_login".to_string(), "hire_date".to_string()]),
             block_size: 256,
             null_equals_null: true,
         }),
    ];

    for (test_name, config) in test_configs {
        println!("\n🔍 {}", test_name);
        
        // Load table data
        let batch = db.get_table_arrow_batch(table_name)?;
        println!("📈 Loaded {} rows", batch.num_rows());

        // Create detector and run detection
        let detector = DuplicateDetector::new(config);
        match detector.detect_duplicates(&batch) {
            Ok(result) => {
                println!("✅ Detection completed successfully");
                println!("   📊 Groups processed: {}", result.stats.groups_processed);
                println!("   📊 Blocks analyzed: {}", result.stats.blocks_analyzed);
                println!("   📊 Unique blocks found: {}", result.stats.unique_blocks);
                println!("   📊 Total duplicate blocks: {}", result.total_duplicates);
                println!("   📊 Total duplicate rows: {}", result.total_duplicate_rows);

                if result.total_duplicates > 0 {
                    println!("   🔍 Found duplicate blocks:");
                    for (i, block) in result.duplicate_blocks.iter().enumerate() {
                        println!("      Block {}: Group '{}', {} occurrences, {} rows each", 
                                i + 1, block.group_id, block.row_indices.len(), block.block_size);
                    }

                    // Test creating clean Arrow file
                    let output_path = format!("test_data/{}_clean.arrow", table_name);
                    match detector.create_clean_arrow_file(&batch, &result, std::path::Path::new(&output_path)) {
                        Ok(kept_rows) => {
                            println!("   💾 Created clean Arrow file: {} (kept {} rows)", output_path, kept_rows);
                        }
                        Err(e) => {
                            println!("   ❌ Failed to create clean Arrow file: {}", e);
                        }
                    }
                } else {
                    println!("   ✅ No duplicates found");
                }
            }
            Err(e) => {
                println!("❌ Detection failed: {}", e);
            }
        }
    }

    Ok(())
} 