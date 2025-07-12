//! Workspace save/load functionality.

use std::path::Path;
use pika_core::{
    error::{PikaError, Result},
    snapshot::WorkspaceSnapshot,
};

/// Save a workspace snapshot to a file
pub async fn save_workspace(snapshot: &WorkspaceSnapshot, path: &Path) -> Result<()> {
    snapshot.save_to_file(path)
}

/// Load a workspace snapshot from a file
pub async fn load_workspace(path: &Path) -> Result<WorkspaceSnapshot> {
    WorkspaceSnapshot::load_from_file(path)
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
                .map_err(|e| anyhow::anyhow!("Failed to serialize to RON: {}", e))?;
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
            return Err(PikaError::InvalidConnection(
                format!("Connection references non-existent source node: {:?}", connection.from_node)
            ));
        }
        if !snapshot.nodes.contains_key(&connection.to_node) {
            return Err(PikaError::InvalidConnection(
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