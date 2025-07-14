//! Core type definitions used throughout the application.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        NodeId(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortId {
    Input(String),
    Output(String),
}

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Point2 { x, y }
    }
}

/// 2D size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size2 {
    pub width: f32,
    pub height: f32,
}

impl Size2 {
    pub fn new(width: f32, height: f32) -> Self {
        Size2 { width, height }
    }
}

/// Connection between two nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub from: (NodeId, PortId),
    pub to: (NodeId, PortId),
}

/// Camera state for the canvas
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera2D {
    pub center: Point2,
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            center: Point2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

/// Import options for data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub has_header: bool,
    pub delimiter: char,
    pub quote_char: Option<char>,
    pub escape_char: Option<char>,
    pub skip_rows: usize,
    pub max_rows: Option<usize>,
    pub encoding: String,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            has_header: true,
            delimiter: ',',
            quote_char: Some('"'),
            escape_char: Some('\\'),
            skip_rows: 0,
            max_rows: None,
            encoding: "utf-8".to_string(),
        }
    }
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub source_path: Option<PathBuf>,
    pub row_count: Option<usize>,
    pub columns: Vec<ColumnInfo>,
    pub preview_data: Option<TablePreview>,
}

/// Table preview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePreview {
    pub rows: Vec<Vec<String>>,
    pub current_page: usize,
    pub rows_per_page: usize,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Query result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub memory_used_bytes: Option<usize>,
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
    Parquet,
    Excel,
    Png,
    Svg,
    Pdf,
}

/// Window identifier
pub type WindowId = String;

/// Rect type for UI layout
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height }
    }
    
    pub fn contains(&self, point: Point2) -> bool {
        point.x >= self.x && 
        point.x <= self.x + self.width &&
        point.y >= self.y && 
        point.y <= self.y + self.height
    }
} 