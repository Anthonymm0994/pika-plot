//! Query caching module.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use pika_core::{
    types::QueryResult,
};

/// Query cache for storing results
pub struct QueryCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
}

#[derive(Clone)]
struct CacheEntry {
    result: Arc<QueryResult>,
    query: String,
}

impl QueryCache {
    /// Create a new query cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: capacity,
        }
    }
    
    /// Create a new query cache with memory limit (for backward compatibility)
    pub fn new_with_limit(_memory_limit: u64) -> Self {
        // For now, ignore memory limit and use default capacity
        Self::new(100)
    }
    
    /// Get a cached result if it exists
    pub fn get(&self, key: &str) -> Option<Arc<QueryResult>> {
        self.entries.get(key).map(|entry| entry.result.clone())
    }
    
    /// Insert a new result into the cache
    pub fn insert(&mut self, query: String, result: QueryResult) -> String {
        let key = format!("query_{}", Uuid::new_v4());
        
        // Simple eviction: remove oldest if at capacity
        if self.entries.len() >= self.max_entries {
            if let Some(first_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&first_key);
            }
        }
        
        self.entries.insert(key.clone(), CacheEntry {
            result: Arc::new(result),
            query,
        });
        
        key
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_operations() {
        let mut cache = QueryCache::new(2);
        
        let result = QueryResult {
            columns: vec!["test".to_string()],
            row_count: 1,
            execution_time_ms: 10,
            memory_used_bytes: None,
        };
        
        let key = cache.insert("SELECT * FROM test".to_string(), result.clone());
        assert_eq!(cache.len(), 1);
        
        let cached = cache.get(&key).unwrap();
        assert_eq!(cached.row_count, 1);
    }
} 