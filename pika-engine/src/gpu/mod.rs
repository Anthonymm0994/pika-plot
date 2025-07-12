//! GPU acceleration module for plot rendering.

mod pipelines;
mod shaders;

pub use pipelines::{PlotPipelines, PlotVertex, PlotInstance, PlotUniforms};

use pika_core::{
    error::{PikaError, Result},
    events::RenderMode,
};
use std::sync::Arc;

/// GPU resource manager
pub struct GpuManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipelines: PlotPipelines,
}

impl GpuManager {
    /// Create a new GPU manager
    pub async fn new() -> Result<Self> {
        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| PikaError::GpuInitialization("No suitable GPU adapter found".to_string()))?;
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Pika-Plot GPU Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| PikaError::GpuInitialization(e.to_string()))?;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        // Create pipelines
        let pipelines = PlotPipelines::new(&device)?;
        
        Ok(GpuManager {
            device,
            queue,
            pipelines,
        })
    }
    
    /// Get the device
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }
    
    /// Get the queue
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
    
    /// Get the pipelines
    pub fn pipelines(&self) -> &PlotPipelines {
        &self.pipelines
    }
    
    /// Create a buffer
    pub fn create_buffer<T: bytemuck::Pod>(
        &self,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plot Data Buffer"),
            contents: bytemuck::cast_slice(data),
            usage,
        })
    }
    
    /// Determine render mode based on point count
    pub fn select_render_mode(&self, point_count: usize) -> RenderMode {
        match point_count {
            0..=10_000 => RenderMode::Direct,
            10_001..=100_000 => RenderMode::Instanced,
            _ => RenderMode::Aggregated,
        }
    }
} 