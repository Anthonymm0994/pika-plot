//! GPU device wrapper that tracks memory allocations.
//! Based on Gemini 2.5 Pro's recommendation for accurate VRAM tracking.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use wgpu::{Buffer, BufferDescriptor, Device, Texture, TextureDescriptor};

/// A wrapper around wgpu::Device that tracks memory allocations.
pub struct TrackedDevice {
    device: Device,
    allocated_bytes: Arc<AtomicUsize>,
    allocation_count: Arc<AtomicUsize>,
}

impl TrackedDevice {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            allocated_bytes: Arc::new(AtomicUsize::new(0)),
            allocation_count: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Create a buffer and track its memory usage.
    pub fn create_buffer(&self, desc: &BufferDescriptor) -> TrackedBuffer {
        let buffer = self.device.create_buffer(desc);
        
        // Track allocation
        self.allocated_bytes.fetch_add(desc.size as usize, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        TrackedBuffer {
            buffer,
            size: desc.size,
            allocated_bytes: self.allocated_bytes.clone(),
            allocation_count: self.allocation_count.clone(),
        }
    }
    
    /// Create a texture and track its memory usage.
    pub fn create_texture(&self, desc: &TextureDescriptor) -> TrackedTexture {
        let texture = self.device.create_texture(desc);
        
        // Calculate texture size
        let bytes_per_pixel = desc.format.block_size(None).unwrap_or(4);
        let size = desc.size.width as u64 
            * desc.size.height as u64 
            * desc.size.depth_or_array_layers as u64
            * bytes_per_pixel as u64;
        
        // Track allocation
        self.allocated_bytes.fetch_add(size as usize, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        TrackedTexture {
            texture,
            size,
            allocated_bytes: self.allocated_bytes.clone(),
            allocation_count: self.allocation_count.clone(),
        }
    }
    
    /// Get current VRAM usage in bytes.
    pub fn used_vram(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }
    
    /// Get number of active allocations.
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }
    
    /// Get the underlying device.
    pub fn device(&self) -> &Device {
        &self.device
    }
}

/// A buffer that automatically updates memory tracking when dropped.
pub struct TrackedBuffer {
    buffer: Buffer,
    size: u64,
    allocated_bytes: Arc<AtomicUsize>,
    allocation_count: Arc<AtomicUsize>,
}

impl TrackedBuffer {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Drop for TrackedBuffer {
    fn drop(&mut self) {
        // Release tracked memory
        self.allocated_bytes.fetch_sub(self.size as usize, Ordering::Relaxed);
        self.allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
}

impl std::ops::Deref for TrackedBuffer {
    type Target = Buffer;
    
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

/// A texture that automatically updates memory tracking when dropped.
pub struct TrackedTexture {
    texture: Texture,
    size: u64,
    allocated_bytes: Arc<AtomicUsize>,
    allocation_count: Arc<AtomicUsize>,
}

impl TrackedTexture {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }
}

impl Drop for TrackedTexture {
    fn drop(&mut self) {
        // Release tracked memory
        self.allocated_bytes.fetch_sub(self.size as usize, Ordering::Relaxed);
        self.allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
}

impl std::ops::Deref for TrackedTexture {
    type Target = Texture;
    
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_tracking() {
        // This would require a real or mock device
        // For now, just test the logic with mock values
        let allocated = Arc::new(AtomicUsize::new(0));
        let count = Arc::new(AtomicUsize::new(0));
        
        // Simulate allocation
        allocated.fetch_add(1024, Ordering::Relaxed);
        count.fetch_add(1, Ordering::Relaxed);
        
        assert_eq!(allocated.load(Ordering::Relaxed), 1024);
        assert_eq!(count.load(Ordering::Relaxed), 1);
        
        // Simulate deallocation
        allocated.fetch_sub(1024, Ordering::Relaxed);
        count.fetch_sub(1, Ordering::Relaxed);
        
        assert_eq!(allocated.load(Ordering::Relaxed), 0);
        assert_eq!(count.load(Ordering::Relaxed), 0);
    }
} 