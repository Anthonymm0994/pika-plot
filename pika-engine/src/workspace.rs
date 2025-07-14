//! Workspace save/load functionality.

use pika_core::{
    error::{PikaError, Result},
    snapshot::WorkspaceSnapshot,
    workspace::{WorkspaceConfig, Connection},
    types::NodeId,
};

use std::path::Path;
use std::collections::HashMap;

/// Workspace manager for handling workspace operations
pub struct WorkspaceManager {
    config: WorkspaceConfig,
    connections: Vec<Connection>,
    nodes: HashMap<NodeId, serde_json::Value>,
}

impl WorkspaceManager {
    pub fn new(config: WorkspaceConfig) -> Self {
        Self {
            config,
            connections: Vec::new(),
            nodes: HashMap::new(),
        }
    }
    
    /// Save workspace to file
    pub async fn save_workspace<P: AsRef<Path>>(
        &self,
        path: P,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<()> {
        let ron_string = ron::to_string(snapshot)
            .map_err(|e| PikaError::Internal(format!("Failed to serialize to RON: {}", e)))?;
        
        std::fs::write(path, ron_string)
            .map_err(|e| PikaError::Io(e))?;
        
        Ok(())
    }
    
    /// Load workspace from file
    pub async fn load_workspace<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<WorkspaceSnapshot> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PikaError::Io(e))?;
        
        let snapshot: WorkspaceSnapshot = ron::from_str(&content)
            .map_err(|e| PikaError::Import(format!("Failed to parse workspace file: {}", e)))?;
        
        // Check version compatibility
        if snapshot.version != WorkspaceSnapshot::CURRENT_VERSION {
            return Err(PikaError::UnsupportedVersion(format!(
                "Found version {}, expected {}",
                snapshot.version,
                WorkspaceSnapshot::CURRENT_VERSION
            )));
        }
        
        // Validate connections
        for connection in &snapshot.connections {
            if !snapshot.nodes.contains_key(&connection.from_node) {
                return Err(PikaError::Validation(format!(
                    "Invalid connection: from node {} not found", connection.from_node
                )));
            }
            if !snapshot.nodes.contains_key(&connection.to_node) {
                return Err(PikaError::Validation(
                    format!("Connection references non-existent node: {}", connection.to_node)
                ));
            }
        }
        
        Ok(snapshot)
    }
    
    /// Migrate workspace to current version
    pub fn migrate_workspace(_snapshot: &mut WorkspaceSnapshot) -> Result<()> {
        // Migration logic would go here
        Ok(())
    }
    
    /// Add a connection between nodes
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }
    
    /// Remove a connection
    pub fn remove_connection(&mut self, connection_id: &str) {
        self.connections.retain(|c| c.id != connection_id);
    }
    
    /// Get all connections
    pub fn get_connections(&self) -> &[Connection] {
        &self.connections
    }
    
    /// Add a node
    pub fn add_node(&mut self, node_id: NodeId, node_data: serde_json::Value) {
        self.nodes.insert(node_id, node_data);
    }
    
    /// Remove a node
    pub fn remove_node(&mut self, node_id: &NodeId) {
        self.nodes.remove(node_id);
        // Also remove any connections involving this node
        self.connections.retain(|c| c.from_node != *node_id && c.to_node != *node_id);
    }
    
    /// Get all nodes
    pub fn get_nodes(&self) -> &HashMap<NodeId, serde_json::Value> {
        &self.nodes
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new(WorkspaceConfig::default())
    }
}

/// Export workspace data to various formats
pub async fn export_workspace(
    snapshot: &WorkspaceSnapshot,
    path: &Path,
    format: WorkspaceExportFormat,
) -> Result<()> {
    match format {
        WorkspaceExportFormat::Json => {
            let json = serde_json::to_string_pretty(snapshot)?;
            std::fs::write(path, json)?;
            Ok(())
        }
        WorkspaceExportFormat::Ron => {
            let ron = ron::to_string(snapshot)
                .map_err(|e| PikaError::Internal(format!("Failed to serialize to RON: {}", e)))?;
            std::fs::write(path, ron)?;
            Ok(())
        }
    }
}

/// Workspace export formats
#[derive(Debug, Clone, Copy)]
pub enum WorkspaceExportFormat {
    Json,
    Ron,
}

/// Validate a workspace snapshot
pub fn validate_snapshot(snapshot: &WorkspaceSnapshot) -> Result<()> {
    // Check version compatibility
    if snapshot.version > WorkspaceSnapshot::CURRENT_VERSION {
        return Err(PikaError::UnsupportedVersion {
            found: snapshot.version,
            expected: WorkspaceSnapshot::CURRENT_VERSION,
        });
    }
    
    // Validate node connections
    for connection in &snapshot.connections {
        // Check that both nodes exist
        if !snapshot.nodes.contains_key(&connection.from_node) {
            return Err(PikaError::Validation(
                format!("Connection references non-existent source node: {:?}", connection.from_node)
            ));
        }
        if !snapshot.nodes.contains_key(&connection.to_node) {
            return Err(PikaError::Validation(
                format!("Connection references non-existent target node: {:?}", connection.to_node)
            ));
        }
    }
    
    Ok(())
}

/// Migrate old snapshot formats to current version
pub fn migrate_snapshot(snapshot: &mut WorkspaceSnapshot) -> Result<()> {
    // Currently no migrations needed
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pika_core::snapshot::SnapshotBuilder;
    
    #[tokio::test]
    async fn test_workspace_save_load() {
        let snapshot = SnapshotBuilder::new()
            .with_description("Test workspace".to_string())
            .build();
        
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Save
        save_workspace(&snapshot, path).await.unwrap();
        
        // Load
        let loaded = load_workspace(path).await.unwrap();
        
        assert_eq!(loaded.version, snapshot.version);
        assert_eq!(loaded.metadata.description, snapshot.metadata.description);
    }
    
    #[test]
    fn test_validate_snapshot() {
        let snapshot = WorkspaceSnapshot::new();
        assert!(validate_snapshot(&snapshot).is_ok());
    }
} 