// Core Type Definitions for Pika-Plot
// This file contains all shared types that cross crate boundaries

use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use std::time::Duration;

// ===== Node System Types =====

/// Unique identifier for nodes in the workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        NodeId(Uuid::new_v4())
    }
}

/// Position in 2D canvas space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
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
    pub delimiter: char,           // default: ','
    pub has_header: bool,         // default: true
    pub quote_char: char,         // default: '"'
    pub escape_char: Option<char>, // default: Some('\\')
    pub null_values: Vec<String>, // default: ["", "NULL", "null", "N/A", "NA", "n/a"]
    pub sample_size: usize,       // default: 10000
    pub type_inference: bool,     // default: true
    pub encoding: String,         // default: "utf-8"
    pub skip_rows: usize,         // default: 0
    pub max_rows: Option<usize>,  // default: None
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            delimiter: ',',
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
            type_inference: true,
            encoding: "utf-8".to_string(),
            skip_rows: 0,
            max_rows: None,
        }
    }
}

// ===== Query & Results Types =====

/// Result of a SQL query execution
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub data: Arc<RecordBatch>,
    pub execution_time: Duration,
    pub row_count: usize,
    pub memory_usage: usize, // in bytes
}

/// Fingerprint for query caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryFingerprint(String);

// ===== Plot Configuration Types =====

/// Configuration for any plot type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotConfig {
    pub plot_type: PlotType,
    pub title: Option<String>,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub color_scale: ColorScale,
    pub theme: PlotTheme,
    pub specific: PlotSpecificConfig,
}

/// Supported plot types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotType {
    Scatter,
    Line,
    Bar,
    Histogram,
    Heatmap,
    Box,
    Violin,
    Area,
    Pie,
    Donut,
    Treemap,
    Sunburst,
    Sankey,
    Network,
    Geo,
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    pub column: String,
    pub label: Option<String>,
    pub scale: AxisScale,
    pub range: Option<(f64, f64)>,
    pub tick_format: Option<String>,
}

/// Axis scale type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisScale {
    Linear,
    Log,
    Time,
    Categorical,
}

/// Color scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScale {
    Sequential(String),    // e.g., "viridis", "plasma", "turbo"
    Diverging(String),     // e.g., "RdBu", "PiYG"
    Categorical(String),   // e.g., "Set1", "Dark2"
    Custom(Vec<Color32>),
}

/// 32-bit color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Plot theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotTheme {
    Light,
    Dark,
    Auto, // Follow system theme
}

/// Plot-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlotSpecificConfig {
    Scatter {
        x_column: String,
        y_column: String,
        size_column: Option<String>,
        color_column: Option<String>,
        label_column: Option<String>,
        point_size: f32,        // default: 5.0
        opacity: f32,           // default: 0.8
        jitter: Option<f32>,    // default: None
    },
    Line {
        x_column: String,
        y_column: String,
        group_column: Option<String>,
        line_width: f32,        // default: 2.0
        interpolation: LineInterpolation,
        markers: bool,          // default: false
    },
    Histogram {
        column: String,
        bins: BinConfig,
        cumulative: bool,       // default: false
        density: bool,          // default: false
    },
    Bar {
        x_column: String,
        y_column: String,
        orientation: Orientation,
        group_mode: BarGroupMode,
    },
    Heatmap {
        x_column: String,
        y_column: String,
        value_column: String,
        aggregation: AggregationType,
        interpolation: bool,    // default: false
    },
    // ... other plot-specific configs
}

/// Line interpolation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineInterpolation {
    Linear,
    Step,
    StepBefore,
    StepAfter,
    Basis,
    Cardinal,
    MonotoneX,
    MonotoneY,
}

/// Histogram bin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinConfig {
    Count(usize),           // Fixed number of bins
    Width(f64),            // Fixed bin width
    Auto,                  // Use Sturges' rule
    Scotts,                // Scott's rule
    FreedmanDiaconis,      // Freedman-Diaconis rule
}

/// Bar chart orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Bar chart grouping mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarGroupMode {
    Grouped,
    Stacked,
    Overlay,
}

