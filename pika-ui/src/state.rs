//! Application state management.

use pika_core::{
    types::{NodeId, TableInfo},
    Node,
};
use crate::nodes::TableNode;
use std::collections::HashMap;

/// View mode for the application
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Welcome,
    Canvas,
    DataExplorer,
    QueryEditor,
}

/// Application state
#[derive(Debug)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub selected_node: Option<NodeId>,
    pub data_nodes: Vec<TableNode>,
    pub zoom: f32,
    pub pan: (f32, f32),
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Welcome,
            selected_node: None,
            data_nodes: Vec::new(),
            zoom: 1.0,
            pan: (0.0, 0.0),
        }
    }
    
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }
    
    pub fn select_node(&mut self, node_id: Option<NodeId>) {
        self.selected_node = node_id;
    }
    
    pub fn get_selected_node(&self) -> Option<NodeId> {
        self.selected_node
    }
    
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 5.0);
    }
    
    pub fn set_pan(&mut self, pan: (f32, f32)) {
        self.pan = pan;
    }
    
    /// Add a table to the application state
    pub fn add_table(&mut self, table_info: TableInfo) {
        let node_id = NodeId::new();
        let mut node = TableNode::new(node_id);
        node.table_info = table_info;
        
        self.data_nodes.push(node);
        self.view_mode = ViewMode::Canvas;
    }
} 