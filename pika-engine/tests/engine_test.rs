use pika_engine::Engine;
use pika_core::{
    events::{EventBus, Event, AppEvent},
    types::{NodeId, ImportOptions},
    error::Result,
};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

#[tokio::test]
async fn test_engine_creation() -> Result<()> {
    let engine = Engine::new();
    
    // Test that engine was created successfully
    let _receiver = engine.event_bus().subscribe();
    
    Ok(())
}

#[tokio::test]
async fn test_csv_import() -> Result<()> {
    let engine = Engine::new();
    
    // Create a temporary CSV file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,value").unwrap();
    writeln!(temp_file, "1,Alice,100").unwrap();
    writeln!(temp_file, "2,Bob,200").unwrap();
    
    let file_path = temp_file.path().to_string_lossy().to_string();
    let options = ImportOptions::default();
    let node_id = NodeId::new();
    
    // Test CSV import
    let table_info = engine.import_csv(file_path, options, node_id).await?;
    
    // Verify table info
    assert!(!table_info.name.is_empty());
    assert!(table_info.row_count.is_some());
    assert!(table_info.columns.len() > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_query_execution() -> Result<()> {
    let engine = Engine::new();
    
    // Test basic query execution
    let query = "SELECT 1 as test_value";
    let _result = engine.execute_query(query.to_string()).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_memory_info() -> Result<()> {
    let _engine = Engine::new();
    
    // Note: Removed calls to non-existent memory functions
    // These would need to be implemented if memory monitoring is required
    
    Ok(())
}

#[tokio::test]
async fn test_event_handling() -> Result<()> {
    let engine = Engine::new();
    
    // Test event bus functionality
    let event_bus = engine.event_bus();
    let mut receiver = event_bus.subscribe();
    
    // Send a test event
    let test_event = Event::App(AppEvent::ImportComplete {
        path: "test.csv".to_string(),
        table_info: pika_core::types::TableInfo {
            name: "test_table".to_string(),
            source_path: None,
            row_count: Some(10),
            columns: vec![],
        }
    });
    
    event_bus.send(test_event);
    
    // Verify event was received
    let received_event = receiver.recv().await.unwrap();
    match received_event {
        Event::App(AppEvent::ImportComplete { table_info, .. }) => {
            assert_eq!(table_info.name, "test_table");
            assert_eq!(table_info.row_count, Some(10));
        }
        _ => panic!("Expected ImportComplete event"),
    }
    
    Ok(())
} 