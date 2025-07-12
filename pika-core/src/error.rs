//! Error types for the application.

use thiserror::Error;
use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};

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
    
    /// File write error
    #[error("Failed to write file: {0}")]
    FileWriteError(String),
    
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
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Invalid port
    #[error("Invalid port: {0}")]
    InvalidPort(String),
    
    /// Enhanced user error with context
    #[error("{}", .context.user_message)]
    UserError { context: ErrorContext },
    
    /// Enhanced system error with context
    #[error("{}", .context.user_message)]
    SystemError { context: ErrorContext },
    
    /// Enhanced transient error with context
    #[error("{}", .context.user_message)]
    TransientError { context: ErrorContext },
    
    /// Enhanced configuration error with context
    #[error("{}", .context.user_message)]
    ConfigurationError { context: ErrorContext },
    
    /// Enhanced data quality error with context
    #[error("{}", .context.user_message)]
    DataQualityError { context: ErrorContext },
    
    /// Enhanced performance error with context
    #[error("{}", .context.user_message)]
    PerformanceError { context: ErrorContext },
    
    /// Import failed after all fallback strategies
    #[error("{}", .context.user_message)]
    ImportFailed { context: ErrorContext },
    
    /// Generic error wrapper
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Enhanced error context with recovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub user_message: String,
    pub technical_details: String,
    pub recovery_suggestions: Vec<RecoverySuggestion>,
    pub diagnostic_info: Option<DiagnosticInfo>,
    pub context_data: HashMap<String, String>,
    pub error_code: Option<String>,
    pub help_url: Option<String>,
}

/// Categories of errors for better handling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// User-fixable errors (wrong file format, missing columns)
    UserError,
    /// System errors (out of memory, disk full)
    SystemError,
    /// Transient errors (network timeouts, file locks)
    TransientError,
    /// Configuration errors (invalid settings, missing dependencies)
    ConfigurationError,
    /// Data quality issues (corrupted files, invalid data)
    DataQualityError,
    /// Performance issues (too much data, slow queries)
    PerformanceError,
}

/// Severity levels for errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational - operation succeeded with warnings
    Info,
    /// Warning - operation succeeded but with issues
    Warning,
    /// Error - operation failed but system remains stable
    Error,
    /// Critical - operation failed and system stability affected
    Critical,
    /// Fatal - system cannot continue
    Fatal,
}

/// Recovery suggestion for error resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    pub action: String,
    pub description: String,
    pub automatic: bool,
    pub confidence: f32,
    pub estimated_time: Option<Duration>,
    pub prerequisites: Vec<String>,
}

/// Diagnostic information for error analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub system_info: SystemInfo,
    pub operation_context: String,
    pub data_characteristics: Option<DataCharacteristics>,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub stack_trace: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System information for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_usage: f32,
    pub gpu_info: Option<GpuInfo>,
}

/// GPU information for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub memory_total: u64,
    pub memory_available: u64,
    pub driver_version: String,
    pub supports_compute: bool,
}

/// Data characteristics for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCharacteristics {
    pub point_count: usize,
    pub memory_usage: u64,
    pub column_count: usize,
    pub data_types: Vec<String>,
    pub file_size: Option<u64>,
    pub estimated_processing_time: Option<Duration>,
}

