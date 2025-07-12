//! Mock GPU implementation for testing without hardware.

use std::sync::{Arc, Mutex};
use wgpu::{BufferDescriptor, ShaderModuleDescriptor};
use anyhow::Result;

/// Mock GPU device for testing.
pub struct MockGpuDevice {
    buffers_created: Arc<Mutex<Vec<BufferInfo>>>,
    shaders_compiled: Arc<Mutex<Vec<String>>>,
    memory_used: Arc<Mutex<u64>>,
    memory_limit: u64,
    fail_on_shader_compile: bool,
}

#[derive(Debug, Clone)]
struct BufferInfo {
    size: u64,
    usage: wgpu::BufferUsages,
    label: Option<String>,
}

impl MockGpuDevice {
    pub fn new() -> Self {
        Self {
            buffers_created: Arc::new(Mutex::new(Vec::new())),
            shaders_compiled: Arc::new(Mutex::new(Vec::new())),
            memory_used: Arc::new(Mutex::new(0)),
            memory_limit: 4 * 1024 * 1024 * 1024, // 4GB default
            fail_on_shader_compile: false,
        }
    }
    
    pub fn with_memory_limit(mut self, limit: u64) -> Self {
        self.memory_limit = limit;
        self
    }
    
    pub fn with_shader_compile_failure(mut self) -> Self {
        self.fail_on_shader_compile = true;
        self
    }
    
    pub fn create_buffer(&self, desc: &BufferDescriptor) -> Result<MockBuffer> {
        let size = desc.size;
        
        // Check memory limit
        let mut memory = self.memory_used.lock().unwrap();
        if *memory + size > self.memory_limit {
            anyhow::bail!("GPU memory limit exceeded");
        }
        
        *memory += size;
        
        // Record buffer creation
        self.buffers_created.lock().unwrap().push(BufferInfo {
            size,
            usage: desc.usage,
            label: desc.label.map(|s| s.to_string()),
        });
        
        Ok(MockBuffer {
            size,
            data: vec![0u8; size as usize],
            device: self.clone(),
        })
    }
    
    pub fn create_shader_module(&self, desc: &ShaderModuleDescriptor) -> Result<MockShaderModule> {
        if self.fail_on_shader_compile {
            anyhow::bail!("Shader compilation failed (mock error)");
        }
        
        let source = match &desc.source {
            wgpu::ShaderSource::Wgsl(code) => code.to_string(),
            _ => "unknown".to_string(),
        };
        
        self.shaders_compiled.lock().unwrap().push(source.clone());
        
        Ok(MockShaderModule {
            source,
            label: desc.label.map(|s| s.to_string()),
        })
    }
    
    /// Get total memory used.
    pub fn memory_used(&self) -> u64 {
        *self.memory_used.lock().unwrap()
    }
    
    /// Get number of buffers created.
    pub fn buffers_created_count(&self) -> usize {
        self.buffers_created.lock().unwrap().len()
    }
    
    /// Get shaders compiled.
    pub fn shaders_compiled(&self) -> Vec<String> {
        self.shaders_compiled.lock().unwrap().clone()
    }
}

impl Clone for MockGpuDevice {
    fn clone(&self) -> Self {
        Self {
            buffers_created: self.buffers_created.clone(),
            shaders_compiled: self.shaders_compiled.clone(),
            memory_used: self.memory_used.clone(),
            memory_limit: self.memory_limit,
            fail_on_shader_compile: self.fail_on_shader_compile,
        }
    }
}

/// Mock GPU buffer.
pub struct MockBuffer {
    size: u64,
    data: Vec<u8>,
    device: MockGpuDevice,
}

impl MockBuffer {
    pub fn write(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        let end = offset + data.len() as u64;
        if end > self.size {
            anyhow::bail!("Buffer write out of bounds");
        }
        
        self.data[offset as usize..end as usize].copy_from_slice(data);
        Ok(())
    }
    
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let end = offset + len;
        if end > self.size {
            anyhow::bail!("Buffer read out of bounds");
        }
        
        Ok(self.data[offset as usize..end as usize].to_vec())
    }
}

impl Drop for MockBuffer {
    fn drop(&mut self) {
        // Release memory
        let mut memory = self.device.memory_used.lock().unwrap();
        *memory -= self.size;
    }
}

/// Mock shader module.
pub struct MockShaderModule {
    source: String,
    label: Option<String>,
}

/// Mock GPU aggregator for testing.
pub struct MockGpuAggregator {
    device: MockGpuDevice,
}

impl MockGpuAggregator {
    pub fn new(device: MockGpuDevice) -> Self {
        Self { device }
    }
    
    pub async fn aggregate_sum(&self, data: &[f32]) -> Result<f32> {
        // Simulate GPU aggregation
        Ok(data.iter().sum())
    }
    
    pub async fn aggregate_avg(&self, data: &[f32]) -> Result<f32> {
        if data.is_empty() {
            return Ok(0.0);
        }
        Ok(data.iter().sum::<f32>() / data.len() as f32)
    }
    
    pub async fn aggregate_min_max(&self, data: &[f32]) -> Result<(f32, f32)> {
        if data.is_empty() {
            anyhow::bail!("Cannot compute min/max of empty dataset");
        }
        
        let min = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        Ok((min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_gpu_memory_limit() {
        let device = MockGpuDevice::new()
            .with_memory_limit(1024 * 1024); // 1MB limit
        
        // Create buffer within limit
        let desc = BufferDescriptor {
            label: Some("test"),
            size: 512 * 1024, // 512KB
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        };
        
        let buffer1 = device.create_buffer(&desc).unwrap();
        assert_eq!(device.memory_used(), 512 * 1024);
        
        // Try to exceed limit
        let desc2 = BufferDescriptor {
            label: Some("test2"),
            size: 600 * 1024, // 600KB - would exceed limit
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        };
        
        assert!(device.create_buffer(&desc2).is_err());
        
        // Drop first buffer to free memory
        drop(buffer1);
        assert_eq!(device.memory_used(), 0);
        
        // Now it should work
        let _buffer2 = device.create_buffer(&desc2).unwrap();
        assert_eq!(device.memory_used(), 600 * 1024);
    }
    
    #[test]
    fn test_mock_shader_compilation() {
        let device = MockGpuDevice::new();
        
        let desc = ShaderModuleDescriptor {
            label: Some("test_shader"),
            source: wgpu::ShaderSource::Wgsl("// Test shader code".into()),
        };
        
        let shader = device.create_shader_module(&desc).unwrap();
        assert_eq!(device.shaders_compiled(), vec!["// Test shader code"]);
        
        // Test failure mode
        let device_fail = MockGpuDevice::new()
            .with_shader_compile_failure();
        
        assert!(device_fail.create_shader_module(&desc).is_err());
    }
    
    #[tokio::test]
    async fn test_mock_gpu_aggregation() {
        let device = MockGpuDevice::new();
        let aggregator = MockGpuAggregator::new(device);
        
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let sum = aggregator.aggregate_sum(&data).await.unwrap();
        assert_eq!(sum, 15.0);
        
        let avg = aggregator.aggregate_avg(&data).await.unwrap();
        assert_eq!(avg, 3.0);
        
        let (min, max) = aggregator.aggregate_min_max(&data).await.unwrap();
        assert_eq!(min, 1.0);
        assert_eq!(max, 5.0);
    }
} 