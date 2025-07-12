//! Memory management and coordination.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use pika_core::error::{PikaError, Result};

/// Memory coordinator for tracking and limiting memory usage
pub struct MemoryCoordinator {
    limit_bytes: Option<usize>,
    used_bytes: Arc<AtomicUsize>,
}

impl MemoryCoordinator {
    /// Create a new memory coordinator
    pub fn new(limit_bytes: Option<usize>) -> Self {
        MemoryCoordinator {
            limit_bytes,
            used_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Allocate memory, checking against limit
    pub fn allocate(&self, bytes: usize) -> Result<()> {
        if let Some(limit) = self.limit_bytes {
            let current = self.used_bytes.load(Ordering::Relaxed);
            if current + bytes > limit {
                return Err(PikaError::MemoryLimitExceeded(
                    format!("Would exceed limit: {} + {} > {}", current, bytes, limit)
                ));
            }
        }
        
        self.used_bytes.fetch_add(bytes, Ordering::Relaxed);
        Ok(())
    }
    
    /// Release memory
    pub fn release(&self, bytes: usize) {
        self.used_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
    
    /// Get current memory usage
    pub fn get_used(&self) -> usize {
        self.used_bytes.load(Ordering::Relaxed)
    }
    
    /// Get memory limit
    pub fn get_limit(&self) -> Option<usize> {
        self.limit_bytes
    }
    
    /// Set memory limit
    pub fn set_limit(&mut self, limit_bytes: Option<usize>) {
        self.limit_bytes = limit_bytes;
    }
}

/// RAII guard for memory allocation
pub struct MemoryGuard {
    coordinator: Arc<MemoryCoordinator>,
    bytes: usize,
}

impl MemoryGuard {
    /// Create a new memory guard
    pub fn new(coordinator: Arc<MemoryCoordinator>, bytes: usize) -> Result<Self> {
        coordinator.allocate(bytes)?;
        Ok(MemoryGuard { coordinator, bytes })
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        self.coordinator.release(self.bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_allocation() {
        let coordinator = MemoryCoordinator::new(Some(1000));
        
        // Allocate within limit
        assert!(coordinator.allocate(500).is_ok());
        assert_eq!(coordinator.get_used(), 500);
        
        // Allocate more
        assert!(coordinator.allocate(400).is_ok());
        assert_eq!(coordinator.get_used(), 900);
        
        // Try to exceed limit
        assert!(coordinator.allocate(200).is_err());
        assert_eq!(coordinator.get_used(), 900);
        
        // Release some
        coordinator.release(400);
        assert_eq!(coordinator.get_used(), 500);
        
        // Now allocation should succeed
        assert!(coordinator.allocate(200).is_ok());
        assert_eq!(coordinator.get_used(), 700);
    }
    
    #[test]
    fn test_memory_guard() {
        let coordinator = Arc::new(MemoryCoordinator::new(Some(1000)));
        
        {
            let _guard = MemoryGuard::new(coordinator.clone(), 500).unwrap();
            assert_eq!(coordinator.get_used(), 500);
        }
        
        // Memory should be released when guard is dropped
        assert_eq!(coordinator.get_used(), 0);
    }
} 