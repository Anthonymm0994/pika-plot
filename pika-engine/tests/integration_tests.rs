//! Integration tests for the pika-engine crate.

use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use pika_engine::{Engine, Database, QueryEngine, PlotRenderer, GpuManager};
use pika_core::{
    types::{ImportOptions, NodeId},
    plots::{PlotConfig, PlotType, PlotDataConfig, MarkerShape, LineInterpolation},
    events::EventBus,
};

/// Test the complete workflow: CSV import -> query -> plot
#[tokio::test]
async fn test_complete_workflow() {
    // Create a temporary CSV file for testing
    let csv_content = "id,name,value,category\n1,Alice,10.5,A\n2,Bob,20.3,B\n3,Charlie,15.7,A\n4,Diana,25.1,B\n";
    let temp_file = "test_data.csv";
    fs::write(temp_file, csv_content).await.unwrap();
    
    // Create database and query engine directly
    let db = Arc::new(tokio::sync::Mutex::new(Database::new().await.unwrap()));
    let query_engine = QueryEngine::new(db.clone());
    
    // Import CSV manually using DuckDB's COPY command
    let import_sql = format!("CREATE TABLE test_data AS SELECT * FROM read_csv_auto('{}')", temp_file);
    {
        let database = db.lock().await;
        database.execute(&import_sql).await.unwrap();
    }
    
    // Verify import worked
    let count_result = query_engine.execute("SELECT COUNT(*) as count FROM test_data").await.unwrap();
    assert_eq!(count_result.row_count, 1); // One row with the count
    
    // Test query execution
    let query_result = query_engine.execute("SELECT * FROM test_data WHERE value > 15").await.unwrap();
    assert_eq!(query_result.row_count, 3); // Bob, Charlie, Diana
    assert_eq!(query_result.columns.len(), 4);
    
    // Test plot configuration (without actual rendering)
    let plot_config = PlotConfig::scatter("value".to_string(), "id".to_string());
    assert_eq!(plot_config.plot_type, PlotType::Scatter);
    
    // Cleanup
    fs::remove_file(temp_file).await.unwrap_or(());
}

/// Test database operations
#[tokio::test]
async fn test_database_operations() {
    let db = Database::new().await.unwrap();
    
    // Test table creation and data insertion
    db.execute("CREATE TABLE test_table (id INTEGER, name VARCHAR, score DOUBLE)").await.unwrap();
    db.execute("INSERT INTO test_table VALUES (1, 'Alice', 95.5), (2, 'Bob', 87.2), (3, 'Charlie', 92.8)").await.unwrap();
    
    // Test scalar query
    let count: i64 = db.query_scalar("SELECT COUNT(*) FROM test_table").await.unwrap();
    assert_eq!(count, 3);
    
    // Test average calculation
    let avg_score: f64 = db.query_scalar("SELECT AVG(score) FROM test_table").await.unwrap();
    assert!((avg_score - 91.83333333333333).abs() < 0.001);
    
    // Test query with mapping
    let results = db.query_map("SELECT name, score FROM test_table ORDER BY score DESC", |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    }).await.unwrap();
    
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], ("Alice".to_string(), 95.5));
    assert_eq!(results[1], ("Charlie".to_string(), 92.8));
    assert_eq!(results[2], ("Bob".to_string(), 87.2));
}

