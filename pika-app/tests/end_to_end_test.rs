use pika_core::{
    events::{EventBus, AppEvent},
    types::{ImportOptions, NodeId},
    plots::{PlotConfig, PlotType},
};
use pika_engine::Engine;
use pika_ui::{
    canvas::CanvasState,
    nodes::{TableNode, QueryNode, PlotNode},
    workspace::{save_workspace, load_workspace},
};
use std::sync::Arc;
use parking_lot::RwLock;
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[test]
fn test_complete_workflow() {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    
    // Create test CSV file
    let csv_path = temp_dir.path().join("test_data.csv");
    std::fs::write(&csv_path, 
        "id,name,value,category\n\
         1,Alice,100,A\n\
         2,Bob,200,B\n\
         3,Charlie,300,A\n\
         4,David,400,B\n\
         5,Eve,500,A"
    ).unwrap();
    
    // Initialize engine
    let engine = rt.block_on(async {
        Engine::new().await.unwrap()
    });
    let engine = Arc::new(RwLock::new(engine));
    
    // Get event bus
    let event_bus = {
        let engine = engine.read();
        engine.event_bus().clone()
    };
    
    // Subscribe to events
    let mut app_rx = event_bus.subscribe_app_events();
    
    // Step 1: Import CSV
    let import_options = ImportOptions::default();
    event_bus.app_events_sender().send(AppEvent::ImportCsv {
        path: csv_path.clone(),
        options: import_options,
    }).unwrap();
    
    // Process import
    rt.block_on(async {
        let mut engine = engine.write();
        engine.process_events().await.unwrap();
    });
    
    // Wait for import complete event
    let table_info = rt.block_on(async {
        loop {
            match app_rx.recv().await {
                Ok(AppEvent::ImportComplete { path: _, table_info }) => {
                    return table_info;
                }
                Ok(_) => continue,
                Err(_) => panic!("Failed to receive import complete event"),
            }
        }
    });
    
    assert_eq!(table_info.row_count, 5);
    assert_eq!(table_info.columns.len(), 4);
    
    // Step 2: Create canvas with nodes
    let mut canvas = CanvasState::new();
    
    // Add table node
    let table_node_id = table_info.id;
    canvas.add_node(table_node_id, egui::pos2(100.0, 100.0));
    
    // Add query node
    let query_node_id = NodeId::new();
    canvas.add_node(query_node_id, egui::pos2(300.0, 100.0));
    
    // Add plot node
    let plot_node_id = NodeId::new();
    canvas.add_node(plot_node_id, egui::pos2(500.0, 100.0));
    
    // Connect nodes
    canvas.add_connection(table_node_id, query_node_id);
    canvas.add_connection(query_node_id, plot_node_id);
    
    // Step 3: Execute query
    let query = "SELECT category, SUM(value) as total FROM test_data GROUP BY category";
    event_bus.app_events_sender().send(AppEvent::ExecuteQuery {
        id: query_node_id,
        query: query.to_string(),
    }).unwrap();
    
    // Process query
    rt.block_on(async {
        let mut engine = engine.write();
        engine.process_events().await.unwrap();
    });
    
    // Wait for query result
    let query_result = rt.block_on(async {
        loop {
            match app_rx.recv().await {
                Ok(AppEvent::QueryComplete { id, result }) if id == query_node_id => {
                    return result.unwrap();
                }
                Ok(_) => continue,
                Err(_) => panic!("Failed to receive query complete event"),
            }
        }
    });
    
    assert_eq!(query_result.row_count, 2); // Two categories: A and B
    
    // Step 4: Configure plot
    let plot_config = PlotConfig::bar("category".to_string(), "total".to_string());
    
    // Step 5: Save workspace
    let workspace_path = temp_dir.path().join("test_workspace.pika");
    let app_state = create_test_app_state(table_info, canvas);
    save_workspace(&app_state, &workspace_path).unwrap();
    
    // Step 6: Load workspace
    let loaded_snapshot = load_workspace(&workspace_path).unwrap();
    assert_eq!(loaded_snapshot.nodes.len(), 1); // Only table nodes are saved in snapshot
    assert_eq!(loaded_snapshot.connections.len(), 2);
    
    // Cleanup is automatic when temp_dir goes out of scope
}