/// Aggregation type for data reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Mean,
    Median,
    Min,
    Max,
    Count,
    CountDistinct,
    StandardDeviation,
    Variance,
    First,
    Last,
}

// ===== GPU Types =====

/// GPU buffer information
#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub uniform_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub instance_count: u32,
    pub topology: wgpu::PrimitiveTopology,
}

/// Plot data prepared for GPU rendering
#[derive(Debug, Clone)]
pub struct PlotData {
    pub bounds: PlotBounds,
    pub point_count: usize,
    pub render_mode: RenderMode,
    pub buffers: Option<GpuBuffer>,
}

/// Rendering mode based on data size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Direct,      // < 50k points
    Instanced,   // 50k - 5M points
    Aggregated,  // > 5M points
}

/// Plot bounds in data space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlotBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

// ===== Event System Types =====

/// Events for UI-Engine communication
#[derive(Debug, Clone)]
pub enum AppEvent {
    // UI -> Engine
    ImportCsv {
        path: PathBuf,
        options: ImportOptions,
    },
    ExecuteQuery {
        id: NodeId,
        sql: String,
        cache_key: Option<QueryFingerprint>,
    },
    Prepareplot {
        id: NodeId,
        source: NodeId,
        config: PlotConfig,
    },
    CancelOperation {
        id: NodeId,
    },
    
    // Engine -> UI
    ImportStarted {
        path: PathBuf,
    },
    ImportProgress {
        path: PathBuf,
        progress: f32,
    },
    ImportComplete {
        path: PathBuf,
        table_name: String,
        schema: Arc<Schema>,
        row_count: usize,
    },
    ImportError {
        path: PathBuf,
        error: PikaError,
    },
    QueryStarted {
        id: NodeId,
    },
    QueryComplete {
        id: NodeId,
        result: Result<QueryResult, PikaError>,
    },
    PlotDataReady {
        id: NodeId,
        data: Arc<PlotData>,
    },
    MemoryWarning {
        used: usize,
        available: usize,
    },
    Error {
        context: String,
        error: PikaError,
    },
}

// ===== Workspace Types =====

/// UI mode for the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceMode {
    Notebook {
        cells: Vec<NotebookCell>,
        active_cell: Option<usize>,
    },
    Canvas {
        nodes: HashMap<NodeId, CanvasNode>,
        connections: Vec<Connection>,
        camera: Camera2D,
    },
}

/// Notebook cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: NodeId,
    pub cell_type: CellType,
    pub collapsed: bool,
    pub execution_count: Option<usize>,
}

/// Canvas node wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: NodeId,
    pub position: Point2,
    pub size: Point2,
    pub node_type: NodeType,
    pub selected: bool,
}

/// Cell type for notebook mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    Table(TableNodeData),
    Query(QueryNodeData),
    Plot(PlotNodeData),
    Markdown(String),
}

/// Node type for canvas mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Table(TableNodeData),
    Query(QueryNodeData),
    Plot(PlotNodeData),
    Transform(TransformNodeData),
    Export(ExportNodeData),
}

/// Table node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNodeData {
    pub source_path: PathBuf,
    pub table_name: String,
    pub import_options: ImportOptions,
    pub schema: Option<Arc<Schema>>,
    pub row_count: Option<usize>,
}

/// Query node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryNodeData {
    pub sql: String,
    pub input_tables: Vec<String>,
    pub cached_result: Option<Arc<QueryResult>>,
}

/// Plot node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotNodeData {
    pub config: PlotConfig,
    pub source_node: Option<NodeId>,
    pub cached_data: Option<Arc<PlotData>>,
}

/// Transform node data (for future extensibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformNodeData {
    pub transform_type: TransformType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Export node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportNodeData {
    pub format: ExportFormat,
    pub destination: Option<PathBuf>,
}

/// Transform types
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

// ===== Export Types =====

/// Supported export formats
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
        orient: JsonOrientation,
    },
    Arrow {
        compression: Option<CompressionType>,
    },
    Parquet {
        compression: CompressionType,
    },
    
    // Workspace
    PikaPlot {
        version: u32,
        include_cache: bool,
    },
}

/// JSON export orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonOrientation {
    Records,
    Columns,
    Values,
    Table,
}

