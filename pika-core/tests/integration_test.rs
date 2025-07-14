use pika_core::{
    error::Result,
    events::{EventBus, Event, AppEvent},
    plots::{PlotConfig, PlotType, PlotDataConfig, MarkerShape, LineInterpolation},
    types::{TableInfo, ColumnInfo, NodeId, ImportOptions},
    snapshot::{WorkspaceSnapshot, SnapshotBuilder},
    nodes::{CanvasNode, NodeType},
};

#[tokio::test]
async fn test_events() -> Result<()> {
    let event_bus = EventBus::new(100);
    let mut receiver = event_bus.subscribe();
    
    let table_info = TableInfo {
        name: "test_table".to_string(),
        source_path: None,
        row_count: Some(100),
        columns: vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "integer".to_string(),
                nullable: false,
            }
        ],
    };
    
    let event = Event::App(AppEvent::ImportComplete { 
        path: "test.csv".to_string(), 
        table_info: table_info.clone() 
    });
    event_bus.send(event);
    
    let received = receiver.recv().await.map_err(|e| pika_core::error::PikaError::Internal(e.to_string()))?;
    match received {
        Event::App(AppEvent::ImportComplete { table_info: received_info, .. }) => {
            assert_eq!(received_info.name, table_info.name);
        }
        _ => panic!("Expected ImportComplete event"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_snapshot() -> Result<()> {
    let snapshot = SnapshotBuilder::new()
        .with_description("Test workspace".to_string())
        .build();
    
    assert!(snapshot.metadata.description.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_plot_config() -> Result<()> {
    let config = PlotConfig {
        plot_type: PlotType::Line,
        title: Some("Time Series".to_string()),
        x_label: Some("Time".to_string()),
        y_label: Some("Value".to_string()),
        width: 800,
        height: 600,
        dark_mode: false,
        specific: PlotDataConfig::LineConfig {
            x_column: "time".to_string(),
            y_column: "value".to_string(),
            color_column: None,
            line_width: 2.0,
            show_points: false,
            interpolation: LineInterpolation::Linear,
        },
    };
    
    // Test that config was created successfully
    assert_eq!(config.plot_type, PlotType::Line);
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert_eq!(config.title, Some("Time Series".to_string()));
    
    // Test specific config
    match config.specific {
        PlotDataConfig::LineConfig { x_column, y_column, .. } => {
            assert_eq!(x_column, "time");
            assert_eq!(y_column, "value");
        }
        _ => panic!("Expected LineConfig"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_import_options() -> Result<()> {
    let options = ImportOptions {
        has_header: true,
        delimiter: ',',
        quote_char: Some('"'),
        escape_char: None,
        skip_rows: 0,
        max_rows: Some(1000),
        encoding: "utf-8".to_string(),
    };
    
    assert!(options.has_header);
    assert_eq!(options.delimiter, ',');
    assert_eq!(options.max_rows, Some(1000));
    
    Ok(())
}

#[tokio::test]
async fn test_table_info() -> Result<()> {
    let table_info = TableInfo {
        name: "test_table".to_string(),
        source_path: Some(std::path::PathBuf::from("test.csv")),
        row_count: Some(1000),
        columns: vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "integer".to_string(),
                nullable: false,
            },
            ColumnInfo {
                name: "name".to_string(),
                data_type: "string".to_string(),
                nullable: true,
            },
        ],
    };
    
    assert_eq!(table_info.name, "test_table");
    assert_eq!(table_info.row_count, Some(1000));
    assert_eq!(table_info.columns.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_event_bus() -> Result<()> {
    let event_bus = EventBus::new(10);
    let mut receiver = event_bus.subscribe();
    
    let test_event = Event::App(AppEvent::ImportComplete {
        path: "test.csv".to_string(),
        table_info: TableInfo {
            name: "test".to_string(),
            source_path: None,
            row_count: Some(5),
            columns: vec![],
        }
    });
    
    event_bus.send(test_event);
    
    let received = receiver.recv().await.map_err(|e| pika_core::error::PikaError::Internal(e.to_string()))?;
    match received {
        Event::App(AppEvent::ImportComplete { table_info, .. }) => {
            assert_eq!(table_info.name, "test");
            assert_eq!(table_info.row_count, Some(5));
        }
        _ => panic!("Expected ImportComplete event"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_canvas_node() -> Result<()> {
    let table_info = TableInfo {
        name: "test_table".to_string(),
        source_path: None,
        row_count: Some(100),
        columns: vec![],
    };
    
    let node = CanvasNode {
        id: NodeId::new(),
        node_type: NodeType::DataSource { table_info },
        position: (100.0, 200.0),
        size: (300.0, 400.0),
        selected: false,
    };
    
    assert_eq!(node.position, (100.0, 200.0));
    assert_eq!(node.size, (300.0, 400.0));
    assert!(!node.selected);
    
    Ok(())
}

#[tokio::test]
async fn test_node_types() -> Result<()> {
    let table_info = TableInfo {
        name: "users".to_string(),
        source_path: None,
        row_count: Some(100),
        columns: vec![],
    };
    
    let data_node = NodeType::DataSource { table_info };
    let query_node = NodeType::Query { query: "SELECT * FROM users".to_string(), result: None };
    let plot_node = NodeType::Plot { config: PlotConfig::scatter("x".to_string(), "y".to_string()) };
    
    // Test that different node types can be created
    match data_node {
        NodeType::DataSource { table_info } => assert_eq!(table_info.name, "users"),
        _ => panic!("Expected DataSource node"),
    }
    
    match query_node {
        NodeType::Query { query, .. } => assert_eq!(query, "SELECT * FROM users"),
        _ => panic!("Expected Query node"),
    }
    
    match plot_node {
        NodeType::Plot { config } => assert_eq!(config.plot_type, PlotType::Scatter),
        _ => panic!("Expected Plot node"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_plot_config_scatter() -> Result<()> {
    let config = PlotConfig {
        plot_type: PlotType::Scatter,
        title: Some("Test Plot".to_string()),
        x_label: Some("X".to_string()),
        y_label: Some("Y".to_string()),
        width: 400,
        height: 300,
        dark_mode: false,
        specific: PlotDataConfig::ScatterConfig {
            x_column: "x".to_string(),
            y_column: "y".to_string(),
            color_column: None,
            point_radius: 3.0,
            marker_shape: MarkerShape::Circle,
            size_column: None,
        },
    };
    
    assert_eq!(config.plot_type, PlotType::Scatter);
    assert_eq!(config.title, Some("Test Plot".to_string()));
    
    match config.specific {
        PlotDataConfig::ScatterConfig { x_column, y_column, marker_shape, .. } => {
            assert_eq!(x_column, "x");
            assert_eq!(y_column, "y");
            assert_eq!(marker_shape, MarkerShape::Circle);
        }
        _ => panic!("Expected ScatterConfig"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_workspace_snapshot() -> Result<()> {
    let snapshot = SnapshotBuilder::new()
        .with_description("Test workspace snapshot".to_string())
        .build();
    
    assert!(snapshot.metadata.description.is_some());
    assert_eq!(snapshot.metadata.description.unwrap(), "Test workspace snapshot");
    
    Ok(())
} 