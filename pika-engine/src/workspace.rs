//! Workspace management for saving and loading project state.

use pika_core::{
    error::{PikaError, Result},
    snapshot::WorkspaceSnapshot,
    types::NodeId,
};
use crate::database::Database;
use std::path::Path;
use std::fs;

/// Create a snapshot of the current workspace state.
pub async fn create_snapshot(db: &Database) -> Result<WorkspaceSnapshot> {
    // For now, just create a basic snapshot
    // In a real implementation, we'd capture all relevant state
    let mut snapshot = WorkspaceSnapshot::new();
    
    // Add table information to metadata
    snapshot.metadata.description = Some("Auto-generated snapshot".to_string());
    snapshot.metadata.tags = vec!["auto".to_string()];
    
    Ok(snapshot)
}

/// Save a workspace snapshot to a file.
pub fn save_snapshot(snapshot: &WorkspaceSnapshot, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)
        .map_err(|e| PikaError::Other(format!("Failed to create file: {}", e)))?;
    
    serde_json::to_writer_pretty(file, snapshot)
        .map_err(|e| PikaError::Other(format!("Failed to serialize snapshot: {}", e)))?;
    
    Ok(())
}

/// Load a workspace snapshot from a file.
pub fn load_snapshot(path: &Path) -> Result<WorkspaceSnapshot> {
    let file = std::fs::File::open(path)
        .map_err(|e| PikaError::Other(format!("Failed to open file: {}", e)))?;
    
    let snapshot = serde_json::from_reader(file)
        .map_err(|e| PikaError::Other(format!("Failed to deserialize snapshot: {}", e)))?;
    
    Ok(snapshot)
}

/// Restore workspace state from a snapshot.
pub async fn restore_snapshot(db: &mut Database, snapshot: &WorkspaceSnapshot) -> Result<()> {
    // In a real implementation, we'd restore all state
    // For now, just validate the snapshot
    if snapshot.metadata.version != "1.0.0" {
        return Err(PikaError::UnsupportedVersion(
            snapshot.metadata.version.clone()
        ));
    }
    
    // TODO: Restore tables, queries, plots, etc.
    
    Ok(())
}

/// Get list of tables in the database.
fn get_table_list(db: &Database) -> Result<Vec<String>> {
    let mut stmt = db.prepare(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'main' 
         AND table_name NOT LIKE 'pika_%'"
    )?;
    
    let tables = stmt.query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| PikaError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| PikaError::Database(e.to_string()))?;
    
    Ok(tables)
} 