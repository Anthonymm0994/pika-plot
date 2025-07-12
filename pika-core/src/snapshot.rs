//! Workspace snapshot functionality for saving and loading state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{
    types::{NodeId, Point2, Size2},
    nodes::CanvasNode,
    workspace::Connection,
};

/// Workspace snapshot for saving/loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    /// Snapshot format version
    pub version: u32,
    
    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
    
    /// Canvas state
    pub canvas: CanvasSnapshot,
    
    /// All nodes in the workspace
    pub nodes: HashMap<NodeId, CanvasNode>,
    
    /// All connections between nodes
    pub connections: Vec<Connection>,
    
    /// Window configurations
    pub windows: Vec<WindowSnapshot>,
    
    /// User preferences
    pub preferences: PreferencesSnapshot,
}

/// Metadata about when and how the snapshot was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub app_version: String,
    pub description: Option<String>,
}

/// Canvas state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    pub camera_position: Point2,
    pub camera_zoom: f32,
    pub selected_nodes: Vec<NodeId>,
    pub grid_visible: bool,
    pub grid_size: f32,
}

/// Window configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSnapshot {
    pub id: String,
    pub window_type: WindowType,
    pub position: Point2,
    pub size: Size2,
    pub visible: bool,
}

/// Types of windows that can be saved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    DataExplorer,
    QueryEditor,
    PlotConfig,
    Properties,
}

/// User preferences snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesSnapshot {
    pub theme: String,
    pub auto_save: bool,
    pub show_minimap: bool,
    pub snap_to_grid: bool,
}

impl WorkspaceSnapshot {
    /// Current snapshot version
    pub const CURRENT_VERSION: u32 = 1;
    
    /// Create a new snapshot
    pub fn new() -> Self {
        WorkspaceSnapshot {
            version: Self::CURRENT_VERSION,
            metadata: SnapshotMetadata {
                created_at: chrono::Utc::now(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                description: None,
            },
            canvas: CanvasSnapshot {
                camera_position: Point2::new(0.0, 0.0),
                camera_zoom: 1.0,
                selected_nodes: Vec::new(),
                grid_visible: true,
                grid_size: 20.0,
            },
            nodes: HashMap::new(),
            connections: Vec::new(),
            windows: Vec::new(),
            preferences: PreferencesSnapshot {
                theme: "dark".to_string(),
                auto_save: true,
                show_minimap: true,
                snap_to_grid: true,
            },
        }
    }
    
    /// Save snapshot to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), crate::error::PikaError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load snapshot from file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, crate::error::PikaError> {
        let json = std::fs::read_to_string(path)?;
        let snapshot: WorkspaceSnapshot = serde_json::from_str(&json)?;
        
        // Check version compatibility
        if snapshot.version > Self::CURRENT_VERSION {
            return Err(crate::error::PikaError::UnsupportedVersion {
                found: snapshot.version,
                expected: Self::CURRENT_VERSION,
            });
        }
        
        Ok(snapshot)
    }
}

/// Builder for creating snapshots
pub struct SnapshotBuilder {
    snapshot: WorkspaceSnapshot,
}

impl SnapshotBuilder {
    pub fn new() -> Self {
        SnapshotBuilder {
            snapshot: WorkspaceSnapshot::new(),
        }
    }
    
    pub fn with_description(mut self, desc: String) -> Self {
        self.snapshot.metadata.description = Some(desc);
        self
    }
    
    pub fn with_canvas_state(mut self, position: Point2, zoom: f32) -> Self {
        self.snapshot.canvas.camera_position = position;
        self.snapshot.canvas.camera_zoom = zoom;
        self
    }
    
    pub fn add_node(mut self, node: CanvasNode) -> Self {
        self.snapshot.nodes.insert(node.id.clone(), node);
        self
    }
    
    pub fn add_connection(mut self, conn: Connection) -> Self {
        self.snapshot.connections.push(conn);
        self
    }
    
    pub fn build(self) -> WorkspaceSnapshot {
        self.snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_serialization() {
        let snapshot = WorkspaceSnapshot::new();
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: WorkspaceSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.version, deserialized.version);
    }
    
    #[test]
    fn test_snapshot_builder() {
        let snapshot = SnapshotBuilder::new()
            .with_description("Test snapshot".to_string())
            .with_canvas_state(Point2::new(100.0, 200.0), 1.5)
            .build();
        
        assert_eq!(snapshot.metadata.description, Some("Test snapshot".to_string()));
        assert_eq!(snapshot.canvas.camera_zoom, 1.5);
    }
} 