/// Performance metrics for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub gpu_memory_usage: Option<u64>,
    pub disk_io_rate: f64,
    pub query_duration: Option<Duration>,
    pub render_duration: Option<Duration>,
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
    
    /// Create a user-friendly error with context
    pub fn user_error(
        message: impl Into<String>,
        technical_details: impl Into<String>,
        suggestions: Vec<RecoverySuggestion>,
    ) -> Self {
        PikaError::UserError {
            context: ErrorContext {
                category: ErrorCategory::UserError,
                severity: ErrorSeverity::Error,
                user_message: message.into(),
                technical_details: technical_details.into(),
                recovery_suggestions: suggestions,
                diagnostic_info: None,
                context_data: HashMap::new(),
                error_code: None,
                help_url: None,
            }
        }
    }
    
    /// Create a transient error with retry suggestion
    pub fn transient_error(
        message: impl Into<String>,
        retry_after: Duration,
    ) -> Self {
        PikaError::TransientError {
            context: ErrorContext {
                category: ErrorCategory::TransientError,
                severity: ErrorSeverity::Warning,
                user_message: message.into(),
                technical_details: "Temporary issue, will retry automatically".to_string(),
                recovery_suggestions: vec![
                    RecoverySuggestion {
                        action: "retry".to_string(),
                        description: format!("Retry in {} seconds", retry_after.as_secs()),
                        automatic: true,
                        confidence: 0.8,
                        estimated_time: Some(retry_after),
                        prerequisites: vec![],
                    }
                ],
                diagnostic_info: None,
                context_data: HashMap::new(),
                error_code: None,
                help_url: None,
            }
        }
    }
    
    /// Create a system error with diagnostic info
    pub fn system_error(
        message: impl Into<String>,
        diagnostic: DiagnosticInfo,
    ) -> Self {
        PikaError::SystemError {
            context: ErrorContext {
                category: ErrorCategory::SystemError,
                severity: ErrorSeverity::Critical,
                user_message: message.into(),
                technical_details: "System resource issue detected".to_string(),
                recovery_suggestions: vec![
                    RecoverySuggestion {
                        action: "free_memory".to_string(),
                        description: "Close unused applications to free memory".to_string(),
                        automatic: false,
                        confidence: 0.9,
                        estimated_time: Some(Duration::from_secs(30)),
                        prerequisites: vec![],
                    }
                ],
                diagnostic_info: Some(diagnostic),
                context_data: HashMap::new(),
                error_code: None,
                help_url: None,
            }
        }
    }
    
    /// Create a memory error with automatic recovery
    pub fn memory_error(
        message: impl Into<String>,
        used_mb: usize,
        available_mb: usize,
    ) -> Self {
        let mut context_data = HashMap::new();
        context_data.insert("used_memory_mb".to_string(), used_mb.to_string());
        context_data.insert("available_memory_mb".to_string(), available_mb.to_string());
        
        PikaError::SystemError {
            context: ErrorContext {
                category: ErrorCategory::SystemError,
                severity: ErrorSeverity::Critical,
                user_message: message.into(),
                technical_details: format!("Memory usage: {}MB / {}MB", used_mb, available_mb),
                recovery_suggestions: vec![
                    RecoverySuggestion {
                        action: "clear_cache".to_string(),
                        description: "Clear data cache to free memory".to_string(),
                        automatic: true,
                        confidence: 0.9,
                        estimated_time: Some(Duration::from_secs(5)),
                        prerequisites: vec![],
                    },
                    RecoverySuggestion {
                        action: "enable_streaming".to_string(),
                        description: "Enable streaming mode for large datasets".to_string(),
                        automatic: true,
                        confidence: 0.8,
                        estimated_time: Some(Duration::from_secs(2)),
                        prerequisites: vec![],
                    },
                    RecoverySuggestion {
                        action: "reduce_sample_size".to_string(),
                        description: "Reduce data sample size".to_string(),
                        automatic: false,
                        confidence: 0.7,
                        estimated_time: Some(Duration::from_secs(10)),
                        prerequisites: vec!["User confirmation".to_string()],
                    },
                ],
                diagnostic_info: Some(DiagnosticInfo {
                    system_info: SystemInfo::current(),
                    operation_context: "Memory allocation".to_string(),
                    data_characteristics: None,
                    performance_metrics: None,
                    stack_trace: None,
                    timestamp: chrono::Utc::now(),
                }),
                context_data,
                error_code: Some("MEM_001".to_string()),
                help_url: Some("https://docs.pika-plot.com/troubleshooting/memory".to_string()),
            }
        }
    }
    
    /// Create a file access error with helpful suggestions
    pub fn file_access_error(
        path: impl Into<String>,
        operation: &str,
        underlying_error: String,
    ) -> Self {
        let path_str = path.into();
        let mut context_data = HashMap::new();
        context_data.insert("file_path".to_string(), path_str.clone());
        context_data.insert("operation".to_string(), operation.to_string());
        
        PikaError::UserError {
            context: ErrorContext {
                category: ErrorCategory::UserError,
                severity: ErrorSeverity::Error,
                user_message: format!("Unable to {} file: {}", operation, path_str),
                technical_details: underlying_error,
                recovery_suggestions: vec![
                    RecoverySuggestion {
                        action: "check_file_exists".to_string(),
                        description: "Verify the file exists and is accessible".to_string(),
                        automatic: false,
                        confidence: 0.9,
                        estimated_time: Some(Duration::from_secs(5)),
                        prerequisites: vec![],
                    },
                    RecoverySuggestion {
                        action: "check_permissions".to_string(),
                        description: "Check file permissions".to_string(),
                        automatic: false,
                        confidence: 0.8,
                        estimated_time: Some(Duration::from_secs(10)),
                        prerequisites: vec![],
                    },
                    RecoverySuggestion {
                        action: "try_different_location".to_string(),
                        description: "Try copying the file to a different location".to_string(),
                        automatic: false,
                        confidence: 0.6,
                        estimated_time: Some(Duration::from_secs(30)),
                        prerequisites: vec!["Write access to alternative location".to_string()],
                    },
                ],
                diagnostic_info: Some(DiagnosticInfo {
                    system_info: SystemInfo::current(),
                    operation_context: format!("File {} operation: {}", operation, path_str),
                    data_characteristics: None,
                    performance_metrics: None,
                    stack_trace: None,
                    timestamp: chrono::Utc::now(),
                }),
                context_data,
                error_code: Some("FILE_001".to_string()),
                help_url: Some("https://docs.pika-plot.com/troubleshooting/file-access".to_string()),
            }
        }
    }
    
    /// Create a data quality error with validation suggestions
    pub fn data_quality_error(
        issue: impl Into<String>,
        data_info: DataCharacteristics,
        suggestions: Vec<RecoverySuggestion>,
    ) -> Self {
        let issue_str = issue.into();
        let mut context_data = HashMap::new();
        context_data.insert("data_points".to_string(), data_info.point_count.to_string());
        context_data.insert("columns".to_string(), data_info.column_count.to_string());
        
        PikaError::DataQualityError {
            context: ErrorContext {
                category: ErrorCategory::DataQualityError,
                severity: ErrorSeverity::Warning,
                user_message: format!("Data quality issue: {}", issue_str),
                technical_details: format!("Dataset has {} rows and {} columns", data_info.point_count, data_info.column_count),
                recovery_suggestions: suggestions,
                diagnostic_info: Some(DiagnosticInfo {
                    system_info: SystemInfo::current(),
                    operation_context: "Data validation".to_string(),
                    data_characteristics: Some(data_info),
                    performance_metrics: None,
                    stack_trace: None,
                    timestamp: chrono::Utc::now(),
                }),
                context_data,
                error_code: Some("DATA_001".to_string()),
                help_url: Some("https://docs.pika-plot.com/troubleshooting/data-quality".to_string()),
            }
        }
    }
    
    /// Get error context if available
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            PikaError::UserError { context } |
            PikaError::SystemError { context } |
            PikaError::TransientError { context } |
            PikaError::ConfigurationError { context } |
            PikaError::DataQualityError { context } |
            PikaError::PerformanceError { context } |
            PikaError::ImportFailed { context } => Some(context),
            _ => None,
        }
    }
    
    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        if let Some(context) = self.context() {
            context.user_message.clone()
        } else {
            // Fallback to simplified messages for legacy errors
            match self {
                PikaError::FileReadError(_) => "Unable to read file".to_string(),
                PikaError::FileWriteError(_) => "Unable to write file".to_string(),
                PikaError::CsvImport(_) => "CSV import failed".to_string(),
                PikaError::QueryExecution(_) => "Query execution failed".to_string(),
                PikaError::MemoryLimitExceeded(_) => "Memory limit exceeded".to_string(),
                PikaError::RenderError(_) => "Rendering failed".to_string(),
                _ => "An error occurred".to_string(),
            }
        }
    }
    
    /// Get technical details
    pub fn technical_details(&self) -> String {
        if let Some(context) = self.context() {
            context.technical_details.clone()
        } else {
            self.to_string()
        }
    }
    
    /// Get recovery suggestions
    pub fn recovery_suggestions(&self) -> Vec<RecoverySuggestion> {
        if let Some(context) = self.context() {
            context.recovery_suggestions.clone()
        } else {
            // Provide default suggestions for legacy errors
            match self {
                PikaError::FileReadError(_) => vec![
                    RecoverySuggestion {
                        action: "check_file_exists".to_string(),
                        description: "Verify the file exists and is accessible".to_string(),
                        automatic: false,
                        confidence: 0.9,
                        estimated_time: Some(Duration::from_secs(5)),
                        prerequisites: vec![],
                    },
                ],
                PikaError::MemoryLimitExceeded(_) => vec![
                    RecoverySuggestion {
                        action: "clear_cache".to_string(),
                        description: "Clear data cache to free memory".to_string(),
                        automatic: true,
                        confidence: 0.9,
                        estimated_time: Some(Duration::from_secs(5)),
                        prerequisites: vec![],
                    },
                ],
                _ => vec![],
            }
        }
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !self.recovery_suggestions().is_empty()
    }
    
    /// Check if error has automatic recovery
    pub fn has_automatic_recovery(&self) -> bool {
        self.recovery_suggestions().iter().any(|s| s.automatic)
    }
}

