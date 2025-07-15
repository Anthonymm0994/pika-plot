//! Central memory coordinator for managing memory pressure.
//! Based on Gemini 2.5 Pro's unified memory management pattern.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::BinaryHeap;
use anyhow::Result;
use pika_core::types::NodeId;
use polars::prelude::*;
use std::collections::HashMap;
use std::collections::BTreeMap;

/// Item in the cache with associated cost for eviction.
#[derive(Debug, Clone)]
struct CachedItem {
    id: String,
    node_id: NodeId,
    size_bytes: usize,
    cost: f64,
    item_type: CacheItemType,
    is_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CacheItemType {
    QueryResult,
    GpuBuffer,
    PlotData,
}

impl CachedItem {
    fn calculate_cost(&self) -> f64 {
        let base_cost = self.size_bytes as f64;
        
        // Adjust cost based on type (GPU memory is more precious)
        let type_multiplier = match self.item_type {
            CacheItemType::QueryResult => 1.0,
            CacheItemType::GpuBuffer => 2.0,
            CacheItemType::PlotData => 1.5,
        };
        
        // Lower priority for non-visible items
        let visibility_multiplier = if self.is_visible { 1.0 } else { 0.5 };
        
        base_cost * type_multiplier * visibility_multiplier
    }
}

// For min-heap (lowest cost = highest priority for eviction)
impl Ord for CachedItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost.partial_cmp(&other.cost).unwrap().reverse()
    }
}

impl PartialOrd for CachedItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CachedItem {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for CachedItem {}

/// Central memory coordinator for the application.
pub struct MemoryCoordinator {
    /// Total memory budget in bytes
    total_budget: usize,
    
    /// Current RAM usage
    ram_used: AtomicUsize,
    
    /// Current VRAM usage
    vram_used: AtomicUsize,
    
    /// DuckDB memory limit
    duckdb_limit: AtomicUsize,
    
    /// Priority queue of cached items (min-heap by cost)
    cached_items: Arc<Mutex<BinaryHeap<CachedItem>>>,
    
    /// Eviction callbacks
    eviction_callbacks: Arc<Mutex<Vec<Box<dyn Fn(&str) + Send + Sync>>>>,

    indices: HashMap<String, BTreeMap<String, Vec<usize>>>
}

impl MemoryCoordinator {
    pub fn new(total_system_ram: usize) -> Self {
        // Reserve 2GB for OS/other apps as Gemini suggested
        let reserved = 2 * 1024 * 1024 * 1024;
        let app_budget = total_system_ram.saturating_sub(reserved);
        
        // 60/40 split between DuckDB and GPU as recommended
        let duckdb_limit = (app_budget as f64 * 0.6) as usize;
        
        Self {
            total_budget: app_budget,
            ram_used: AtomicUsize::new(0),
            vram_used: AtomicUsize::new(0),
            duckdb_limit: AtomicUsize::new(duckdb_limit),
            cached_items: Arc::new(Mutex::new(BinaryHeap::new())),
            eviction_callbacks: Arc::new(Mutex::new(Vec::new())),
            indices: HashMap::new()
        }
    }
    
    /// Configure DuckDB with the current memory limit.
    pub fn configure_duckdb(&self, conn: &duckdb::Connection) -> Result<()> {
        let limit_mb = self.duckdb_limit.load(Ordering::Relaxed) / 1024 / 1024;
        conn.execute(&format!("SET memory_limit='{}MB'", limit_mb), [])?;
        Ok(())
    }
    
    /// Register RAM usage for a cached item.
    pub fn register_ram_usage(&self, id: String, node_id: NodeId, size: usize) {
        self.ram_used.fetch_add(size, Ordering::Relaxed);
        
        let item = CachedItem {
            id,
            node_id,
            size_bytes: size,
            cost: 0.0, // Will be calculated
            item_type: CacheItemType::QueryResult,
            is_visible: true,
        };
        
        let mut items = self.cached_items.lock().unwrap();
        let mut item = item;
        item.cost = item.calculate_cost();
        items.push(item);
    }
    
    /// Register VRAM usage for a GPU buffer.
    pub fn register_vram_usage(&self, id: String, node_id: NodeId, size: usize) {
        self.vram_used.fetch_add(size, Ordering::Relaxed);
        
        let item = CachedItem {
            id,
            node_id,
            size_bytes: size,
            cost: 0.0, // Will be calculated
            item_type: CacheItemType::GpuBuffer,
            is_visible: true,
        };
        
        let mut items = self.cached_items.lock().unwrap();
        let mut item = item;
        item.cost = item.calculate_cost();
        items.push(item);
    }
    
    /// Update visibility status of items for a node.
    pub fn update_visibility(&self, node_id: NodeId, is_visible: bool) {
        let mut items = self.cached_items.lock().unwrap();
        
        // Rebuild heap with updated costs
        let updated_items: Vec<_> = items.drain()
            .map(|mut item| {
                if item.node_id == node_id {
                    item.is_visible = is_visible;
                    item.cost = item.calculate_cost();
                }
                item
            })
            .collect();
        
        *items = updated_items.into_iter().collect();
    }
    
    /// Check if we can allocate the requested memory.
    pub fn can_allocate(&self, size: usize) -> bool {
        let current_usage = self.ram_used.load(Ordering::Relaxed) 
            + self.vram_used.load(Ordering::Relaxed);
        
        current_usage + size <= self.total_budget
    }
    
