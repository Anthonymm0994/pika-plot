//! Data processing engine for Pika Plot.
//!
//! This crate provides the core data processing functionality including:
//! - CSV file import with automatic type detection
//! - Query building utilities
//! - Result caching with LRU eviction
//! 
//! # Architecture
//! 
//! The engine is designed to be the data processing layer between the UI
//! and the underlying data sources. Currently, it supports CSV import
//! with plans for additional data sources in the future.
//!
//! # Example
//! 
//! ```ignore
//! use pika_engine::{Engine, csv::EnhancedCsvImporter};
//! use pika_core::types::ImportOptions;
//! 
//! // Create an engine instance
//! let engine = Engine::new(event_bus);
//! 
//! // Import a CSV file
//! let options = ImportOptions::default();
//! let table_info = engine.import_csv("data.csv", options, node_id).await?;
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod cache;
pub mod csv;
pub mod query;
pub mod error;

// Re-export commonly used types
pub use error::{EngineError, Result};
pub use cache::QueryCache;
pub use csv::EnhancedCsvImporter;
pub use query::QueryBuilder;

use std::sync::Arc;
use tokio::sync::Mutex;
use pika_core::{
    events::EventBus,
    types::{TableInfo, ImportOptions, NodeId, QueryResult},
};

/// Configuration for the engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum number of cached query results
    pub cache_size: usize,
    /// Maximum file size for import (in bytes)
    pub max_file_size: Option<usize>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
        }
    }
}

/// Main engine for data processing
#[derive(Debug)]
pub struct Engine {
    event_bus: Arc<EventBus>,
    cache: Arc<Mutex<QueryCache>>,
    config: EngineConfig,
}

impl Engine {
    /// Create a new engine instance with default configuration
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self::with_config(event_bus, EngineConfig::default())
    }
    
    /// Create a new engine instance with custom configuration
    pub fn with_config(event_bus: Arc<EventBus>, config: EngineConfig) -> Self {
        let cache = Arc::new(Mutex::new(QueryCache::new(config.cache_size)));
        
        Self {
            event_bus,
            cache,
            config,
        }
    }
    
    /// Import a CSV file
    /// 
    /// # Arguments
    /// * `path` - Path to the CSV file
    /// * `options` - Import options
    /// * `node_id` - ID of the node requesting the import
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file exceeds the maximum size limit
    /// - CSV parsing fails
    pub async fn import_csv(
        &self, 
        path: &str, 
        options: ImportOptions, 
        _node_id: NodeId
    ) -> Result<TableInfo> {
        // Check file size if limit is configured
        if let Some(max_size) = self.config.max_file_size {
            let metadata = std::fs::metadata(path)
                .map_err(|e| EngineError::Io(e))?;
                
            if metadata.len() > max_size as u64 {
                return Err(EngineError::resource_limit(
                    format!("File size exceeds limit of {} bytes", max_size)
                ));
            }
        }
        
        // Use the CSV importer
        let importer = csv::EnhancedCsvImporter::new();
        let table_info = importer.import(path, options).await
            .map_err(|e| EngineError::csv_parse(e.to_string()))?;
        
        // TODO: Emit import success event via event_bus
        
        Ok(table_info)
    }
    
    /// Execute a query
    /// 
    /// Currently returns a placeholder result. This will be implemented
    /// when a proper query execution backend is added.
    pub async fn execute_query(
        &self, 
        _node_id: NodeId, 
        sql: String
    ) -> Result<QueryResult> {
        // Check cache first
        let cache_key = format!("sql_{}", &sql);
        
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok((*cached_result).clone());
            }
        }
        
        // TODO: Implement actual query execution
        // For now, return a placeholder result
        let result = QueryResult {
            columns: vec!["placeholder".to_string()],
            row_count: 0,
            execution_time_ms: 0,
            memory_used_bytes: None,
        };
        
        // Cache the result
        {
            let mut cache = self.cache.lock().await;
            cache.insert(sql, result.clone());
        }
        
        Ok(result)
    }
    
    /// Clear all cached results
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> cache::CacheStats {
        let cache = self.cache.lock().await;
        cache.stats()
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pika_core::events::EventBus;
    
    #[tokio::test]
    async fn test_engine_creation() {
        let event_bus = Arc::new(EventBus::new());
        let engine = Engine::new(event_bus);
        
        assert_eq!(engine.config().cache_size, 1000);
        assert_eq!(engine.config().max_file_size, Some(100 * 1024 * 1024));
    }
    
    #[tokio::test]
    async fn test_engine_with_custom_config() {
        let event_bus = Arc::new(EventBus::new());
        let config = EngineConfig {
            cache_size: 500,
            max_file_size: Some(50 * 1024 * 1024),
        };
        
        let engine = Engine::with_config(event_bus, config);
        
        assert_eq!(engine.config().cache_size, 500);
        assert_eq!(engine.config().max_file_size, Some(50 * 1024 * 1024));
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let event_bus = Arc::new(EventBus::new());
        let engine = Engine::new(event_bus);
        
        // Initially cache should be empty
        let stats = engine.cache_stats().await;
        assert_eq!(stats.entries, 0);
        
        // Clear cache should work even when empty
        engine.clear_cache().await;
        
        let stats = engine.cache_stats().await;
        assert_eq!(stats.entries, 0);
    }
}
