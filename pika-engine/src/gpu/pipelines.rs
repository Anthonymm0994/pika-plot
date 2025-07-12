//! GPU rendering pipelines for plots.

use pika_core::error::Result;
use bytemuck::{Pod, Zeroable};

/// Vertex data for plot points
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PlotVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl PlotVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];
    
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PlotVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Instance data for instanced rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PlotInstance {
    pub position: [f32; 2],
    pub size: f32,
    pub color: [f32; 4],
}

impl PlotInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x2,
        3 => Float32,
        4 => Float32x4,
    ];
    
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PlotInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Uniform buffer data
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PlotUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub bounds: [f32; 4], // x_min, y_min, x_max, y_max
}

/// GPU pipelines for plot rendering
pub struct PlotPipelines {
    /// Direct rendering pipeline for small datasets
    pub direct: wgpu::RenderPipeline,
    
    /// Instanced rendering pipeline for medium datasets
    pub instanced: wgpu::RenderPipeline,
    
    /// Compute pipeline for aggregation
    pub aggregation: wgpu::ComputePipeline,
    
    /// Bind group layout for uniforms
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl PlotPipelines {
    /// Create all plot rendering pipelines
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        // Create bind group layout for uniforms
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Plot Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Load shaders
        let plot_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Plot Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/plot.wgsl").into()),
        });
        
        let aggregation_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Aggregation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/aggregation.wgsl").into()),
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Plot Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create direct rendering pipeline
        let direct = create_direct_pipeline(device, &pipeline_layout, &plot_shader)?;
        
        // Create instanced rendering pipeline
        let instanced = create_instanced_pipeline(device, &pipeline_layout, &plot_shader)?;
        
        // Create aggregation compute pipeline
        let aggregation = create_aggregation_pipeline(device, &aggregation_shader)?;
        
        Ok(Self {
            direct,
            instanced,
            aggregation,
            uniform_bind_group_layout,
        })
    }
    
    /// Create uniform bind group
    pub fn create_uniform_bind_group(
        &self,
        device: &wgpu::Device,
        uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Plot Uniform Bind Group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

/// Create direct rendering pipeline for small datasets
fn create_direct_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> Result<wgpu::RenderPipeline> {
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Direct Plot Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[super::PlotVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::PointList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    
    Ok(pipeline)
}

/// Create instanced rendering pipeline for medium datasets
fn create_instanced_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> Result<wgpu::RenderPipeline> {
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Instanced Plot Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_instanced",
            buffers: &[
                super::PlotVertex::desc(),
                super::PlotInstance::desc(),
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    
    Ok(pipeline)
}

/// Create aggregation compute pipeline for large datasets
fn create_aggregation_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
) -> Result<wgpu::ComputePipeline> {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Aggregation Bind Group Layout"),
        entries: &[
            // Input points buffer
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Output bins buffer
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Parameters uniform
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Aggregation Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Aggregation Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: shader,
        entry_point: "aggregate_points",
    });
    
    Ok(pipeline)
} 