/// Test query engine functionality
#[tokio::test]
async fn test_query_engine() {
    use tokio::sync::Mutex;
    
    let db = Arc::new(Mutex::new(Database::new().await.unwrap()));
    let query_engine = QueryEngine::new(db.clone());
    
    // Setup test data
    {
        let database = db.lock().await;
        database.execute("CREATE TABLE products (id INTEGER, name VARCHAR, price DOUBLE, category VARCHAR)").await.unwrap();
        database.execute("INSERT INTO products VALUES 
            (1, 'Laptop', 999.99, 'Electronics'),
            (2, 'Book', 19.99, 'Books'),
            (3, 'Phone', 699.99, 'Electronics'),
            (4, 'Desk', 299.99, 'Furniture')").await.unwrap();
    }
    
    // Test query execution
    let result = query_engine.execute("SELECT * FROM products WHERE price > 500").await.unwrap();
    assert_eq!(result.row_count, 2); // Laptop and Phone
    assert_eq!(result.columns.len(), 4);
    assert!(result.execution_time_ms > 0);
    
    // Test query validation
    assert!(query_engine.validate("SELECT name, price FROM products").await.is_ok());
    assert!(query_engine.validate("INVALID SQL QUERY").await.is_err());
    
    // Test aggregation query
    let agg_result = query_engine.execute("SELECT category, COUNT(*), AVG(price) FROM products GROUP BY category").await.unwrap();
    assert_eq!(agg_result.row_count, 3); // Electronics, Books, Furniture
    assert_eq!(agg_result.columns.len(), 3);
}

/// Test plot configuration and rendering setup
#[tokio::test]
async fn test_plot_configurations() {
    // Test different plot types
    let scatter_config = PlotConfig::scatter("x".to_string(), "y".to_string());
    assert_eq!(scatter_config.plot_type, PlotType::Scatter);
    
    let line_config = PlotConfig::line("time".to_string(), "value".to_string());
    assert_eq!(line_config.plot_type, PlotType::Line);
    
    let bar_config = PlotConfig::bar("category".to_string(), "count".to_string());
    assert_eq!(bar_config.plot_type, PlotType::Bar);
    
    let histogram_config = PlotConfig::histogram("values".to_string());
    assert_eq!(histogram_config.plot_type, PlotType::Histogram);
    
    // Test plot data configurations
    match scatter_config.specific {
        PlotDataConfig::ScatterConfig { x_column, y_column, point_radius, marker_shape, .. } => {
            assert_eq!(x_column, "x");
            assert_eq!(y_column, "y");
            assert_eq!(point_radius, 3.0);
            assert_eq!(marker_shape, MarkerShape::Circle);
        }
        _ => panic!("Expected ScatterConfig"),
    }
    
    match line_config.specific {
        PlotDataConfig::LineConfig { x_column, y_column, line_width, interpolation, .. } => {
            assert_eq!(x_column, "time");
            assert_eq!(y_column, "value");
            assert_eq!(line_width, 2.0);
            assert_eq!(interpolation, LineInterpolation::Linear);
        }
        _ => panic!("Expected LineConfig"),
    }
}

/// Test plot renderer initialization
#[tokio::test]
async fn test_plot_renderer() {
    // Test renderer creation without GPU
    let _renderer = PlotRenderer::new(None);
    assert!(true); // Just verify it doesn't panic
    
    // Test renderer creation with GPU (if available)
    let gpu_manager = GpuManager::new().await.ok().map(Arc::new);
    let _renderer_with_gpu = PlotRenderer::new(gpu_manager);
    assert!(true); // Just verify it doesn't panic
}

/// Test memory management and limits
#[tokio::test]
async fn test_memory_management() {
    let db = Database::new().await.unwrap();
    
    // Test setting memory limit
    let _result = db.set_memory_limit(100 * 1024 * 1024).await; // 100MB
    // Note: This might fail on some systems, so we don't assert
    
    // Create a large table to test memory usage
    db.execute("CREATE TABLE large_table AS SELECT i as id, 'data_' || i as data FROM range(1000) t(i)").await.unwrap();
    
    let count: i64 = db.query_scalar("SELECT COUNT(*) FROM large_table").await.unwrap();
    assert_eq!(count, 1000);
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    use tokio::sync::Mutex;
    
    let db = Arc::new(Mutex::new(Database::new().await.unwrap()));
    
    // Setup test table
    {
        let database = db.lock().await;
        database.execute("CREATE TABLE concurrent_test (id INTEGER, value INTEGER)").await.unwrap();
    }
    
    // Run concurrent inserts
    let mut handles = vec![];
    for i in 0..5 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let database = db_clone.lock().await;
            database.execute(&format!("INSERT INTO concurrent_test VALUES ({}, {})", i, i * 10)).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all inserts to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all rows were inserted
    let count: i64 = {
        let database = db.lock().await;
        database.query_scalar("SELECT COUNT(*) FROM concurrent_test").await.unwrap()
    };
    assert_eq!(count, 5);
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let db = Database::new().await.unwrap();
    
    // Test invalid SQL
    let result = db.execute("INVALID SQL STATEMENT");
    assert!(result.await.is_err());
    
    // Test querying non-existent table
    let result = db.query_scalar::<i64>("SELECT COUNT(*) FROM non_existent_table");
    assert!(result.await.is_err());
    
    // Test invalid column access
    db.execute("CREATE TABLE error_test (id INTEGER)").await.unwrap();
    let result = db.query_scalar::<String>("SELECT non_existent_column FROM error_test");
    assert!(result.await.is_err());
}

/// Test event bus functionality
#[tokio::test]
async fn test_event_bus() {
    use pika_core::events::{Event, AppEvent};
    
    let event_bus = Arc::new(EventBus::new(100));
    let mut receiver = event_bus.subscribe();
    
    // Send an event
    event_bus.send(Event::App(AppEvent::Started));
    
    // Receive the event
    let received_event = receiver.recv().await.unwrap();
    match received_event {
        Event::App(AppEvent::Started) => {
            // Success
        }
        _ => panic!("Unexpected event type"),
    }
}

/// Test workspace snapshot functionality
#[tokio::test]
async fn test_workspace_snapshots() {
    use pika_core::{
        snapshot::{WorkspaceSnapshot, SnapshotBuilder},
        types::Point2,
    };
    
    // Create a snapshot
    let snapshot = SnapshotBuilder::new()
        .with_description("Test snapshot".to_string())
        .with_canvas_state(Point2::new(100.0, 200.0), 1.5)
        .build();
    
    assert_eq!(snapshot.metadata.description, Some("Test snapshot".to_string()));
    assert_eq!(snapshot.canvas.camera_position, Point2::new(100.0, 200.0));
    assert_eq!(snapshot.canvas.camera_zoom, 1.5);
    
    // Test serialization
    let serialized = serde_json::to_string(&snapshot).unwrap();
    let deserialized: WorkspaceSnapshot = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.metadata.description, snapshot.metadata.description);
    assert_eq!(deserialized.canvas.camera_zoom, snapshot.canvas.camera_zoom);
} 