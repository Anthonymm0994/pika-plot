use pika_core::{
    snapshot::{WorkspaceSnapshot, SnapshotBuilder},
    types::NodeId,
    Result, PikaError,
};
use crate::{
    state::AppState,
    canvas::CanvasState,
    nodes::{TableNode, QueryNode, PlotNode},
};
use std::path::Path;
use std::fs;

/// Save the current workspace to a file
pub fn save_workspace(state: &AppState, path: &Path) -> Result<()> {
    let snapshot = create_snapshot(state)?;
    
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| PikaError::Internal(format!("Failed to serialize workspace: {}", e)))?;
    
    fs::write(path, json)
        .map_err(|e| PikaError::Export(format!("Failed to write workspace file: {}", e)))?;
    
    Ok(())
}

/// Load a workspace from a file
pub fn load_workspace(path: &Path) -> Result<WorkspaceSnapshot> {
    let json = fs::read_to_string(path)
        .map_err(|e| PikaError::FileReadError(format!("Failed to read workspace file: {}", e)))?;
    
    let snapshot: WorkspaceSnapshot = serde_json::from_str(&json)
        .map_err(|e| PikaError::Internal(format!("Failed to deserialize workspace: {}", e)))?;
    
    // Validate version
    if snapshot.version > 1 {
        return Err(PikaError::UnsupportedVersion {
            found: snapshot.version,
            expected: 1,
        });
    }
    
    Ok(snapshot)
}

/// Create a snapshot from the current application state
fn create_snapshot(state: &AppState) -> Result<WorkspaceSnapshot> {
    let mut builder = SnapshotBuilder::new()
        .name("Pika-Plot Workspace")
        .description("Saved workspace");
    
    // Add canvas state
    if let Some(canvas_state) = get_canvas_state(state) {
        builder = builder.canvas_state(canvas_state);
    }
    
    // Add nodes
    for (id, node) in &state.data_nodes {
        let node_snapshot = pika_core::snapshot::NodeSnapshot {
            id: *id,
            node_type: "TableNode".to_string(),
            position: [node.position.x, node.position.y],
            size: [node.size.x, node.size.y],
            data: serde_json::json!({
                "name": node.name,
                "table_info": node.table_info,
            }),
        };
        builder = builder.add_node(node_snapshot);
    }
    
    // Add connections
    for connection in &state.connections {
        let conn_snapshot = pika_core::snapshot::ConnectionSnapshot {
            id: format!("{}-{}", connection.from, connection.to),
            from_node: connection.from,
            to_node: connection.to,
            from_port: "output".to_string(),
            to_port: "input".to_string(),
            connection_type: format!("{:?}", connection.connection_type),
        };
        builder = builder.add_connection(conn_snapshot);
    }
    
    // Add preferences
    let preferences = serde_json::json!({
        "theme": "dark",
        "show_grid": true,
        "snap_to_grid": true,
        "auto_save": false,
    });
    builder = builder.preferences(preferences);
    
    Ok(builder.build())
}

/// Apply a snapshot to the application state
pub fn apply_snapshot(state: &mut AppState, snapshot: WorkspaceSnapshot) -> Result<()> {
    // Clear current state
    state.data_nodes.clear();
    state.connections.clear();
    state.selected_node = None;
    
    // Apply canvas state
    if let Some(canvas_state) = snapshot.canvas_state {
        state.camera_zoom = canvas_state.zoom;
        state.camera_pan = [canvas_state.pan[0], canvas_state.pan[1]];
    }
    
    // Recreate nodes
    for node_snapshot in snapshot.nodes {
        // For now, we only support table nodes in snapshots
        if node_snapshot.node_type == "TableNode" {
            if let Ok(table_info) = serde_json::from_value(node_snapshot.data["table_info"].clone()) {
                let mut node = crate::state::DataNode {
                    id: node_snapshot.id,
                    name: node_snapshot.data["name"].as_str().unwrap_or("Unknown").to_string(),
                    table_info,
                    position: egui::pos2(node_snapshot.position[0], node_snapshot.position[1]),
                    size: egui::vec2(node_snapshot.size[0], node_snapshot.size[1]),
                    last_query_result: None,
                };
                
                state.data_nodes.insert(node.id, node);
            }
        }
    }
    
    // Recreate connections
    for conn in snapshot.connections {
        let connection = crate::state::NodeConnection {
            from: conn.from_node,
            to: conn.to_node,
            connection_type: match conn.connection_type.as_str() {
                "DataFlow" => crate::state::ConnectionType::DataFlow,
                "Transform" => crate::state::ConnectionType::Transform,
                "Join" => crate::state::ConnectionType::Join,
                _ => crate::state::ConnectionType::DataFlow,
            },
        };
        state.connections.push(connection);
    }
    
    Ok(())
}

/// Get canvas state from app state
fn get_canvas_state(state: &AppState) -> Option<pika_core::snapshot::CanvasState> {
    Some(pika_core::snapshot::CanvasState {
        zoom: state.camera_zoom,
        pan: [state.camera_pan[0], state.camera_pan[1]],
        selected_nodes: state.selected_node.map(|id| vec![id]).unwrap_or_default(),
        grid_visible: true,
        snap_to_grid: true,
    })
}

/// Auto-save functionality
pub struct AutoSave {
    enabled: bool,
    interval_seconds: u64,
    last_save: std::time::Instant,
    save_path: Option<std::path::PathBuf>,
}

impl AutoSave {
    pub fn new() -> Self {
        Self {
            enabled: false,
            interval_seconds: 300, // 5 minutes
            last_save: std::time::Instant::now(),
            save_path: None,
        }
    }
    
    pub fn enable(&mut self, path: std::path::PathBuf) {
        self.enabled = true;
        self.save_path = Some(path);
        self.last_save = std::time::Instant::now();
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn check_and_save(&mut self, state: &AppState) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let elapsed = self.last_save.elapsed().as_secs();
        if elapsed >= self.interval_seconds {
            if let Some(path) = &self.save_path {
                save_workspace(state, path)?;
                self.last_save = std::time::Instant::now();
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pika_core::types::TableInfo;
    
    #[test]
    fn test_workspace_save_load() {
        let mut state = AppState::new();
        
        // Add a test node
        let table_info = TableInfo {
            name: "test_table".to_string(),
            source_path: None,
            row_count: Some(100),
            columns: vec![],
            preview_data: None,
        };
        state.add_data_node(table_info);
        
        // Save to temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let save_path = temp_dir.path().join("test_workspace.json");
        
        save_workspace(&state, &save_path).unwrap();
        
        // Load back
        let snapshot = load_workspace(&save_path).unwrap();
        
        assert_eq!(snapshot.nodes.len(), 1);
        assert_eq!(snapshot.version, 1);
    }
} 