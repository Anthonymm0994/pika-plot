//! Memory management and coordination.

use pika_core::error::{PikaError, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Memory manager for tracking and limiting memory usage
pub struct MemoryManager {
    current_usage: Arc<AtomicUsize>,
    memory_limit: usize,
}

impl MemoryManager {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            current_usage: Arc::new(AtomicUsize::new(0)),
            memory_limit,
        }
    }
    
    /// Allocate memory with limit checking
    pub fn allocate(&self, size: usize) -> Result<MemoryAllocation> {
        let current = self.current_usage.load(Ordering::SeqCst);
        if current + size > self.memory_limit {
            return Err(PikaError::Memory(format!(
                "Memory allocation would exceed limit: {} + {} > {}",
                current, size, self.memory_limit
            )));
        }
        
        // Atomically update the current usage
        self.current_usage.fetch_add(size, Ordering::SeqCst);
        
        Ok(MemoryAllocation {
            size,
            manager: Arc::clone(&self.current_usage),
        })
    }
    
    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }
    
    /// Get memory limit
    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }
    
    /// Get available memory
    pub fn available_memory(&self) -> usize {
        self.memory_limit.saturating_sub(self.current_usage())
    }
    
    /// Check if allocation would exceed limit
    pub fn can_allocate(&self, size: usize) -> bool {
        self.current_usage() + size <= self.memory_limit
    }
}

/// Represents an allocated memory block
pub struct MemoryAllocation {
    size: usize,
    manager: Arc<AtomicUsize>,
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        self.manager.fetch_sub(self.size, Ordering::Relaxed);
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        // Default to 1GB memory limit
        Self::new(1024 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_allocation() {
        let coordinator = MemoryManager::new(1000);
        
        // Allocate within limit
        let alloc1 = coordinator.allocate(500).unwrap();
        assert_eq!(coordinator.current_usage(), 500);
        
        // Allocate more
        let alloc2 = coordinator.allocate(400).unwrap();
        assert_eq!(coordinator.current_usage(), 900);
        
        // Try to exceed limit
        assert!(coordinator.allocate(200).is_err());
        assert_eq!(coordinator.current_usage(), 900);
        
        // Release some by dropping alloc1
        drop(alloc1);
        assert_eq!(coordinator.current_usage(), 400);
        
        // Now we can allocate again
        let _alloc3 = coordinator.allocate(200).unwrap();
        assert_eq!(coordinator.current_usage(), 600);
        
        // Keep alloc2 alive to prevent it from being dropped
        drop(alloc2);
    }
    
    #[test]
    fn test_memory_guard() {
        let coordinator = Arc::new(MemoryManager::new(1000));
        
        {
            let _guard = coordinator.allocate(500).unwrap();
            assert_eq!(coordinator.current_usage(), 500);
        }
        
        // Memory should be released when guard is dropped
        assert_eq!(coordinator.current_usage(), 0);
    }
} 