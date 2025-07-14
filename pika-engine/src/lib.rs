//! Pika plotting engine for data processing and visualization.

pub mod error;
pub mod import;
pub mod enhanced_csv;
pub mod cache;
pub mod query;
pub mod streaming;
pub mod workspace;

// TEMPORARILY DISABLED - These modules have heavy dependencies (polars/duckdb/arrow)
// We'll re-enable them once we resolve the dependency conflicts
/*
pub mod aggregation;
pub mod analysis;
pub mod database;
pub mod streaming_processor; 
pub mod memory_coordinator;
pub mod plot;
pub mod gpu;
pub mod spatial_indexing;
pub mod collaboration;
pub mod feature_engineering;
pub mod neural_networks;
pub mod predictive_analytics;
pub mod advanced_ml;
pub mod automated_insights;
pub mod advanced_visualization;
pub mod graph_analysis;
pub mod chaos_visualization;
pub mod jupyter_integration;
pub mod gpu_acceleration;
pub mod memory;
*/

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
        // Use the enhanced CSV importer
        let importer = enhanced_csv::EnhancedCsvImporter::new();
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
