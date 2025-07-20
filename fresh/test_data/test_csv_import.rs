use fresh::core::database::Database;
use std::path::Path;

fn main() {
    // Test the enhanced CSV import functionality
    println!("Testing enhanced CSV import...");
    
    // Create a new database
    let mut db = Database::open_writable("test_db").expect("Failed to create database");
    
    // Test importing the complex CSV file
    let csv_path = Path::new("test_complex.csv");
    
    // Test with skip_lines = 2 and header_row = 1 (0-indexed, so row 2)
    println!("Importing with skip_lines=2, header_row=1 (0-indexed)");
    match db.stream_insert_csv_with_header_row("test_table", csv_path, ',', 1, 2) {
        Ok(_) => {
            println!("✅ Successfully imported CSV with enhanced functionality!");
            
            // Try to query the table to verify it worked
            match db.execute_query("SELECT * FROM test_table LIMIT 5") {
                Ok(results) => {
                    println!("✅ Successfully queried imported data!");
                    println!("Number of rows: {}", results.len());
                    
                    if !results.is_empty() {
                        println!("First row: {:?}", results[0]);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to query imported data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to import CSV: {}", e);
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file("test_db");
} 