// Helper function to create test app state
fn create_test_app_state(
    table_info: pika_core::types::TableInfo,
    canvas: CanvasState,
) -> pika_ui::state::AppState {
    let mut state = pika_ui::state::AppState::new();
    state.add_data_node(table_info);
    
    // Add connections from canvas
    for conn in canvas.connections() {
        state.connections.push(pika_ui::state::NodeConnection {
            from: conn.from,
            to: conn.to,
            connection_type: pika_ui::state::ConnectionType::DataFlow,
        });
    }
    
    state
}

#[test]
fn test_plot_rendering() {
    use arrow::array::{Float64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;
    
    // Create test data
    let schema = Arc::new(Schema::new(vec![
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("category", DataType::Utf8, false),
    ]));
    
    let x_array = Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let y_array = Float64Array::from(vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    let category_array = StringArray::from(vec!["A", "B", "A", "B", "A"]);
    
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(x_array),
            Arc::new(y_array),
            Arc::new(category_array),
        ],
    ).unwrap();
    
    // Test scatter plot data extraction
    let points = pika_engine::plot::extract_xy_points(&batch, "x", "y").unwrap();
    assert_eq!(points.len(), 5);
    assert_eq!(points[0], (1.0, 2.0));
    assert_eq!(points[4], (5.0, 10.0));
    
    // Test category extraction
    let pairs = pika_engine::plot::extract_category_values(&batch, "category", "y").unwrap();
    assert_eq!(pairs.len(), 5);
    
    let aggregated = pika_engine::plot::aggregate_by_category(pairs);
    assert_eq!(aggregated.get("A"), Some(&18.0)); // 2 + 6 + 10
    assert_eq!(aggregated.get("B"), Some(&12.0)); // 4 + 8
}

#[test]
fn test_memory_management() {
    let rt = Runtime::new().unwrap();
    
    // Create engine with memory limit
    let engine = rt.block_on(async {
        Engine::new_with_limit(Some(512 * 1024 * 1024)).await.unwrap() // 512MB limit
    });
    
    // Check memory info
    let mem_info = engine.memory_coordinator().get_memory_info();
    assert!(mem_info.total_mb > 0);
    assert_eq!(mem_info.total_mb, 512);
    assert!(mem_info.used_mb <= mem_info.total_mb);
    assert!(mem_info.available_mb > 0);
}

#[test]
fn test_export_functionality() {
    use pika_ui::export::{ExportManager, ExportOptions, ExportFormat, CsvExportOptions};
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;
    
    let temp_dir = TempDir::new().unwrap();
    
    // Create test data
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("value", DataType::Int32, false),
    ]));
    
    let id_array = Int32Array::from(vec![1, 2, 3]);
    let value_array = Int32Array::from(vec![10, 20, 30]);
    
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(id_array), Arc::new(value_array)],
    ).unwrap();
    
    // Test CSV export
    let csv_path = temp_dir.path().join("export.csv");
    let export_manager = ExportManager::new();
    
    let options = ExportOptions {
        format: ExportFormat::Csv,
        csv_options: Some(CsvExportOptions {
            delimiter: b',',
            include_header: true,
        }),
        png_options: None,
        json_options: None,
    };
    
    export_manager.export_data(&batch, &csv_path, &options).unwrap();
    
    // Verify CSV was created
    assert!(csv_path.exists());
    let csv_content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(csv_content.contains("id,value"));
    assert!(csv_content.contains("1,10"));
    assert!(csv_content.contains("2,20"));
    assert!(csv_content.contains("3,30"));
} 