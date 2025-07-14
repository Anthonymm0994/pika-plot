//! Application state management.

use pika_core::{
    types::{NodeId, TableInfo},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Canvas,
    Notebook,
}

#[derive(Debug, Clone)]
pub struct DataNode {
    pub id: NodeId,
    pub table_info: TableInfo,
}

#[derive(Debug)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub selected_node: Option<NodeId>,
    pub data_nodes: Vec<DataNode>,
    pub zoom: f32,
    pub pan: (f32, f32),
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Canvas,
            selected_node: None,
            data_nodes: Vec::new(),
            zoom: 1.0,
            pan: (0.0, 0.0),
        }
    }
    
    pub fn add_data_node(&mut self, table_info: TableInfo) {
        let node = DataNode {
            id: NodeId::new(),
            table_info,
        };
        self.data_nodes.push(node);
    }
    
    pub fn remove_data_node(&mut self, id: NodeId) {
        self.data_nodes.retain(|node| node.id != id);
        if self.selected_node == Some(id) {
            self.selected_node = None;
        }
    }
    
    pub fn get_data_node(&self, id: NodeId) -> Option<&DataNode> {
        self.data_nodes.iter().find(|node| node.id == id)
    }
} 