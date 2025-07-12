//! Integration tests for end-to-end workflows.

use pika_core::{
    error::Result,
    events::{EventBus, NodeEvent},
    nodes::{CanvasNode, NodeType, TableNodeData, QueryNodeData, PlotNodeData},
    plots::PlotConfig,
    types::{NodeId, Point2, Size2, PlotType},
    snapshot::{WorkspaceSnapshot, SnapshotBuilder},
};
use pika_engine::{Engine, import::CsvImportConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_csv_to_plot_workflow() -> Result<()> {
    // Create engine
    let engine = Arc::new(RwLock::new(Engine::new().await?));
    
    // Create event bus
    let event_bus = Arc::new(EventBus::new());
    
    // Test data path
    let csv_path = PathBuf::from("../test_data/sample_sales.csv");
    
    // Step 1: Import CSV
    println!("Step 1: Importing CSV...");
    let table_name = {
        let mut engine = engine.write().await;
        let config = CsvImportConfig {
            delimiter: b',',
            has_header: true,
            quote_char: Some(b'"'),
            encoding: "utf-8".to_string(),
            type_inference: true,
            sample_size: 1000,
            columns: None,
        };
        
        engine.import_csv(&csv_path, "sales_data", config).await?
    };
    
    println!("  ✓ Imported table: {}", table_name);
    
    // Step 2: Execute query to aggregate data
    println!("Step 2: Running aggregation query...");
    let query = "SELECT Category, SUM(Revenue) as TotalRevenue FROM sales_data GROUP BY Category ORDER BY TotalRevenue DESC";
    
    let query_result = {
        let mut engine = engine.write().await;
        engine.execute_query(query).await?
    };
    
    println!("  ✓ Query returned {} rows", query_result.num_rows());
    
    // Step 3: Create nodes for visualization
    println!("Step 3: Creating canvas nodes...");
    
    let table_node = CanvasNode {
        id: NodeId::new(),
        position: Point2 { x: 100.0, y: 200.0 },
        size: Size2 { x: 150.0, y: 100.0 },
        collapsed: false,
        node_type: NodeType::Table(TableNodeData {
            table_name: table_name.clone(),
            file_path: Some(csv_path.clone()),
            row_count: 10, // From our sample data
        }),
    };
    
    let query_node = CanvasNode {
        id: NodeId::new(),
        position: Point2 { x: 350.0, y: 200.0 },
        size: Size2 { x: 200.0, y: 120.0 },
        collapsed: false,
        node_type: NodeType::Query(QueryNodeData {
            sql: query.to_string(),
            result: Some(query_result),
        }),
    };
    
    let plot_node = CanvasNode {
        id: NodeId::new(),
        position: Point2 { x: 650.0, y: 200.0 },
        size: Size2 { x: 180.0, y: 150.0 },
        collapsed: false,
        node_type: NodeType::Plot(PlotNodeData {
            config: PlotConfig {
                plot_type: PlotType::Bar,
                title: Some("Revenue by Category".to_string()),
                x_label: Some("Category".to_string()),
                y_label: Some("Total Revenue ($)".to_string()),
                width: 800,
                height: 600,
                theme: Default::default(),
                show_legend: false,
                show_grid: true,
            },
            input_data: None,
        }),
    };
    
    // Send events
    let _ = event_bus.send_node_event(NodeEvent::DatasetLoaded {
        node_id: table_node.id,
        table_name: table_name.clone(),
        row_count: 10,
    });
    
    let _ = event_bus.send_node_event(NodeEvent::NodeExecutionCompleted {
        node_id: query_node.id,
        success: true,
        execution_time: std::time::Duration::from_millis(50),
    });
    
    // Step 4: Save snapshot
    println!("Step 4: Saving workspace snapshot...");
    
    let snapshot = SnapshotBuilder::new()
        .with_description("Sales analysis workflow".to_string())
        .with_tags(vec!["test".to_string(), "sales".to_string()])
        .with_canvas_state(Point2 { x: 400.0, y: 200.0 }, 1.0)
        .with_nodes(vec![&table_node, &query_node, &plot_node])
        .build();
    
    let json = snapshot.to_json()?;
    println!("  ✓ Snapshot size: {} bytes", json.len());
    
    // Verify we can reload the snapshot
    let loaded = WorkspaceSnapshot::from_json(&json)?;
    assert_eq!(loaded.nodes.len(), 3);
    assert_eq!(loaded.metadata.description, Some("Sales analysis workflow".to_string()));
    
    println!("\n✅ End-to-end workflow completed successfully!");
    
    Ok(())
}

#[tokio::test]
async fn test_edge_case_empty_csv() -> Result<()> {
    // Create engine
    let engine = Arc::new(RwLock::new(Engine::new().await?));
    
    // Create empty CSV
    let empty_csv = "Column1,Column2,Column3\n";
    let temp_path = std::env::temp_dir().join("empty_test.csv");
    std::fs::write(&temp_path, empty_csv)?;
    
    // Try to import
    let result = {
        let mut engine = engine.write().await;
        let config = CsvImportConfig::default();
        engine.import_csv(&temp_path, "empty_table", config).await
    };
    
    // Should succeed but with 0 rows
    assert!(result.is_ok());
    
    // Clean up
    std::fs::remove_file(temp_path)?;
    
    Ok(())
}

#[tokio::test]
async fn test_edge_case_invalid_query() -> Result<()> {
    // Create engine
    let engine = Arc::new(RwLock::new(Engine::new().await?));
    
    // Execute invalid query
    let result = {
        let mut engine = engine.write().await;
        engine.execute_query("SELECT * FROM non_existent_table").await
    };
    
    // Should return an error
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test] 
async fn test_large_csv_import() -> Result<()> {
    // Create engine
    let engine = Arc::new(RwLock::new(Engine::new().await?));
    
    // Generate large CSV (1MB+)
    let mut csv_content = String::from("ID,Value,Category\n");
    for i in 0..50000 {
        csv_content.push_str(&format!("{},{}.,Category{}\n", i, i as f64 * 1.5, i % 10));
    }
    
    let temp_path = std::env::temp_dir().join("large_test.csv");
    std::fs::write(&temp_path, csv_content)?;
    
    // Import with monitoring
    let start = std::time::Instant::now();
    
    let result = {
        let mut engine = engine.write().await;
        let config = CsvImportConfig::default();
        engine.import_csv(&temp_path, "large_table", config).await
    };
    
    let duration = start.elapsed();
    println!("Large CSV import took: {:?}", duration);
    
    assert!(result.is_ok());
    assert!(duration.as_secs() < 10, "Import took too long");
    
    // Clean up
    std::fs::remove_file(temp_path)?;
    
    Ok(())
} 