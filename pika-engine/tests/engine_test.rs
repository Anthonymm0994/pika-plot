use pika_engine::{Engine, Result};
use pika_core::{
    events::{EventBus, AppEvent},
    types::{ImportOptions, NodeId},
};
use std::path::PathBuf;
use tokio::runtime::Runtime;

#[tokio::test]
async fn test_engine_creation() {
    let engine = Engine::new().await;
    assert!(engine.is_ok());
}

#[tokio::test]
async fn test_database_connection() {
    let engine = Engine::new().await.unwrap();
    
    // Test basic query
    let result = engine.execute_query("SELECT 1 as test").await;
    assert!(result.is_ok());
    
    let query_result = result.unwrap();
    assert_eq!(query_result.row_count, 1);
}

#[tokio::test]
async fn test_csv_import() {
    let engine = Engine::new().await.unwrap();
    
    // Create a test CSV file
    let test_dir = tempfile::tempdir().unwrap();
    let csv_path = test_dir.path().join("test.csv");
    
    std::fs::write(&csv_path, "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300").unwrap();
    
    // Import the CSV
    let result = engine.import_csv(&csv_path, "test_table").await;
    assert!(result.is_ok());
    
    // Query the imported data
    let query_result = engine.execute_query("SELECT COUNT(*) as count FROM test_table").await;
    assert!(query_result.is_ok());
    assert_eq!(query_result.unwrap().row_count, 1);
}

#[tokio::test]
async fn test_memory_coordinator() {
    let engine = Engine::new().await.unwrap();
    
    let mem_info = engine.memory_coordinator().get_memory_info();
    assert!(mem_info.total_mb > 0);
    assert!(mem_info.used_mb <= mem_info.total_mb);
}

#[tokio::test]
async fn test_event_processing() {
    let engine = Engine::new().await.unwrap();
    let event_bus = engine.event_bus();
    
    // Subscribe to app events
    let mut app_rx = event_bus.subscribe_app_events();
    
    // Send a test event
    let app_tx = event_bus.app_events_sender();
    app_tx.send(AppEvent::ClearCache {
        query_cache: true,
        gpu_cache: false,
    }).unwrap();
    
    // Process events
    engine.process_events().await.unwrap();
    
    // Check if event was received
    match app_rx.try_recv() {
        Ok(AppEvent::ClearCache { query_cache, gpu_cache }) => {
            assert!(query_cache);
            assert!(!gpu_cache);
        }
        _ => panic!("Expected ClearCache event"),
    }
}

#[test]
fn test_streaming_config() {
    use pika_engine::streaming::StreamingConfig;
    
    let config = StreamingConfig {
        batch_size: 1000,
        buffer_size_mb: 100,
        flush_interval_ms: 1000,
    };
    
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.buffer_size_mb, 100);
    assert_eq!(config.flush_interval_ms, 1000);
}

#[tokio::test]
async fn test_streaming_buffer() {
    use pika_engine::streaming::StreamingBuffer;
    
    let mut buffer = StreamingBuffer::new(1000);
    
    assert_eq!(buffer.capacity(), 1000);
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
    
    // Test buffer operations
    buffer.clear();
    assert!(buffer.is_empty());
} 