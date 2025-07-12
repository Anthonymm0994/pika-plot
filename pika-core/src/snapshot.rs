//! Snapshot system for saving and loading workspace state.

use serde::{Serialize, Deserialize};
use crate::{
    types::{NodeId, Point2, Size2, Connection, ExportFormat},
    plots::SimplePlotConfig,
    nodes::{CanvasNode, NodeType},
    error::{PikaError, Result},
    WindowId,
};
use std::collections::HashMap;
use std::path::Path;
use chrono::{DateTime, Utc};

/// Main snapshot structure containing the entire workspace state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    
    /// Canvas state
    pub canvas: CanvasSnapshot,
    
    /// All nodes in the workspace
    pub nodes: Vec<NodeSnapshot>,
    
    /// All connections between nodes
    pub connections: Vec<ConnectionSnapshot>,
    
    /// Window layout configuration
    pub windows: Vec<WindowSnapshot>,
    
    /// User preferences and settings
    pub preferences: PreferencesSnapshot,
}

/// Metadata about the snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Version of the snapshot format
    pub version: String,
    
    /// When the snapshot was created
    pub created_at: DateTime<Utc>,
    
    /// Last modification time
    pub modified_at: DateTime<Utc>,
    
    /// Application version that created this snapshot
    pub app_version: String,
    
    /// Optional user description
    pub description: Option<String>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Canvas viewport and camera state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    /// Camera center position
    pub camera_center: Point2,
    
    /// Current zoom level
    pub zoom_level: f32,
    
    /// Grid configuration
    pub grid_config: GridConfig,
    
    /// Selected nodes
    pub selected_nodes: Vec<NodeId>,
}

/// Grid and snapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub grid_size: f32,
    pub snap_enabled: bool,
    pub snap_threshold: f32,
    pub show_grid: bool,
}

/// Node state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    /// Unique node identifier
    pub id: NodeId,
    
    /// Node position on canvas
    pub position: Point2,
    
    /// Node size
    pub size: Size2,
    
    /// Whether the node is collapsed
    pub collapsed: bool,
    
    /// Node-specific data
    pub data: NodeDataSnapshot,
}

/// Node-specific data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeDataSnapshot {
    Table {
        table_name: String,
        file_path: Option<String>,
        row_count: usize,
        columns: Vec<ColumnInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_data: Option<Vec<HashMap<String, serde_json::Value>>>,
    },
    Query {
        sql: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cached_result_preview: Option<QueryResultPreview>,
    },
    Plot {
        config: SimplePlotConfig,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_render_settings: Option<RenderSettings>,
    },
    Transform {
        transform_type: crate::nodes::TransformType,
        parameters: HashMap<String, serde_json::Value>,
    },
    Export {
        format: ExportFormat,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_path: Option<String>,
        settings: HashMap<String, serde_json::Value>,
    },
}

/// Column information for table nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Query result preview for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultPreview {
    pub row_count: usize,
    pub columns: Vec<String>,
    pub preview_rows: Vec<HashMap<String, serde_json::Value>>,
}

/// Render settings for plot nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub theme: String,
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSnapshot {
    pub from_node: NodeId,
    pub from_port: String,
    pub to_node: NodeId,
    pub to_port: String,
}

/// Window layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSnapshot {
    pub id: WindowId,
    pub node_id: Option<NodeId>,
    pub position: Point2,
    pub size: Size2,
    pub docked: bool,
    pub dock_position: Option<String>,
}

/// User preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesSnapshot {
    pub theme: String,
    pub auto_save_enabled: bool,
    pub auto_save_interval_seconds: u64,
    pub default_plot_settings: HashMap<String, serde_json::Value>,
    pub recent_files: Vec<String>,
}

impl WorkspaceSnapshot {
    /// Create a new workspace snapshot
    pub fn new() -> Self {
        WorkspaceSnapshot {
            metadata: SnapshotMetadata {
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                description: None,
                tags: Vec::new(),
            },
            canvas: CanvasSnapshot {
                camera_center: Point2 { x: 0.0, y: 0.0 },
                zoom_level: 1.0,
                grid_config: GridConfig {
                    grid_size: 20.0,
                    snap_enabled: true,
                    snap_threshold: 10.0,
                    show_grid: true,
                },
                selected_nodes: Vec::new(),
            },
            nodes: Vec::new(),
            connections: Vec::new(),
            windows: Vec::new(),
            preferences: PreferencesSnapshot {
                theme: "dark".to_string(),
                auto_save_enabled: true,
                auto_save_interval_seconds: 300,
                default_plot_settings: HashMap::new(),
                recent_files: Vec::new(),
            },
        }
    }
    
    /// Save snapshot to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;
        
