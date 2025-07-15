//! Query caching module with LRU eviction policy.
//!
//! This module provides a simple Least Recently Used (LRU) cache
//! for storing query results to improve performance.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;
use pika_core::types::QueryResult;

/// Maximum default cache capacity
const DEFAULT_MAX_ENTRIES: usize = 1000;

/// Cache key type for better type safety
pub type CacheKey = String;

/// Simple LRU query cache for storing results
/// 
/// Note: This implementation is not thread-safe. If concurrent access
/// is needed, wrap in a Mutex or RwLock.
/// 
/// FIXME: Consider making this thread-safe by default using parking_lot::RwLock
/// to avoid forcing users to handle synchronization.
#[derive(Debug)]
pub struct QueryCache {
    /// Map from cache key to cache entry
    entries: HashMap<CacheKey, CacheEntry>,
    /// Queue to track access order (most recent at back)
    access_order: VecDeque<CacheKey>,
    /// Maximum number of entries to store
    max_entries: usize,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    result: Arc<QueryResult>,
    query: String,
}

impl QueryCache {
    /// Create a new query cache with specified capacity
    /// 
    /// # Panics
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        
        Self {
            entries: HashMap::with_capacity(capacity),
            access_order: VecDeque::with_capacity(capacity),
            max_entries: capacity,
        }
    }
    
    /// Create a new query cache with default capacity
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_MAX_ENTRIES)
    }
    
    /// Get a cached result if it exists
    /// 
    /// This marks the entry as recently used, moving it to the end
    /// of the LRU queue.
    pub fn get(&mut self, key: &str) -> Option<Arc<QueryResult>> {
        if let Some(entry) = self.entries.get(key) {
            // Update access order - remove and re-add at the back
            self.update_access_order(key);
            Some(entry.result.clone())
        } else {
            None
        }
    }
    
    /// Insert a new result into the cache
    /// 
    /// Returns the generated cache key for the entry.
    /// If the cache is at capacity, the least recently used entry is evicted.
    pub fn insert(&mut self, query: String, result: QueryResult) -> CacheKey {
        let key = Self::generate_cache_key();
        
        // Check if we need to evict
        if self.entries.len() >= self.max_entries {
            self.evict_lru();
        }
        
        debug_assert!(self.entries.len() < self.max_entries);
        
        // Insert the new entry
        self.entries.insert(key.clone(), CacheEntry {
            result: Arc::new(result),
            query,
        });
        
        // Add to access order
        self.access_order.push_back(key.clone());
        
        key
    }
    
    /// Remove a specific entry from the cache
    pub fn remove(&mut self, key: &str) -> Option<Arc<QueryResult>> {
        if let Some(entry) = self.entries.remove(key) {
            // Remove from access order
            self.access_order.retain(|k| k != key);
            Some(entry.result)
        } else {
            None
        }
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
    
    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.entries.len(), self.access_order.len());
        self.entries.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get the maximum capacity of the cache
    pub fn capacity(&self) -> usize {
        self.max_entries
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.len(),
            capacity: self.max_entries,
            hit_rate: 0.0, // TODO: Track hits and misses
        }
    }
    
    /// Update the access order for a key
    fn update_access_order(&mut self, key: &str) {
        // Remove from current position
        self.access_order.retain(|k| k != key);
        // Add to the back (most recently used)
        self.access_order.push_back(key.to_string());
    }
    
    /// Evict the least recently used entry
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.access_order.pop_front() {
            self.entries.remove(&lru_key);
        }
    }
    
    /// Generate a unique cache key
    fn generate_cache_key() -> CacheKey {
        format!("query_{}", Uuid::new_v4())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_result(columns: Vec<&str>, row_count: usize) -> QueryResult {
        QueryResult {
            columns: columns.into_iter().map(|s| s.to_string()).collect(),
            row_count,
            execution_time_ms: 10,
            memory_used_bytes: None,
        }
    }
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = QueryCache::new(2);
        
        let result = create_test_result(vec!["test"], 1);
        let key = cache.insert("SELECT * FROM test".to_string(), result.clone());
        
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
        
        let cached = cache.get(&key).unwrap();
        assert_eq!(cached.row_count, 1);
        assert_eq!(cached.columns, vec!["test"]);
    }
    
    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = QueryCache::new(3);
        
        // Insert 3 items
        let key1 = cache.insert("query1".to_string(), create_test_result(vec!["col1"], 1));
        let key2 = cache.insert("query2".to_string(), create_test_result(vec!["col2"], 2));
        let key3 = cache.insert("query3".to_string(), create_test_result(vec!["col3"], 3));
        
        assert_eq!(cache.len(), 3);
        
        // Access key1 to make it recently used
        assert!(cache.get(&key1).is_some());
        
        // Insert a 4th item - should evict key2 (least recently used)
        let key4 = cache.insert("query4".to_string(), create_test_result(vec!["col4"], 4));
        
        assert_eq!(cache.len(), 3);
        assert!(cache.get(&key1).is_some()); // Should still exist
        assert!(cache.get(&key2).is_none()); // Should be evicted
        assert!(cache.get(&key3).is_some()); // Should still exist
        assert!(cache.get(&key4).is_some()); // Should exist
    }
    
    #[test]
    fn test_cache_remove() {
        let mut cache = QueryCache::new(5);
        
        let key1 = cache.insert("query1".to_string(), create_test_result(vec!["col1"], 1));
        let key2 = cache.insert("query2".to_string(), create_test_result(vec!["col2"], 2));
        
        assert_eq!(cache.len(), 2);
        
        let removed = cache.remove(&key1);
        assert!(removed.is_some());
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
    }
    
    #[test]
    fn test_cache_clear() {
        let mut cache = QueryCache::new(10);
        
        cache.insert("query1".to_string(), create_test_result(vec!["col1"], 1));
        cache.insert("query2".to_string(), create_test_result(vec!["col2"], 2));
        cache.insert("query3".to_string(), create_test_result(vec!["col3"], 3));
        
        assert_eq!(cache.len(), 3);
        
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
    
    #[test]
    fn test_cache_capacity() {
        let cache = QueryCache::new(100);
        assert_eq!(cache.capacity(), 100);
        
        let default_cache = QueryCache::with_default_capacity();
        assert_eq!(default_cache.capacity(), DEFAULT_MAX_ENTRIES);
    }
    
    #[test]
    #[should_panic(expected = "Cache capacity must be greater than 0")]
    fn test_cache_zero_capacity() {
        QueryCache::new(0);
    }
    
    #[test]
    fn test_cache_stats() {
        let mut cache = QueryCache::new(10);
        
        cache.insert("query1".to_string(), create_test_result(vec!["col1"], 1));
        cache.insert("query2".to_string(), create_test_result(vec!["col2"], 2));
        
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.capacity, 10);
    }
    
    #[test]
    fn test_cache_access_order_updates() {
        let mut cache = QueryCache::new(3);
        
        // Insert items in order: 1, 2, 3
        let key1 = cache.insert("query1".to_string(), create_test_result(vec!["col1"], 1));
        let key2 = cache.insert("query2".to_string(), create_test_result(vec!["col2"], 2));
        let key3 = cache.insert("query3".to_string(), create_test_result(vec!["col3"], 3));
        
        // At this point, LRU order is: 1, 2, 3 (1 is least recently used)
        
        // Access key1 to make it most recently used
        assert!(cache.get(&key1).is_some());
        
        // Now LRU order is: 2, 3, 1 (2 is least recently used)
        
        // Insert a 4th item - should evict key2 (least recently used)
        let key4 = cache.insert("query4".to_string(), create_test_result(vec!["col4"], 4));
        
        assert_eq!(cache.len(), 3);
        assert!(cache.get(&key1).is_some()); // Should still exist (recently accessed)
        assert!(cache.get(&key2).is_none()); // Should be evicted (least recently used)
        assert!(cache.get(&key3).is_some()); // Should still exist
        assert!(cache.get(&key4).is_some()); // Should exist (just inserted)
    }
} 