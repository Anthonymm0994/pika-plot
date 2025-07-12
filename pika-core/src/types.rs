//! Core types used throughout the system.

use serde::{Serialize, Deserialize};
use std::time::Duration;
use uuid::Uuid;

// ===== Node System Types =====

/// Unique identifier for nodes in the workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Create a new unique node ID
    pub fn new() -> Self {
        NodeId(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Position in 2D canvas space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Point2 { x, y }
    }
}

/// 2D size type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size2 {
    pub width: f32,
    pub height: f32,
}

impl Size2 {
    /// Create a new size
    pub fn new(width: f32, height: f32) -> Self {
        Size2 { width, height }
    }
}

impl Default for Size2 {
    fn default() -> Self {
        Size2 { width: 200.0, height: 150.0 }
    }
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: (NodeId, PortId),
    pub to: (NodeId, PortId),
}

/// Port identifier for node connections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortId {
    Input(String),
    Output(String),
}

// ===== Data Import Types =====

/// Options for CSV import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub delimiter: u8,  // Changed to u8 for consistency with UI
    pub has_header: bool,
    pub quote_char: char,
    pub escape_char: Option<char>,
    pub null_values: Vec<String>,
    pub sample_size: usize,
    pub infer_schema: bool,  // Renamed from type_inference
    pub encoding: String,
    pub skip_rows: usize,
    pub max_rows: Option<usize>,
    pub create_table: bool,
    pub table_name: String,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            delimiter: b',',
            has_header: true,
            quote_char: '"',
            escape_char: Some('\\'),
            null_values: vec![
                "".to_string(),
                "NULL".to_string(),
                "null".to_string(),
                "N/A".to_string(),
                "NA".to_string(),
                "n/a".to_string(),
            ],
            sample_size: 10000,
            infer_schema: true,
            encoding: "utf-8".to_string(),
            skip_rows: 0,
            max_rows: None,
            create_table: true,
            table_name: String::new(),
        }
    }
}

/// Information about an imported table.
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub id: NodeId,
    pub name: String,
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: usize,
    pub estimated_size: u64, // in bytes
}

/// Column information.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

// ===== Query & Results Types =====

/// Result of a SQL query execution
/// The actual data is stored as an opaque pointer to avoid
/// direct arrow dependency in core. The engine layer will handle it.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Opaque data handle - engine-specific
    pub data_handle: std::sync::Arc<dyn std::any::Any + Send + Sync>,
    pub execution_time: Duration,
    pub row_count: usize,
    pub memory_usage: usize, // in bytes
    pub execution_time_ms: f64,
    /// Column names for UI display
    pub column_names: Vec<String>,
    /// Column types as strings for UI display  
    pub column_types: Vec<String>,
}

/// Fingerprint for query caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryFingerprint(String);

impl QueryFingerprint {
    /// Create a new query fingerprint from SQL
    pub fn new(sql: &str) -> Self {
        // Simple normalization: lowercase and trim whitespace
        QueryFingerprint(sql.trim().to_lowercase())
    }
}

// ===== 2D Camera for canvas navigation =====

/// 2D camera for canvas navigation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera2D {
    pub center: Point2,
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            center: Point2 { x: 0.0, y: 0.0 },
            zoom: 1.0,
        }
    }
}

// ===== Export Format Types =====

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    // Images
    Png {
        width: u32,
        height: u32,
        dpi: u32,
    },
    Svg {
        width: u32,
        height: u32,
        embed_fonts: bool,
    },
    
    // Data
    Csv {
        delimiter: char,
        header: bool,
    },
    Json {
        pretty: bool,
    },
    
    // Workspace
    PikaPlot {
        version: u32,
    },
}

/// Memory usage information
#[derive(Debug, Clone, Copy)]
pub struct MemoryStatus {
    pub total_physical: u64,
    pub available_physical: u64,
    pub total_virtual: u64,
    pub available_virtual: u64,
} 