        serde_json::to_writer_pretty(file, self)
            .map_err(|e| PikaError::Serialization(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load snapshot from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        
        let snapshot = serde_json::from_reader(file)
            .map_err(|e| PikaError::Deserialization(e.to_string()))?;
        
        Ok(snapshot)
    }
    
    /// Export to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PikaError::Serialization(e.to_string()))
    }
    
    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| PikaError::Deserialization(e.to_string()))
    }
    
    /// Add a node to the snapshot
    pub fn add_node(&mut self, node: &CanvasNode) {
        let data = match &node.node_type {
            NodeType::Table(data) => NodeDataSnapshot::Table {
                table_name: data.table_name.clone(),
                file_path: Some(data.source_path.display().to_string()),
                row_count: data.row_count.unwrap_or(0),
                columns: Vec::new(), // TODO: Populate from schema
                sample_data: None,
            },
            NodeType::Query(data) => NodeDataSnapshot::Query {
                sql: data.sql.clone(),
                cached_result_preview: None,
            },
            NodeType::Plot(data) => NodeDataSnapshot::Plot {
                config: data.config.clone(),
                last_render_settings: None,
            },
            NodeType::Transform(data) => NodeDataSnapshot::Transform {
                transform_type: data.transform_type.clone(),
                parameters: HashMap::new(), // TODO: Serialize parameters
            },
            NodeType::Export(data) => NodeDataSnapshot::Export {
                format: data.format.clone(),
                output_path: data.destination.as_ref()
                    .map(|path| path.display().to_string()),
                settings: HashMap::new(), // TODO: Serialize settings
            },
        };
        
        self.nodes.push(NodeSnapshot {
            id: node.id,
            position: node.position,
            size: node.size,
            collapsed: false, // Canvas nodes don't have collapsed state, using default
            data,
        });
    }
    
    /// Add a connection to the snapshot
    pub fn add_connection(&mut self, connection: &Connection) {
        self.connections.push(ConnectionSnapshot {
            from_node: connection.from.0,
            from_port: match &connection.from.1 {
                crate::types::PortId::Input(name) => name.clone(),
                crate::types::PortId::Output(name) => name.clone(),
            },
            to_node: connection.to.0,
            to_port: match &connection.to.1 {
                crate::types::PortId::Input(name) => name.clone(),
                crate::types::PortId::Output(name) => name.clone(),
            },
        });
    }
}

/// Builder for creating snapshots
pub struct SnapshotBuilder {
    snapshot: WorkspaceSnapshot,
}

impl SnapshotBuilder {
    /// Create a new snapshot builder
    pub fn new() -> Self {
        SnapshotBuilder {
            snapshot: WorkspaceSnapshot::new(),
        }
    }
    
    /// Set snapshot description
    pub fn with_description(mut self, description: String) -> Self {
        self.snapshot.metadata.description = Some(description);
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.snapshot.metadata.tags = tags;
        self
    }
    
    /// Set canvas state
    pub fn with_canvas_state(mut self, center: Point2, zoom: f32) -> Self {
        self.snapshot.canvas.camera_center = center;
        self.snapshot.canvas.zoom_level = zoom;
        self
    }
    
    /// Add nodes
    pub fn with_nodes(mut self, nodes: Vec<&CanvasNode>) -> Self {
        for node in nodes {
            self.snapshot.add_node(node);
        }
        self
    }
    
    /// Add connections
    pub fn with_connections(mut self, connections: Vec<&Connection>) -> Self {
        for conn in connections {
            self.snapshot.add_connection(conn);
        }
        self
    }
    
    /// Build the snapshot
    pub fn build(self) -> WorkspaceSnapshot {
        self.snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plots::{PlotType, PlotTheme};
    
    #[test]
    fn test_snapshot_serialization() {
        let snapshot = WorkspaceSnapshot::new();
        
        // Serialize to JSON
        let json = snapshot.to_json().unwrap();
        
        // Deserialize back
        let deserialized = WorkspaceSnapshot::from_json(&json).unwrap();
        
        assert_eq!(snapshot.metadata.version, deserialized.metadata.version);
        assert_eq!(snapshot.canvas.zoom_level, deserialized.canvas.zoom_level);
    }
    
    #[test]
    fn test_snapshot_builder() {
        let snapshot = SnapshotBuilder::new()
            .with_description("Test workspace".to_string())
            .with_tags(vec!["test".to_string(), "example".to_string()])
            .with_canvas_state(Point2 { x: 100.0, y: 200.0 }, 1.5)
            .build();
        
        assert_eq!(snapshot.metadata.description, Some("Test workspace".to_string()));
        assert_eq!(snapshot.metadata.tags.len(), 2);
        assert_eq!(snapshot.canvas.camera_center.x, 100.0);
        assert_eq!(snapshot.canvas.zoom_level, 1.5);
    }
    
    #[test]
    fn test_node_snapshot_serialization() {
        let node_snapshot = NodeSnapshot {
            id: NodeId::new(),
            position: Point2 { x: 50.0, y: 100.0 },
            size: Size2 { width: 200.0, height: 150.0 },
            collapsed: false,
            data: NodeDataSnapshot::Plot {
                config: SimplePlotConfig {
                    plot_type: PlotType::Scatter,
                    title: Some("Test Plot".to_string()),
                    x_label: Some("X Axis".to_string()),
                    y_label: Some("Y Axis".to_string()),
                    width: 800,
                    height: 600,
                    theme: PlotTheme::Dark,
                    show_legend: true,
                    show_grid: true,
                },
                last_render_settings: Some(RenderSettings {
                    width: 800,
                    height: 600,
                    dpi: 96.0,
                    theme: "dark".to_string(),
                }),
            },
        };
        
        let json = serde_json::to_string(&node_snapshot).unwrap();
        let deserialized: NodeSnapshot = serde_json::from_str(&json).unwrap();
        
        assert_eq!(node_snapshot.position.x, deserialized.position.x);
        
        match deserialized.data {
            NodeDataSnapshot::Plot { config, .. } => {
                assert_eq!(config.title, Some("Test Plot".to_string()));
            }
            _ => panic!("Expected Plot node"),
        }
    }
} 