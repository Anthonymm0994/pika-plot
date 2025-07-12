//! Error types for the Pika-Plot system.

use thiserror::Error;

/// Result type alias for Pika operations.
pub type Result<T> = std::result::Result<T, PikaError>;

/// Main error type for Pika-Plot.
#[derive(Error, Debug)]
pub enum PikaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("GPU error: {0}")]
    GPU(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Import error: {0}")]
    Import(String),
    
    #[error("Export error: {0}")]
    Export(String),
    
    #[error("Node error: {0}")]
    Node(String),
    
    #[error("Invalid connection: {0}")]
    InvalidConnection(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Memory limit exceeded: {0}")]
    MemoryLimit(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
    
    #[error("Plot configuration invalid: {reason}")]
    InvalidPlotConfig { reason: String },
    
    #[error("Insufficient memory: required {required} bytes, available {available} bytes")]
    InsufficientMemory {
        required: u64,
        available: u64,
    },
    
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
    
    #[error("GPU initialization failed: {0}")]
    GpuInitialization(String),
    
    #[error("File read error: {error}")]
    FileReadError { error: String },
    
    #[error("CSV import error: {error} (line {line:?})")]
    CsvImport {
        error: String,
        line: Option<usize>,
    },
    
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("File too large: {path} ({size} bytes)")]
    FileTooLarge { path: String, size: u64 },
    
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Invalid port: {0}")]
    InvalidPort(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl From<duckdb::Error> for PikaError {
    fn from(err: duckdb::Error) -> Self {
        PikaError::Database(err.to_string())
    }
}

impl From<anyhow::Error> for PikaError {
    fn from(err: anyhow::Error) -> Self {
        PikaError::Other(err.to_string())
    }
} 