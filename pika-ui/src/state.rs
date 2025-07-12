//! Application state management.

use std::collections::HashMap;
use pika_core::{
    types::{NodeId, TableInfo, QueryResult},
};
use egui::pos2;

/// View mode for the main canvas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Canvas,
    Grid,
}

/// Main application state.
pub struct AppState {
    /// Currently loaded data nodes
    pub data_nodes: HashMap<NodeId, DataNode>,
    
    /// Currently selected node
    pub selected_node: Option<NodeId>,
    
    /// View state
    pub camera_zoom: f32,
    pub camera_pan: [f32; 2],
    
    /// Current view mode
    pub view_mode: ViewMode,
    
    /// UI visibility flags
    pub show_properties: bool,
    pub show_data_panel: bool,
    pub show_memory_warning: bool,
    pub show_memory_dialog: bool,
    pub show_about_dialog: bool,
    
    /// Canvas connections between nodes
    pub connections: Vec<NodeConnection>,
}

/// Represents a data node in the workspace.
pub struct DataNode {
    pub id: NodeId,
    pub name: String,
    pub table_info: TableInfo,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub last_query_result: Option<QueryResult>,
}

/// Connection between two nodes
pub struct NodeConnection {
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    DataFlow,
    Transform,
    Join,
}

impl AppState {
    /// Create new application state.
    pub fn new() -> Self {
        Self {
            data_nodes: HashMap::new(),
            selected_node: None,
            camera_zoom: 1.0,
            camera_pan: [0.0, 0.0],
            view_mode: ViewMode::Canvas,
            show_properties: true,
            show_data_panel: true,
            show_memory_warning: false,
            show_memory_dialog: false,
            show_about_dialog: false,
            connections: Vec::new(),
        }
    }
    
    /// Add a new data node from imported table info.
    pub fn add_data_node(&mut self, table_info: TableInfo) {
        let node = DataNode {
            id: table_info.id,
            name: table_info.name.clone(),
            table_info,
            position: pos2(100.0 + (self.data_nodes.len() as f32 * 150.0), 100.0),
            size: egui::vec2(200.0, 150.0),
            last_query_result: None,
        };
        
        self.data_nodes.insert(node.id, node);
    }
    
    /// Update query result for a node.
    pub fn update_query_result(&mut self, node_id: NodeId, result: QueryResult) {
        if let Some(node) = self.data_nodes.get_mut(&node_id) {
            node.last_query_result = Some(result);
        }
    }
} 