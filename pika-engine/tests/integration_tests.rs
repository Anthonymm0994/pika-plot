//! Integration tests for the pika-engine crate.

use pika_core::{
    error::Result,
    types::{NodeId, ImportOptions},
};
use pika_engine::{Engine, import::DataImporter};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

#[tokio::test]
async fn test_engine_integration() -> Result<()> {
    let engine = Engine::new();
    
    // Test that engine was created successfully
    let event_bus = engine.event_bus();
    let _receiver = event_bus.subscribe();
    
    // Test basic functionality
    assert!(true); // Engine creation succeeded if we get here
    
    Ok(())
}

#[tokio::test]
async fn test_data_importer_integration() -> Result<()> {
    let importer = DataImporter::new();
    
    // Create a test CSV file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,value").unwrap();
    writeln!(temp_file, "1,Test,123").unwrap();
    writeln!(temp_file, "2,Data,456").unwrap();
    
    let file_path = temp_file.path();
    let config = pika_engine::import::CsvImportConfig::default();
    
    // Test CSV import
    let table_info = importer.import_csv(file_path, config).await?;
    
    // Verify results
    assert!(!table_info.name.is_empty());
    assert_eq!(table_info.row_count, Some(2)); // 2 data rows
    assert_eq!(table_info.columns.len(), 3); // id, name, value
    
    Ok(())
}

#[tokio::test]
async fn test_engine_csv_workflow() -> Result<()> {
    let engine = Engine::new();
    
    // Create a test CSV file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "product,price,quantity").unwrap();
    writeln!(temp_file, "Apple,1.50,100").unwrap();
    writeln!(temp_file, "Orange,2.00,50").unwrap();
    
    let file_path = temp_file.path().to_string_lossy().to_string();
    let options = ImportOptions::default();
    let node_id = NodeId::new();
    
    // Test full workflow
    let table_info = engine.import_csv(file_path, options, node_id).await?;
    
    // Verify table structure
    assert!(!table_info.name.is_empty());
    assert_eq!(table_info.row_count, Some(2));
    assert_eq!(table_info.columns.len(), 3);
    
    // Verify column names
    let column_names: Vec<&str> = table_info.columns.iter()
        .map(|col| col.name.as_str())
        .collect();
    assert!(column_names.contains(&"product"));
    assert!(column_names.contains(&"price"));
    assert!(column_names.contains(&"quantity"));
    
    Ok(())
} 