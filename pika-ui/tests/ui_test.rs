use pika_ui::{
    canvas::{CanvasState, Camera2D, BreadcrumbTrail},
    nodes::{TableNode, QueryNode, PlotNode},
    export::{ExportManager, ExportType, ExportFormat},
    workspace::{Workspace, WorkspaceMode},
};
use pika_core::{
    types::{NodeId, TableInfo, ColumnInfo},
    node::{Node, NodeContext, Port, PortType, PortDirection},
};
use egui::{pos2, vec2, Rect};

#[test]
fn test_canvas_state_creation() {
    let mut canvas = CanvasState::new();
    
    assert_eq!(canvas.nodes.len(), 0);
    assert_eq!(canvas.connections.len(), 0);
    assert!(canvas.selected_nodes.is_empty());
}

#[test]
fn test_camera_transform() {
    let camera = Camera2D::new();
    
    // Test world to screen transformation
    let world_pos = pos2(100.0, 100.0);
    let screen_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
    let screen_pos = camera.world_to_screen(world_pos, screen_rect);
    
    // At default zoom and pan, should be centered
    assert_eq!(screen_pos.x, 500.0); // 400 + 100
    assert_eq!(screen_pos.y, 400.0); // 300 + 100
}

#[test]
fn test_breadcrumb_trail() {
    let mut trail = BreadcrumbTrail::new();
    
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();
    
    trail.add_node(node1, "Dataset".to_string());
    trail.add_node(node2, "Query".to_string());
    trail.add_node(node3, "Plot".to_string());
    
    assert_eq!(trail.nodes.len(), 3);
    assert_eq!(trail.nodes[0].1, "Dataset");
    assert_eq!(trail.nodes[2].1, "Plot");
}

#[test]
fn test_table_node_creation() {
    let table_info = TableInfo {
        id: NodeId::new(),
        name: "test_table".to_string(),
        table_name: "test_table".to_string(),
        row_count: 1000,
        columns: vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
        ],
        size_bytes: 8192,
    };
    
    let node = TableNode::new(table_info);
    
    assert_eq!(node.name(), "test_table");
    assert_eq!(node.node_type(), "Table");
    
    let ports = node.ports();
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].direction, PortDirection::Output);
    assert_eq!(ports[0].port_type, PortType::Table);
}

#[test]
fn test_query_node_creation() {
    let node = QueryNode::new(NodeId::new());
    
    assert_eq!(node.node_type(), "Query");
    
    let ports = node.ports();
    assert!(ports.iter().any(|p| p.direction == PortDirection::Input));
    assert!(ports.iter().any(|p| p.direction == PortDirection::Output));
}

#[test]
fn test_plot_node_creation() {
    let node = PlotNode::new(NodeId::new());
    
    assert_eq!(node.node_type(), "Plot");
    
    let ports = node.ports();
    assert_eq!(ports.len(), 2);
    assert!(ports.iter().all(|p| p.direction == PortDirection::Input));
}

#[test]
fn test_export_manager() {
    let manager = ExportManager::new();
    
    // Test format detection
    assert_eq!(
        ExportManager::detect_format(&std::path::PathBuf::from("test.png")),
        Some(ExportFormat::Png)
    );
    assert_eq!(
        ExportManager::detect_format(&std::path::PathBuf::from("data.csv")),
        Some(ExportFormat::Csv)
    );
    assert_eq!(
        ExportManager::detect_format(&std::path::PathBuf::from("plot.svg")),
        Some(ExportFormat::Svg)
    );
}

#[test]
fn test_workspace_mode_switching() {
    let mut workspace = Workspace::new();
    
    assert_eq!(workspace.mode(), WorkspaceMode::Canvas);
    
    workspace.set_mode(WorkspaceMode::Notebook);
    assert_eq!(workspace.mode(), WorkspaceMode::Notebook);
    
    workspace.set_mode(WorkspaceMode::Canvas);
    assert_eq!(workspace.mode(), WorkspaceMode::Canvas);
}

#[test]
fn test_canvas_snapping() {
    let canvas = CanvasState::new();
    
    // Test grid snapping
    let pos = pos2(123.0, 456.0);
    let snapped = canvas.snap_to_grid(pos);
    
    // Should snap to nearest 20 unit grid
    assert_eq!(snapped.x, 120.0);
    assert_eq!(snapped.y, 460.0);
} 