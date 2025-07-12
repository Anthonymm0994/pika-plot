//! Workspace management types.

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::types::{NodeId, Camera2D, Connection, ImportOptions, Point2};
use crate::nodes::{CanvasNode, NotebookCell};
use crate::plots::PlotTheme;

/// UI mode for the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceMode {
    /// Linear notebook interface
    Notebook {
        cells: Vec<NotebookCell>,
        active_cell: Option<usize>,
    },
    /// Free-form canvas interface
    Canvas {
        nodes: HashMap<NodeId, CanvasNode>,
        connections: Vec<Connection>,
        camera: Camera2D,
    },
}

impl Default for WorkspaceMode {
    fn default() -> Self {
        // Notebook mode is the default per requirements
        WorkspaceMode::Notebook {
            cells: Vec::new(),
            active_cell: None,
        }
    }
}

/// Workspace snapshot for saving/loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    /// Snapshot format version
    pub version: u32,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Current workspace mode and content
    pub mode: WorkspaceMode,
    /// Referenced data sources (not embedded)
    pub data_sources: Vec<DataSourceRef>,
    /// UI theme
    pub theme: PlotTheme,
    /// Memory limit in bytes (if set)
    pub memory_limit: Option<usize>,
}

impl WorkspaceSnapshot {
    /// Current snapshot version
    pub const CURRENT_VERSION: u32 = 1;
    
    /// Create a new snapshot
    pub fn new(mode: WorkspaceMode, data_sources: Vec<DataSourceRef>, theme: PlotTheme) -> Self {
        WorkspaceSnapshot {
            version: Self::CURRENT_VERSION,
            created_at: chrono::Utc::now(),
            mode,
            data_sources,
            theme,
            memory_limit: None,
        }
    }
}

/// Reference to a data source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRef {
    /// Original file path
    pub original_path: PathBuf,
    /// Table name in DuckDB
    pub table_name: String,
    /// Import options used
    pub import_options: ImportOptions,
    /// SHA256 hash of the file
    pub file_hash: String,
    /// File size in bytes
    pub file_size: u64,
    /// Last modified timestamp
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

/// Workspace state that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Current mode
    pub mode: WorkspaceMode,
    /// Window size
    pub window_size: Point2,
    /// Theme
    pub theme: PlotTheme,
    /// Recent files
    pub recent_files: Vec<PathBuf>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        WorkspaceState {
            mode: WorkspaceMode::default(),
            window_size: Point2::new(1280.0, 720.0),
            theme: PlotTheme::default(),
            recent_files: Vec::new(),
        }
    }
} 