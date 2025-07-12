//! Node system for the visual programming interface.

use std::path::PathBuf;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::types::{NodeId, PortId, ImportOptions, QueryResult};
use crate::plots::SimplePlotConfig;
use crate::events::PlotRenderData;

/// Trait for all node types in the system
pub trait Node: Send + Sync {
    /// Get the unique ID of this node
    fn id(&self) -> NodeId;
    
    /// Get the type name of this node (for display)
    fn node_type(&self) -> &'static str;
    
    /// Get the display title of this node
    fn title(&self) -> String;
    
    /// Get input ports for this node
    fn inputs(&self) -> Vec<PortId>;
    
    /// Get output ports for this node
    fn outputs(&self) -> Vec<PortId>;
    
    /// Check if a connection can be made between ports
    fn can_connect(&self, from_port: &PortId, to_node: &dyn Node, to_port: &PortId) -> bool;
}

/// Node types that can appear in canvas mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Table(TableNodeData),
    Query(QueryNodeData),
    Plot(PlotNodeData),
    Transform(TransformNodeData),
    Export(ExportNodeData),
}

/// Cell types that can appear in notebook mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    Table(TableNodeData),
    Query(QueryNodeData),
    Plot(PlotNodeData),
    Markdown(String),
}

/// Table node data - represents an imported data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNodeData {
    pub source_path: PathBuf,
    pub table_name: String,
    pub import_options: ImportOptions,
    /// Column information (name, type) pairs
    pub columns: Option<Vec<(String, String)>>,
    pub row_count: Option<usize>,
}

impl TableNodeData {
    /// Create a new table node
    pub fn new(path: PathBuf, table_name: String, options: ImportOptions) -> Self {
        TableNodeData {
            source_path: path,
            table_name,
            import_options: options,
            columns: None,
            row_count: None,
        }
    }
}

/// Query node data - executes SQL queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryNodeData {
    pub sql: String,
    pub input_tables: Vec<String>,
    #[serde(skip)]
    pub cached_result: Option<Arc<QueryResult>>,
}

impl QueryNodeData {
    /// Create a new query node
    pub fn new(sql: String) -> Self {
        QueryNodeData {
            sql,
            input_tables: Vec::new(),
            cached_result: None,
        }
    }
}

/// Plot node data - visualizes data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotNodeData {
    pub config: SimplePlotConfig,
    pub source_node: Option<NodeId>,
    #[serde(skip)]
    pub cached_data: Option<Arc<PlotRenderData>>,
}

impl PlotNodeData {
    /// Create a new plot node
    pub fn new(config: SimplePlotConfig) -> Self {
        PlotNodeData {
            config,
            source_node: None,
            cached_data: None,
        }
    }
}

/// Transform node data - for data transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformNodeData {
    pub transform_type: TransformType,
    pub parameters: serde_json::Value,
}

/// Transform types available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    Pivot,
    Melt,
    GroupBy,
    Join,
    Filter,
    Sort,
    Sample,
}

/// Export node data - exports data or plots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNodeData {
    pub format: crate::types::ExportFormat,
    pub destination: Option<PathBuf>,
}

/// Export destination options
#[derive(Debug, Clone, PartialEq)]
pub enum ExportDestination {
    File(PathBuf),
    Directory(PathBuf),
}

/// Node wrapper for canvas mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: NodeId,
    pub position: crate::types::Point2,
    pub size: crate::types::Size2,
    pub node_type: NodeType,
    pub selected: bool,
}

impl CanvasNode {
    /// Create a new canvas node
    pub fn new(id: NodeId, position: crate::types::Point2, node_type: NodeType) -> Self {
        CanvasNode {
            id,
            position,
            size: crate::types::Size2 { width: 200.0, height: 150.0 }, // Default size
            node_type,
            selected: false,
        }
    }
}

/// Cell wrapper for notebook mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: NodeId,
    pub cell_type: CellType,
    pub collapsed: bool,
    pub execution_count: Option<usize>,
}

impl NotebookCell {
    /// Create a new notebook cell
    pub fn new(id: NodeId, cell_type: CellType) -> Self {
        NotebookCell {
            id,
            cell_type,
            collapsed: false,
            execution_count: None,
        }
    }
} 