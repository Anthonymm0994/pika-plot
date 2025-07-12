// Complete Error Taxonomy for Pika-Plot
// All error types with user-friendly messages

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PikaError {
    // ===== Import/Export Errors =====
    
    #[error("CSV import failed at line {line}: {reason}")]
    CsvImport { 
        line: usize, 
        reason: String 
    },
    
    #[error("File not found: {}", path.display())]
    FileNotFound { 
        path: PathBuf 
    },
    
    #[error("Cannot read file '{}': {reason}", path.display())]
    FileReadError {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Unsupported file format: {extension}. Supported formats: CSV")]
    UnsupportedFormat { 
        extension: String 
    },
    
    #[error("File '{}' is too large: {size_mb}MB exceeds limit of {limit_mb}MB", path.display())]
    FileTooLarge {
        path: PathBuf,
        size_mb: usize,
        limit_mb: usize,
    },
    
    #[error("Invalid encoding in file '{}': expected {expected}, detected {detected}", path.display())]
    EncodingError {
        path: PathBuf,
        expected: String,
        detected: String,
    },
    
    // ===== Query Errors =====
    
    #[error("SQL query error: {message}")]
    QueryError { 
        message: String 
    },
    
    #[error("Query timeout after {seconds}s. Consider adding indexes or reducing data size.")]
    QueryTimeout { 
        seconds: u64 
    },
    
    #[error("Table '{name}' not found. Available tables: {}", available.join(", "))]
    TableNotFound { 
        name: String,
        available: Vec<String>,
    },
    
    #[error("Invalid SQL syntax near: {near}")]
    SqlSyntaxError {
        near: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    
    #[error("Circular reference detected in query dependencies")]
    CircularDependency {
        nodes: Vec<String>,
    },
    
    // ===== Memory Errors =====
    
    #[error("Not enough memory: need {required}MB, have {available}MB. Try closing other applications.")]
    InsufficientMemory { 
        required: usize, 
        available: usize 
    },
    
    #[error("Dataset too large: {rows:?} rows exceeds limit of {max_rows:?} rows. Consider filtering or sampling.")]
    DatasetTooLarge { 
        rows: usize, 
        max_rows: usize 
    },
    
    #[error("Memory allocation failed: {reason}")]
    AllocationError {
        reason: String,
    },
    
    // ===== GPU Errors =====
    
    #[error("GPU initialization failed: {reason}. Please check your graphics drivers.")]
    GpuInit { 
        reason: String 
    },
    
    #[error("Not enough GPU memory: need {required}MB, have {available}MB. Try reducing plot complexity.")]
    GpuMemory { 
        required: usize, 
        available: usize 
    },
    
    #[error("No discrete GPU found. Pika-Plot requires a dedicated graphics card.")]
    NoDiscreteGpu,
    
    #[error("GPU device lost. This can happen during driver updates or power saving.")]
    GpuDeviceLost,
    
    #[error("Shader compilation failed: {stage} shader: {reason}")]
    ShaderError {
        stage: String,
        reason: String,
    },
    
    #[error("GPU feature not supported: {feature}. Required GPU capabilities: {required}")]
    GpuFeatureNotSupported {
        feature: String,
        required: String,
    },
    
    // ===== Plot Errors =====
    
    #[error("Invalid plot configuration: {reason}")]
    InvalidPlotConfig { 
        reason: String 
    },
    
    #[error("Column '{column}' not found. Available columns: {}", available.join(", "))]
    MissingColumn { 
        column: String,
        available: Vec<String>,
    },
    
    #[error("Incompatible data type for column '{column}': expected {expected}, got {actual}")]
    IncompatibleType {
        column: String,
        expected: String,
        actual: String,
    },
    
    #[error("Too many unique values ({count}) for categorical axis. Maximum allowed: {max}")]
    TooManyCategories {
        count: usize,
        max: usize,
    },
    
    #[error("No numeric data found for plot type '{plot_type}'")]
    NoNumericData {
        plot_type: String,
    },
    
    #[error("Invalid data range: {reason}. Check for NaN or infinite values.")]
    InvalidDataRange {
        reason: String,
    },
    
    // ===== Export Errors =====
    
    #[error("Export failed: {reason}")]
    ExportError { 
        reason: String 
    },
    
    #[error("Permission denied: cannot write to '{}'", path.display())]
    PermissionDenied { 
        path: PathBuf 
    },
    
    #[error("Export cancelled by user")]
    ExportCancelled,
    
    #[error("Invalid export dimensions: {width}x{height}. Maximum allowed: {max_width}x{max_height}")]
    InvalidExportSize {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
    
    // ===== Snapshot Errors =====
    
    #[error("Invalid workspace snapshot: version {version} is not supported (expected {expected})")]
    InvalidSnapshotVersion { 
        version: u32, 
        expected: u32 
    },
    
    #[error("Data source has changed: '{}' (hash mismatch). Please re-import the file.", path.display())]
    DataSourceChanged { 
        path: PathBuf 
    },
    
    #[error("Corrupted snapshot file: {reason}")]
    CorruptedSnapshot {
        reason: String,
    },
    
    #[error("Missing data source: '{}' referenced in snapshot but not found", path.display())]
    MissingDataSource {
        path: PathBuf,
    },
    
    // ===== Node/Connection Errors =====
    
    #[error("Invalid connection: {reason}")]
    InvalidConnection {
        reason: String,
    },
    
    #[error("Node '{node_id}' not found")]
    NodeNotFound {
        node_id: String,
    },
    
    #[error("Port '{port_id}' not found on node '{node_id}'")]
    PortNotFound {
        node_id: String,
        port_id: String,
    },
    
    #[error("Type mismatch: cannot connect {from_type} output to {to_type} input")]
    ConnectionTypeMismatch {
        from_type: String,
        to_type: String,
    },
    
    #[error("Maximum node limit ({max}) reached. Delete unused nodes to continue.")]
    TooManyNodes {
        max: usize,
    },
    
    // ===== Data Processing Errors =====
    
    #[error("Data type inference failed: {reason}. Try specifying types manually.")]
    TypeInferenceError {
        reason: String,
    },
    
    #[error("Date parsing error in column '{column}': '{value}' doesn't match format '{format}'")]
    DateParseError {
        column: String,
        value: String,
        format: String,
    },
    
    #[error("Numeric overflow in calculation: {operation}")]
    NumericOverflow {
        operation: String,
    },
    
    #[error("Division by zero in expression: {expression}")]
    DivisionByZero {
        expression: String,
    },
    
    // ===== Cache Errors =====
    
    #[error("Cache corrupted: {reason}. Clearing cache may resolve this.")]
    CacheError {
        reason: String,
    },
    
    #[error("Cache size limit exceeded. Some queries will not be cached.")]
    CacheFull,
    
    // ===== Network Errors (for future cloud features) =====
    
    #[error("This feature requires an internet connection")]
    OfflineMode,
    
    // ===== General Errors =====
    
    #[error("Operation cancelled by user")]
    Cancelled,
    
    #[error("Operation in progress. Please wait for it to complete.")]
    OperationInProgress,
    
    #[error("Internal error: {message}. Please report this issue.")]
    Internal { 
        message: String 
    },
    
    #[error("Not implemented: {feature}")]
    NotImplemented {
        feature: String,
    },
    
    #[error("Invalid state: {reason}")]
    InvalidState {
        reason: String,
    },
}

// Helper methods for error handling
impl PikaError {
    /// Returns true if this error is recoverable (user can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            PikaError::QueryTimeout { .. } |
            PikaError::GpuDeviceLost |
            PikaError::Cancelled |
            PikaError::OperationInProgress |
            PikaError::CacheFull
        )
    }
    
    /// Returns a suggested action for the user
    pub fn suggested_action(&self) -> Option<&'static str> {
        match self {
            PikaError::InsufficientMemory { .. } => 
                Some("Try closing other applications or reducing the dataset size"),
            PikaError::NoDiscreteGpu => 
                Some("Pika-Plot requires a dedicated GPU. Check system requirements"),
            PikaError::GpuDeviceLost => 
                Some("Try restarting the application"),
            PikaError::DataSourceChanged { .. } => 
                Some("Re-import the file to update the data"),
            PikaError::TooManyCategories { .. } => 
                Some("Consider grouping categories or using a different plot type"),
            PikaError::CacheError { .. } => 
                Some("Clear the cache from Settings > Advanced"),
            _ => None,
        }
    }
    
    /// Returns an error code for logging/debugging
    pub fn error_code(&self) -> &'static str {
        match self {
            PikaError::CsvImport { .. } => "E001",
            PikaError::FileNotFound { .. } => "E002",
            PikaError::FileReadError { .. } => "E003",
            PikaError::UnsupportedFormat { .. } => "E004",
            PikaError::FileTooLarge { .. } => "E005",
            PikaError::EncodingError { .. } => "E006",
            PikaError::QueryError { .. } => "E101",
            PikaError::QueryTimeout { .. } => "E102",
            PikaError::TableNotFound { .. } => "E103",
            PikaError::SqlSyntaxError { .. } => "E104",
            PikaError::CircularDependency { .. } => "E105",
            PikaError::InsufficientMemory { .. } => "E201",
            PikaError::DatasetTooLarge { .. } => "E202",
            PikaError::AllocationError { .. } => "E203",
            PikaError::GpuInit { .. } => "E301",
            PikaError::GpuMemory { .. } => "E302",
            PikaError::NoDiscreteGpu => "E303",
            PikaError::GpuDeviceLost => "E304",
            PikaError::ShaderError { .. } => "E305",
            PikaError::GpuFeatureNotSupported { .. } => "E306",
            PikaError::InvalidPlotConfig { .. } => "E401",
            PikaError::MissingColumn { .. } => "E402",
            PikaError::IncompatibleType { .. } => "E403",
            PikaError::TooManyCategories { .. } => "E404",
            PikaError::NoNumericData { .. } => "E405",
            PikaError::InvalidDataRange { .. } => "E406",
            PikaError::ExportError { .. } => "E501",
            PikaError::PermissionDenied { .. } => "E502",
            PikaError::ExportCancelled => "E503",
            PikaError::InvalidExportSize { .. } => "E504",
            PikaError::InvalidSnapshotVersion { .. } => "E601",
            PikaError::DataSourceChanged { .. } => "E602",
            PikaError::CorruptedSnapshot { .. } => "E603",
            PikaError::MissingDataSource { .. } => "E604",
            PikaError::InvalidConnection { .. } => "E701",
            PikaError::NodeNotFound { .. } => "E702",
            PikaError::PortNotFound { .. } => "E703",
            PikaError::ConnectionTypeMismatch { .. } => "E704",
            PikaError::TooManyNodes { .. } => "E705",
            PikaError::TypeInferenceError { .. } => "E801",
            PikaError::DateParseError { .. } => "E802",
            PikaError::NumericOverflow { .. } => "E803",
            PikaError::DivisionByZero { .. } => "E804",
            PikaError::CacheError { .. } => "E901",
            PikaError::CacheFull => "E902",
            PikaError::OfflineMode => "E1001",
            PikaError::Cancelled => "E1101",
            PikaError::OperationInProgress => "E1102",
            PikaError::Internal { .. } => "E9999",
            PikaError::NotImplemented { .. } => "E9998",
            PikaError::InvalidState { .. } => "E9997",
        }
    }
}

// Conversion from common error types
impl From<std::io::Error> for PikaError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => PikaError::FileNotFound {
                path: PathBuf::from("<unknown>"),
            },
            std::io::ErrorKind::PermissionDenied => PikaError::PermissionDenied {
                path: PathBuf::from("<unknown>"),
            },
            _ => PikaError::Internal {
                message: err.to_string(),
            },
        }
    }
}

impl From<duckdb::Error> for PikaError {
    fn from(err: duckdb::Error) -> Self {
        // Parse DuckDB errors and convert to appropriate PikaError
        let msg = err.to_string();
        if msg.contains("does not exist") || msg.contains("not found") {
            // Extract table name if possible
            PikaError::TableNotFound {
                name: "<unknown>".to_string(),
                available: vec![],
            }
        } else if msg.contains("syntax error") {
            PikaError::SqlSyntaxError {
                near: msg.clone(),
                line: None,
                column: None,
            }
        } else {
            PikaError::QueryError { message: msg }
        }
    }
} 