impl SystemInfo {
    /// Get current system information
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: num_cpus::get(),
            total_memory: Self::get_total_memory(),
            available_memory: Self::get_available_memory(),
            cpu_usage: Self::get_cpu_usage(),
            gpu_info: None, // Would be populated by GPU detection
        }
    }
    
    fn get_total_memory() -> u64 {
        // Platform-specific memory detection
        // This is a simplified version
        8 * 1024 * 1024 * 1024 // Default to 8GB
    }
    
    fn get_available_memory() -> u64 {
        // Platform-specific available memory detection
        // This is a simplified version
        4 * 1024 * 1024 * 1024 // Default to 4GB
    }
    
    fn get_cpu_usage() -> f32 {
        // Platform-specific CPU usage detection
        // This is a simplified version
        0.0
    }
}

impl DataCharacteristics {
    /// Estimate memory usage for the dataset
    pub fn estimated_memory_usage(&self) -> u64 {
        // Rough estimation: 8 bytes per numeric value, 50 bytes per string
        let avg_bytes_per_value = 30; // Mixed data types
        (self.point_count * self.column_count * avg_bytes_per_value) as u64
    }
    
    /// Check if dataset is considered large
    pub fn is_large_dataset(&self) -> bool {
        self.point_count > 1_000_000 || self.estimated_memory_usage() > 1_000_000_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_error_creation() {
        let error = PikaError::user_error(
            "Test error",
            "Technical details",
            vec![RecoverySuggestion {
                action: "test_action".to_string(),
                description: "Test description".to_string(),
                automatic: false,
                confidence: 0.8,
                estimated_time: Some(Duration::from_secs(10)),
                prerequisites: vec![],
            }],
        );
        
        assert_eq!(error.user_message(), "Test error");
        assert_eq!(error.technical_details(), "Technical details");
        assert!(error.is_recoverable());
        assert!(!error.has_automatic_recovery());
    }
    
    #[test]
    fn test_memory_error_creation() {
        let error = PikaError::memory_error("Out of memory", 7000, 8000);
        
        assert!(error.user_message().contains("Out of memory"));
        assert!(error.is_recoverable());
        assert!(error.has_automatic_recovery());
        
        let suggestions = error.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.automatic));
    }
    
    #[test]
    fn test_file_access_error_creation() {
        let error = PikaError::file_access_error(
            "/path/to/file.csv",
            "read",
            "Permission denied".to_string(),
        );
        
        assert!(error.user_message().contains("Unable to read file"));
        assert!(error.is_recoverable());
        
        let suggestions = error.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.action == "check_file_exists"));
    }
    
    #[test]
    fn test_data_characteristics() {
        let data_chars = DataCharacteristics {
            point_count: 1_000_000,
            memory_usage: 0,
            column_count: 10,
            data_types: vec!["int".to_string(), "float".to_string()],
            file_size: Some(100_000_000),
            estimated_processing_time: None,
        };
        
        assert!(data_chars.is_large_dataset());
        assert!(data_chars.estimated_memory_usage() > 0);
    }
} 