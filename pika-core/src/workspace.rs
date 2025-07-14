//! Workspace management types and functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::NodeId;
use crate::nodes::NodeConnection;

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub description: Option<String>,
    pub auto_save: bool,
    pub auto_save_interval_secs: u64,
    pub recent_files: Vec<PathBuf>,
    pub preferences: WorkspacePreferences,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        WorkspaceConfig {
            name: "Untitled Workspace".to_string(),
            description: None,
            auto_save: true,
            auto_save_interval_secs: 300, // 5 minutes
            recent_files: Vec::new(),
            preferences: WorkspacePreferences::default(),
        }
    }
}

/// Workspace preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePreferences {
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub auto_layout: bool,
    pub show_minimap: bool,
    pub connection_style: ConnectionStyle,
}

impl Default for WorkspacePreferences {
    fn default() -> Self {
        WorkspacePreferences {
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            auto_layout: false,
            show_minimap: true,
            connection_style: ConnectionStyle::Bezier,
        }
    }
}

/// Connection line style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStyle {
    Straight,
    Bezier,
    Orthogonal,
}

/// Workspace state that can be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub canvas_nodes: HashMap<NodeId, NodeConnection>,
    pub connections: Vec<NodeConnection>,
    pub camera_position: (f32, f32),
    pub camera_zoom: f32,
    pub selected_nodes: Vec<NodeId>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        WorkspaceState {
            canvas_nodes: HashMap::new(),
            connections: Vec::new(),
            camera_position: (0.0, 0.0),
            camera_zoom: 1.0,
            selected_nodes: Vec::new(),
        }
    }
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub from_node: NodeId,
    pub from_port: String,
    pub to_node: NodeId,
    pub to_port: String,
}

/// Workspace mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceMode {
    Canvas,
    Notebook,
    Split,
}

impl Default for WorkspaceMode {
    fn default() -> Self {
        WorkspaceMode::Canvas
    }
} 