/// Compression type for exports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Brotli,
    Zstd,
    Lz4,
}

// ===== Snapshot Types =====

/// Workspace snapshot for saving/loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub mode: WorkspaceMode,
    pub data_sources: Vec<DataSourceRef>,
    pub theme: PlotTheme,
    pub memory_limit: Option<usize>,
}

/// Reference to external data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRef {
    pub original_path: PathBuf,
    pub table_name: String,
    pub import_options: ImportOptions,
    pub file_hash: String,        // SHA256 hash
    pub file_size: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

// ===== Error Types =====

#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
pub enum PikaError {
    // Import errors
    #[error("CSV import failed at line {line}: {reason}")]
    CsvImport { line: usize, reason: String },
    
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Unsupported file format: {extension}")]
    UnsupportedFormat { extension: String },
    
    // Query errors
    #[error("Query error: {message}")]
    QueryError { message: String },
    
    #[error("Query timeout after {seconds}s")]
    QueryTimeout { seconds: u64 },
    
    #[error("Table not found: {name}")]
    TableNotFound { name: String },
    
    // Memory errors
    #[error("Not enough memory: need {required}MB, have {available}MB")]
    InsufficientMemory { required: usize, available: usize },
    
    #[error("Dataset too large: {rows} rows exceeds limit of {max_rows}")]
    DatasetTooLarge { rows: usize, max_rows: usize },
    
    // GPU errors
    #[error("GPU initialization failed: {reason}")]
    GpuInit { reason: String },
    
    #[error("Not enough GPU memory: need {required}MB, have {available}MB")]
    GpuMemory { required: usize, available: usize },
    
    #[error("No discrete GPU found")]
    NoDiscreteGpu,
    
    // Plot errors
    #[error("Invalid plot configuration: {reason}")]
    InvalidPlotConfig { reason: String },
    
    #[error("Missing required column: {column}")]
    MissingColumn { column: String },
    
    #[error("Incompatible data type for {column}: expected {expected}, got {actual}")]
    IncompatibleType {
        column: String,
        expected: String,
        actual: String,
    },
    
    // Export errors
    #[error("Export failed: {reason}")]
    ExportError { reason: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    // Snapshot errors
    #[error("Invalid snapshot version: {version} (expected {expected})")]
    InvalidSnapshotVersion { version: u32, expected: u32 },
    
    #[error("Data source changed: {path} (hash mismatch)")]
    DataSourceChanged { path: PathBuf },
    
    // General errors
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

// ===== Trait Definitions =====

/// Core trait for all node types
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn node_type(&self) -> &'static str;
    fn title(&self) -> String;
    fn render(&mut self, ui: &mut egui::Ui, ctx: &AppContext);
    fn inputs(&self) -> Vec<PortId>;
    fn outputs(&self) -> Vec<PortId>;
    fn can_connect(&self, from_port: &PortId, to_node: &dyn Node, to_port: &PortId) -> bool;
}

/// Application context passed to nodes
pub struct AppContext {
    pub events_tx: mpsc::Sender<AppEvent>,
    pub theme: PlotTheme,
    pub memory_monitor: Arc<MemoryMonitor>,
}

/// Memory monitoring
pub struct MemoryMonitor {
    pub warning_threshold: f64,
    pub max_threshold: f64,
}

impl MemoryMonitor {
    pub fn available_memory(&self) -> usize {
        // Platform-specific implementation
        #[cfg(windows)]
        {
            // Use Windows API
            0 // Placeholder
        }
        #[cfg(not(windows))]
        {
            // Use other platform APIs
            0 // Placeholder
        }
    }
    
    pub fn used_memory(&self) -> usize {
        // Get process memory usage
        0 // Placeholder
    }
    
    pub fn check_allocation(&self, bytes: usize) -> Result<(), PikaError> {
        let available = self.available_memory();
        let used = self.used_memory();
        let after = used + bytes;
        
        if after as f64 > available as f64 * self.max_threshold {
            Err(PikaError::InsufficientMemory {
                required: bytes / 1_048_576,
                available: (available - used) / 1_048_576,
            })
        } else {
            Ok(())
        }
    }
} 