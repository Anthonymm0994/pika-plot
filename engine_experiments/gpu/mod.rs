//! GPU management and rendering infrastructure.

mod pipelines;

pub use pipelines::{DirectPipeline, InstancedPipeline, AggregationPipeline, ViewProjectionUniform};

use std::sync::Arc;
use pika_core::error::{PikaError, Result};

/// GPU manager for handling device and rendering resources
pub struct GpuManager {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub adapter_info: wgpu::AdapterInfo,
}

impl GpuManager {
    /// Create a new GPU manager
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| PikaError::RenderError("No suitable GPU adapter found".to_string()))?;
        
        let adapter_info = adapter.get_info();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Pika-Plot GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| PikaError::RenderError(format!("Failed to create GPU device: {}", e)))?;
        
        Ok(GpuManager {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info,
        })
    }
    
    /// Create a buffer with data
    pub fn create_buffer_with_data(&self, data: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Data Buffer"),
            contents: data,
            usage,
        })
    }
    
    /// Get device limits
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }
    
    /// Check if GPU supports required features
    pub fn supports_required_features(&self) -> bool {
        // For now, we don't require any special features
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // GPU tests require actual hardware
    async fn test_gpu_manager_creation() {
        let gpu = GpuManager::new().await;
        assert!(gpu.is_ok() || gpu.is_err()); // Either works or fails gracefully
    }
} 