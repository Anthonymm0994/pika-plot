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
    /// Track query hash to cache key mapping for deduplication
    query_hash_map: HashMap<u64, String>,
}

#[derive(Clone)]
struct CacheEntry {
    result: Arc<QueryResult>,
    query: String,
    query_hash: u64,
    access_count: u32,
}

impl QueryCache {
    /// Create a new query cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: capacity,
            query_hash_map: HashMap::new(),
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
    
    /// Get cached result by query (deduplication support)
    pub fn get_by_query(&mut self, query: &str) -> Option<Arc<QueryResult>> {
        let query_hash = Self::hash_query(query);
        
        if let Some(cache_key) = self.query_hash_map.get(&query_hash) {
            if let Some(entry) = self.entries.get_mut(cache_key) {
                entry.access_count += 1;
                return Some(entry.result.clone());
            }
        }
        None
    }
    
    /// Insert a new result into the cache
    pub fn insert(&mut self, query: String, result: QueryResult) -> String {
        let query_hash = Self::hash_query(&query);
        
        // Check if we already have this query cached (deduplication)
        if let Some(existing_key) = self.query_hash_map.get(&query_hash) {
            return existing_key.clone();
        }
        
        let key = format!("query_{}", Uuid::new_v4());
        
        // Smart eviction: remove least recently accessed if at capacity
        if self.entries.len() >= self.max_entries {
            self.evict_least_accessed();
        }
        
        self.query_hash_map.insert(query_hash, key.clone());
        self.entries.insert(key.clone(), CacheEntry {
            result: Arc::new(result),
            query,
            query_hash,
            access_count: 1,
        });
        
        key
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.query_hash_map.clear();
    }
    
    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Invalidate cache entries that might be affected by table changes
    pub fn invalidate_table(&mut self, table_name: &str) {
        let mut keys_to_remove = Vec::new();
        
        for (key, entry) in &self.entries {
            // Simple check: if query mentions the table, invalidate it
            if entry.query.to_lowercase().contains(&table_name.to_lowercase()) {
                keys_to_remove.push(key.clone());
            }
        }
        
        for key in keys_to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.query_hash_map.remove(&entry.query_hash);
            }
        }
    }
    
    /// Hash a query string for deduplication
    fn hash_query(query: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        // Normalize query by trimming and lowercasing
        query.trim().to_lowercase().hash(&mut hasher);
        hasher.finish()
    }
    
    /// Evict the least recently accessed entry
    fn evict_least_accessed(&mut self) {
        if let Some((key, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, e)| (k.clone(), e.query_hash)) {
            
            if let Some(entry) = self.entries.remove(&key) {
                self.query_hash_map.remove(&entry.query_hash);
            }
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_hits: u32 = self.entries.values()
            .map(|e| e.access_count)
            .sum();
        
        CacheStats {
            total_entries: self.entries.len(),
            total_hits,
            capacity: self.max_entries,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u32,
    pub capacity: usize,
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
    
    #[test]
    fn test_query_deduplication() {
        let mut cache = QueryCache::new(10);
        
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            row_count: 5,
            execution_time_ms: 20,
            memory_used_bytes: Some(1024),
        };
        
        // Insert same query twice
        let key1 = cache.insert("SELECT * FROM users".to_string(), result.clone());
        let key2 = cache.insert("SELECT * FROM users".to_string(), result.clone());
        
        // Should return the same key (deduplication)
        assert_eq!(key1, key2);
        assert_eq!(cache.len(), 1);
        
        // Test case insensitive deduplication
        let key3 = cache.insert("select * from users".to_string(), result.clone());
        assert_eq!(key1, key3);
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut cache = QueryCache::new(10);
        
        let result = QueryResult {
            columns: vec!["col1".to_string()],
            row_count: 10,
            execution_time_ms: 5,
            memory_used_bytes: None,
        };
        
        cache.insert("SELECT * FROM users".to_string(), result.clone());
        cache.insert("SELECT * FROM posts".to_string(), result.clone());
        cache.insert("SELECT * FROM users JOIN posts".to_string(), result.clone());
        
        assert_eq!(cache.len(), 3);
        
        // Invalidate entries related to 'users' table
        cache.invalidate_table("users");
        
        // Should only have the posts query left
        assert_eq!(cache.len(), 1);
    }
} 