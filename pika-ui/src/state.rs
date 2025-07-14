//! Application state management.

use pika_core::{
    types::{NodeId, TableInfo, QueryResult},
    plots::PlotConfig,
    nodes::Node,
};
use egui;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Canvas,
    Notebook,
    FileConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Pan,
    Rectangle,
    Circle,
    Line,
    Draw,
    Text,
}

#[derive(Debug, Clone)]
pub struct DataNode {
    pub id: NodeId,
    pub table_info: TableInfo,
    pub position: egui::Vec2,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub id: String,
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    DataFlow,
    Transform,
    Join,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

/// Canvas state for zoom and pan
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub zoom: f32,
    pub show_grid: bool,
    pub pan_offset: egui::Vec2,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            show_grid: true,
            pan_offset: egui::Vec2::ZERO,
        }
    }
}

// Canvas node for visualization
#[derive(Debug, Clone)]
pub struct CanvasNode {
    pub id: NodeId,
    pub position: egui::Vec2,
    pub size: egui::Vec2,
    pub node_type: CanvasNodeType,
}

#[derive(Debug, Clone)]
pub enum CanvasNodeType {
    Table { table_info: TableInfo },
    Plot { plot_type: String },
    Note { content: String },
    Shape { shape_type: ShapeType },
}

#[derive(Debug, Clone)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line { end: egui::Vec2 },
    Arrow { end: egui::Vec2 },
}

/// Main application state
#[derive(Debug)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub tool_mode: ToolMode,
    pub selected_node: Option<NodeId>,
    pub data_nodes: Vec<DataNode>,
    pub connections: Vec<NodeConnection>,
    pub query_results: HashMap<NodeId, QueryResult>,
    pub theme: Theme,
    pub show_import_dialog: bool,
    pub show_export_dialog: bool,
    pub notification: Option<String>,
    pub plot_configs: HashMap<NodeId, PlotConfig>,
    pub canvas_state: CanvasState,
    pub tables: Vec<TableInfo>, // Convenience alias for data sources
    pub canvas_nodes: HashMap<NodeId, CanvasNode>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Canvas,
            tool_mode: ToolMode::Select,
            selected_node: None,
            data_nodes: vec![],
            connections: vec![],
            query_results: HashMap::new(),
            theme: Theme::Dark,
            show_import_dialog: false,
            show_export_dialog: false,
            notification: None,
            plot_configs: HashMap::new(),
            canvas_state: CanvasState::default(),
            tables: vec![],
            canvas_nodes: HashMap::new(),
        }
    }

    /// Update tables list from data nodes
    pub fn update_tables(&mut self) {
        self.tables = self.data_nodes.iter()
            .map(|node| node.table_info.clone())
            .collect();
    }

    /// Add a new data node
    pub fn add_data_node(&mut self, table: TableInfo) -> NodeId {
        let node_id = NodeId(uuid::Uuid::new_v4());
        let node = DataNode {
            id: node_id,
            table_info: table.clone(),
            position: egui::Vec2::new(100.0, 100.0),
            selected: false,
        };
        self.data_nodes.push(node);
        
        // Also add canvas node
        let canvas_node = CanvasNode {
            id: node_id,
            position: egui::Vec2::new(100.0, 100.0),
            size: egui::Vec2::new(200.0, 150.0),
            node_type: CanvasNodeType::Table { table_info: table },
        };
        self.canvas_nodes.insert(node_id, canvas_node);
        
        self.update_tables();
        node_id
    }

    /// Remove a data node
    pub fn remove_data_node(&mut self, id: NodeId) {
        self.data_nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from != id && c.to != id);
        self.query_results.remove(&id);
        self.plot_configs.remove(&id);
        self.canvas_nodes.remove(&id);
        self.update_tables();
    }

    pub fn get_data_node(&self, id: NodeId) -> Option<&DataNode> {
        self.data_nodes.iter().find(|n| n.id == id)
    }

    pub fn add_connection(&mut self, from: NodeId, to: NodeId, connection_type: ConnectionType) {
        let connection = NodeConnection {
            id: format!("{}-{}", from.0, to.0),
            from,
            to,
            connection_type,
        };
        self.connections.push(connection);
    }

    pub fn get_canvas_node(&self, id: NodeId) -> Option<&CanvasNode> {
        self.canvas_nodes.get(&id)
    }

    pub fn get_canvas_node_mut(&mut self, id: NodeId) -> Option<&mut CanvasNode> {
        self.canvas_nodes.get_mut(&id)
    }
} 