//! Error types for the application.

use thiserror::Error;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Core error types for the Pika plotting system
#[derive(Debug, thiserror::Error)]
pub enum PikaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(String),  // Generic database error for now
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Plot rendering error: {0}")]
    PlotRendering(String),
    
    #[error("Data processing error: {0}")]
    DataProcessing(String),
    
    #[error("Node execution error: {0}")]
    NodeExecution(String),
    
    #[error("Workspace error: {0}")]
    Workspace(String),
    
    #[error("Import error: {0}")]
    Import(String),
    
    #[error("Export error: {0}")]
    Export(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("GPU error: {0}")]
    Gpu(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Permission error: {0}")]
    Permission(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Cancelled operation: {0}")]
    Cancelled(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, PikaError>;

// COMMENTED OUT COMPLEX ERROR HANDLING - Will be restored once core functionality works
/*
impl PikaError {
    // Complex error categorization and handling functions
    // These will be restored once the basic structure is working
}
*/ 