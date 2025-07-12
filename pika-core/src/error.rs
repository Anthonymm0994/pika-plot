//! Error types for the application.

use thiserror::Error;

/// Main error type for Pika-Plot
#[derive(Error, Debug)]
pub enum PikaError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),
    
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Unsupported version error
    #[error("Unsupported version: found {found}, expected {expected}")]
    UnsupportedVersion {
        found: u32,
        expected: u32,
    },
    
    /// Invalid plot configuration
    #[error("Invalid plot configuration: {0}")]
    InvalidPlotConfig(String),
    
    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// File read error
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    
    /// CSV import error
    #[error("CSV import error: {0}")]
    CsvImport(String),
    
    /// Query execution error
    #[error("Query execution error: {0}")]
    QueryExecution(String),
    
    /// Memory limit exceeded
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    
    /// Invalid node connection
    #[error("Invalid node connection: {0}")]
    InvalidConnection(String),
    
    /// Window not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    
    /// Invalid data type
    #[error("Invalid data type: expected {expected}, found {found}")]
    InvalidDataType {
        expected: String,
        found: String,
    },
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Render error
    #[error("Render error: {0}")]
    RenderError(String),
    
    /// Export error
    #[error("Export error: {0}")]
    ExportError(String),
    
    /// Generic error wrapper
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, PikaError>;

impl PikaError {
    /// Create an internal error with a message
    pub fn internal(msg: impl Into<String>) -> Self {
        PikaError::Internal(msg.into())
    }
    
    /// Create a not implemented error
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        PikaError::NotImplemented(feature.into())
    }
} 