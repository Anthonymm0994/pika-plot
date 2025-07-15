//! Error types for the pika-engine crate.
//!
//! This module defines the error types that can occur during
//! engine operations, including data import, query execution, and caching.

use thiserror::Error;
use pika_core::error::PikaError;

/// Main error type for engine operations
#[derive(Error, Debug)]
pub enum EngineError {
    /// Error from the core library
    #[error("Core error: {0}")]
    Core(#[from] PikaError),
    
    /// Generic engine error with a message
    #[error("Engine error: {0}")]
    Engine(String),
    
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// CSV parsing error
    #[error("CSV parsing error: {0}")]
    CsvParse(String),
    
    /// Data type inference error
    #[error("Failed to infer data type: {0}")]
    TypeInference(String),
    
    /// Query building or execution error
    #[error("Query error: {0}")]
    Query(String),
    
    /// Cache operation error
    #[error("Cache error: {0}")]
    Cache(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

impl EngineError {
    /// Create a generic engine error with a message
    pub fn engine(msg: impl Into<String>) -> Self {
        Self::Engine(msg.into())
    }
    
    /// Create a CSV parsing error
    pub fn csv_parse(msg: impl Into<String>) -> Self {
        Self::CsvParse(msg.into())
    }
    
    /// Create a type inference error
    pub fn type_inference(msg: impl Into<String>) -> Self {
        Self::TypeInference(msg.into())
    }
    
    /// Create a query error
    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query(msg.into())
    }
    
    /// Create a cache error
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }
    
    /// Create an invalid configuration error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
    
    /// Create a resource limit error
    pub fn resource_limit(msg: impl Into<String>) -> Self {
        Self::ResourceLimit(msg.into())
    }
    
    /// Check if this is an IO error
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::Io(_))
    }
    
    /// Check if this is a resource limit error
    pub fn is_resource_limit(&self) -> bool {
        matches!(self, Self::ResourceLimit(_))
    }
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, EngineError>; 

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, msg: impl Into<String>) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<EngineError>,
{
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            EngineError::Engine(format!("{}: {}", msg.into(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = EngineError::engine("Something went wrong");
        assert_eq!(err.to_string(), "Engine error: Something went wrong");
        
        let err = EngineError::csv_parse("Invalid delimiter");
        assert_eq!(err.to_string(), "CSV parsing error: Invalid delimiter");
        
        let err = EngineError::type_inference("Cannot determine type");
        assert_eq!(err.to_string(), "Failed to infer data type: Cannot determine type");
    }
    
    #[test]
    fn test_error_constructors() {
        let err = EngineError::query("Invalid SQL");
        assert!(matches!(err, EngineError::Query(_)));
        
        let err = EngineError::cache("Cache full");
        assert!(matches!(err, EngineError::Cache(_)));
        
        let err = EngineError::invalid_config("Missing required field");
        assert!(matches!(err, EngineError::InvalidConfig(_)));
        
        let err = EngineError::resource_limit("Memory limit exceeded");
        assert!(matches!(err, EngineError::ResourceLimit(_)));
    }
    
    #[test]
    fn test_error_predicates() {
        let io_err = EngineError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        ));
        assert!(io_err.is_io_error());
        assert!(!io_err.is_resource_limit());
        
        let limit_err = EngineError::resource_limit("Too many rows");
        assert!(!limit_err.is_io_error());
        assert!(limit_err.is_resource_limit());
    }
    
    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let engine_error: EngineError = io_error.into();
        assert!(matches!(engine_error, EngineError::Io(_)));
    }
    
    #[test]
    fn test_result_context() {
        fn failing_operation() -> std::result::Result<(), std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
        }
        
        let result = failing_operation().context("Failed to perform operation");
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to perform operation"));
        assert!(err.to_string().contains("Not found"));
    }
} 