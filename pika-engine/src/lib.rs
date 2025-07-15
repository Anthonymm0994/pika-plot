//! Engine module for data processing and visualization.

#![warn(missing_docs)]

pub mod cache;
pub mod csv;  // Renamed from enhanced_csv
pub mod query;
pub mod database;
pub mod plot;  // Plot data extraction and rendering
pub mod error;

use std::sync::Arc;
use tokio::sync::Mutex;
use pika_core::{
    events::{EventBus},
    error::Result,
    types::{TableInfo, ImportOptions, NodeId},
};

use crate::error::EngineError;

/// Main engine for data processing
pub struct Engine {
    event_bus: Arc<EventBus>,
    cache: Arc<Mutex<cache::QueryCache>>,
}

impl Engine {
    /// Create a new engine instance
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let cache = Arc::new(Mutex::new(cache::QueryCache::new(100)));
        
        Self {
            event_bus,
            cache,
        }
    }
    
    /// Import a CSV file
    pub async fn import_csv(&self, path: &str, options: ImportOptions, _node_id: NodeId) -> Result<TableInfo> {
        // Use the CSV importer
        let importer = csv::EnhancedCsvImporter::new();
        let table_info = importer.import(path, options).await?;
        
        Ok(table_info)
    }
    
    /// Execute a query
    pub async fn execute_query(&self, _node_id: NodeId, _sql: String) -> Result<pika_core::types::QueryResult> {
        // For now, return a placeholder result
        Ok(pika_core::types::QueryResult {
            columns: vec!["placeholder".to_string()],
            row_count: 0,
            execution_time_ms: 0,
            memory_used_bytes: None,
        })
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }
}