    /// Try to allocate memory, triggering eviction if needed.
    pub async fn allocate_with_eviction(&self, size: usize) -> Result<()> {
        while !self.can_allocate(size) {
            // Evict lowest-cost item
            let item_to_evict = {
                let mut items = self.cached_items.lock().unwrap();
                items.pop()
            };
            
            if let Some(item) = item_to_evict {
                // Call eviction callbacks
                let callbacks = self.eviction_callbacks.lock().unwrap();
                for callback in callbacks.iter() {
                    callback(&item.id);
                }
                
                // Update usage counters
                match item.item_type {
                    CacheItemType::QueryResult => {
                        self.ram_used.fetch_sub(item.size_bytes, Ordering::Relaxed);
                    }
                    CacheItemType::GpuBuffer | CacheItemType::PlotData => {
                        self.vram_used.fetch_sub(item.size_bytes, Ordering::Relaxed);
                    }
                }
            } else {
                anyhow::bail!("Cannot allocate {} bytes: no items to evict", size);
            }
        }
        
        Ok(())
    }
    
    /// Rebalance memory between DuckDB and GPU based on pressure.
    pub fn rebalance(&self, gpu_pressure: f32, db_pressure: f32) {
        if gpu_pressure > 0.9 && db_pressure < 0.5 {
            // Shift memory from DB to GPU
            let current = self.duckdb_limit.load(Ordering::Relaxed);
            let new_limit = (current as f64 * 0.8) as usize;
            self.duckdb_limit.store(new_limit, Ordering::Relaxed);
        } else if db_pressure > 0.9 && gpu_pressure < 0.5 {
            // Shift memory from GPU to DB
            let current = self.duckdb_limit.load(Ordering::Relaxed);
            let max_limit = (self.total_budget as f64 * 0.7) as usize;
            let new_limit = std::cmp::min((current as f64 * 1.2) as usize, max_limit);
            self.duckdb_limit.store(new_limit, Ordering::Relaxed);
        }
    }
    
    /// Register an eviction callback.
    pub fn on_eviction<F>(&self, callback: F) 
    where 
        F: Fn(&str) + Send + Sync + 'static 
    {
        let mut callbacks = self.eviction_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }
    
    /// Get current memory statistics.
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_budget: self.total_budget,
            ram_used: self.ram_used.load(Ordering::Relaxed),
            vram_used: self.vram_used.load(Ordering::Relaxed),
            duckdb_limit: self.duckdb_limit.load(Ordering::Relaxed),
            pressure: self.calculate_pressure(),
        }
    }
    
    fn calculate_pressure(&self) -> f32 {
        let used = self.ram_used.load(Ordering::Relaxed) 
            + self.vram_used.load(Ordering::Relaxed);
        (used as f32 / self.total_budget as f32).min(1.0)
    }
    
    /// Get simple memory info for UI display.
    pub fn get_memory_info(&self) -> MemoryInfo {
        let used = self.ram_used.load(Ordering::Relaxed) 
            + self.vram_used.load(Ordering::Relaxed);
        MemoryInfo {
            used_mb: used / 1024 / 1024,
            total_mb: self.total_budget / 1024 / 1024,
        }
    }
    
    /// Update memory pressure (called periodically).
    pub fn update_memory_pressure(&self) {
        // This could trigger events or warnings based on pressure
        let pressure = self.calculate_pressure();
        if pressure > 0.9 {
            tracing::warn!("Memory pressure high: {:.1}%", pressure * 100.0);
        }
    }

    pub fn index_column(&mut self, table: &str, col: &str, values: &Series) {
        let mut map = BTreeMap::new();
        for (idx, val) in values.iter().enumerate() {
            let key = val.to_string();
            map.entry(key).or_insert(Vec::new()).push(idx);
        }
        self.indices.insert(format!("{table}_{col}"), map);
    }
    pub fn query_index(&self, table: &str, col: &str, value: &AnyValue) -> Option<Vec<usize>> {
        let key = format!("{table}_{col}");
        self.indices.get(&key).and_then(|m| m.get(&value.to_string()).cloned())
    }
}

/// Memory usage statistics.
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_budget: usize,
    pub ram_used: usize,
    pub vram_used: usize,
    pub duckdb_limit: usize,
    pub pressure: f32,
}

/// Simple memory info for UI display
#[derive(Debug, Clone, Copy)]
pub struct MemoryInfo {
    pub used_mb: usize,
    pub total_mb: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_memory_allocation() {
        let coordinator = MemoryCoordinator::new(8 * 1024 * 1024 * 1024); // 8GB
        
        let node_id = NodeId(Uuid::new_v4());
        
        // Register some RAM usage
        coordinator.register_ram_usage("query1".to_string(), node_id, 1024 * 1024);
        assert_eq!(coordinator.stats().ram_used, 1024 * 1024);
        
        // Register some VRAM usage
        coordinator.register_vram_usage("buffer1".to_string(), node_id, 2 * 1024 * 1024);
        assert_eq!(coordinator.stats().vram_used, 2 * 1024 * 1024);
    }
    
    #[test]
    fn test_cost_calculation() {
        let query_item = CachedItem {
            id: "test".to_string(),
            node_id: NodeId(Uuid::new_v4()),
            size_bytes: 1000,
            cost: 0.0,
            item_type: CacheItemType::QueryResult,
            is_visible: true,
        };
        
        assert_eq!(query_item.calculate_cost(), 1000.0);
        
        let gpu_item = CachedItem {
            id: "test".to_string(),
            node_id: NodeId(Uuid::new_v4()),
            size_bytes: 1000,
            cost: 0.0,
            item_type: CacheItemType::GpuBuffer,
            is_visible: true,
        };
        
        assert_eq!(gpu_item.calculate_cost(), 2000.0); // 2x multiplier
    }
} 