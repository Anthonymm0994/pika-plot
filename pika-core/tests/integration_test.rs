use pika_core::{
    error::Result,
    events::{EventBus, CanvasEvent, NodeEvent, AppEvent},
    node::{Node, NodeContext, Port, PortType, PortDirection},
    plots::{PlotConfig, PlotType, PlotDataConfig, ScatterConfig},
    snapshot::{WorkspaceSnapshot, SnapshotBuilder},
    types::{NodeId, TableInfo, ImportOptions, QueryResult},
};
use std::sync::Arc;
use tokio::sync::broadcast;

#[test]
fn test_event_bus_creation() {
    let event_bus = EventBus::new();
    
    // Test sending canvas events
    let canvas_tx = event_bus.canvas_events_sender();
    assert!(canvas_tx.send(CanvasEvent::NodeAdded { id: NodeId::new() }).is_ok());
    
    // Test sending node events
    let node_tx = event_bus.node_events_sender();
    assert!(node_tx.send(NodeEvent::NodeSelected { id: NodeId::new() }).is_ok());
}

#[test]
fn test_event_subscription() {
    let event_bus = EventBus::new();
    
    // Subscribe to canvas events
    let mut canvas_rx = event_bus.subscribe_canvas_events();
    
    // Send an event
    let node_id = NodeId::new();
    let canvas_tx = event_bus.canvas_events_sender();
    canvas_tx.send(CanvasEvent::NodeAdded { id: node_id }).unwrap();
    
    // Receive the event
    match canvas_rx.try_recv() {
        Ok(CanvasEvent::NodeAdded { id }) => assert_eq!(id, node_id),
        _ => panic!("Expected NodeAdded event"),
    }
}

#[test]
fn test_snapshot_serialization() {
    let snapshot = SnapshotBuilder::new()
        .name("Test Workspace")
        .description("Test workspace for unit tests")
        .build();
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&snapshot).unwrap();
    
    // Deserialize back
    let deserialized: WorkspaceSnapshot = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.metadata.name, "Test Workspace");
    assert_eq!(deserialized.metadata.description, Some("Test workspace for unit tests".to_string()));
}

#[test]
fn test_plot_config_creation() {
    let config = PlotConfig {
        plot_type: PlotType::Scatter,
        title: Some("Test Plot".to_string()),
        x_label: Some("X Axis".to_string()),
        y_label: Some("Y Axis".to_string()),
        width: 800,
        height: 600,
        specific: PlotDataConfig::ScatterConfig {
            x_column: "x".to_string(),
            y_column: "y".to_string(),
            size_column: None,
            color_column: None,
        },
    };
    
    assert_eq!(config.plot_type, PlotType::Scatter);
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
}

#[test]
fn test_import_options_default() {
    let options = ImportOptions::default();
    
    assert!(options.has_header);
    assert_eq!(options.delimiter, b',');
    assert!(options.infer_schema);
    assert_eq!(options.sample_size, 1000);
}

#[test]
fn test_table_info_creation() {
    let table_info = TableInfo {
        id: NodeId::new(),
        name: "test_table".to_string(),
        table_name: "test_table".to_string(),
        row_count: 1000,
        columns: vec![
            pika_core::types::ColumnInfo {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            pika_core::types::ColumnInfo {
                name: "value".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: true,
            },
        ],
        size_bytes: 8192,
    };
    
    assert_eq!(table_info.name, "test_table");
    assert_eq!(table_info.row_count, 1000);
    assert_eq!(table_info.columns.len(), 2);
}

#[tokio::test]
async fn test_app_event_flow() {
    let event_bus = EventBus::new();
    let mut app_rx = event_bus.subscribe_app_events();
    let app_tx = event_bus.app_events_sender();
    
    // Send import event
    let path = std::path::PathBuf::from("test.csv");
    let options = ImportOptions::default();
    
    app_tx.send(AppEvent::ImportCsv { 
        path: path.clone(), 
        options: options.clone() 
    }).unwrap();
    
    // Receive the event
    match app_rx.recv().await {
        Ok(AppEvent::ImportCsv { path: p, options: o }) => {
            assert_eq!(p, path);
            assert_eq!(o.delimiter, options.delimiter);
        }
        _ => panic!("Expected ImportCsv event"),
    }
} 