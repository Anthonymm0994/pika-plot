# GPU Vertex Buffer Layouts for Pika-Plot

This document specifies the exact vertex buffer layouts for all rendering modes in Pika-Plot.

## Overview

Pika-Plot uses three rendering strategies based on data size:
1. **Direct Rendering**: < 50k points - Each point is a vertex
2. **Instanced Rendering**: 50k-5M points - Points rendered as instanced quads
3. **Aggregated Rendering**: > 5M points - Compute shader generates density texture

## 1. Direct Rendering Vertex Layout

For small datasets, each data point becomes a vertex with all attributes.

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectVertex {
    // Position in normalized device coordinates (after transformation)
    pub position: [f32; 2],     // 8 bytes
    
    // Original data values (for tooltips/interaction)
    pub data_x: f32,            // 4 bytes
    pub data_y: f32,            // 4 bytes
    
    // Visual attributes
    pub color: [f32; 4],        // 16 bytes (RGBA)
    pub size: f32,              // 4 bytes
    pub shape: u32,             // 4 bytes (0=circle, 1=square, 2=triangle, etc.)
    
    // Selection/highlight state
    pub selected: u32,          // 4 bytes (0 or 1)
    
    // Padding for alignment
    pub _padding: [f32; 2],     // 8 bytes
}
// Total: 48 bytes per vertex (aligned to 16 bytes)

const DIRECT_VERTEX_LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: 48,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        // position
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        },
        // data values
        wgpu::VertexAttribute {
            offset: 8,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
        // color
        wgpu::VertexAttribute {
            offset: 16,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        // size
        wgpu::VertexAttribute {
            offset: 32,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32,
        },
        // shape
        wgpu::VertexAttribute {
            offset: 36,
            shader_location: 4,
            format: wgpu::VertexFormat::Uint32,
        },
        // selected
        wgpu::VertexAttribute {
            offset: 40,
            shader_location: 5,
            format: wgpu::VertexFormat::Uint32,
        },
    ],
};
```

## 2. Instanced Rendering Vertex Layout

For medium datasets, we use a quad mesh with per-instance data.

### Base Quad Vertex
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],     // Local quad position (-1 to 1)
    pub uv: [f32; 2],          // Texture coordinates
}

const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [-1.0, -1.0], uv: [0.0, 1.0] }, // Bottom-left
    QuadVertex { position: [ 1.0, -1.0], uv: [1.0, 1.0] }, // Bottom-right
    QuadVertex { position: [ 1.0,  1.0], uv: [1.0, 0.0] }, // Top-right
    QuadVertex { position: [-1.0,  1.0], uv: [0.0, 0.0] }, // Top-left
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
```

### Instance Data
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    // World position of the point
    pub center: [f32; 2],       // 8 bytes
    
    // Original data values
    pub data_x: f32,            // 4 bytes
    pub data_y: f32,            // 4 bytes
    
    // Visual attributes
    pub color: [f32; 4],        // 16 bytes
    pub size: f32,              // 4 bytes
    pub shape: u32,             // 4 bytes
    
    // Selection state
    pub selected: u32,          // 4 bytes
}
// Total: 44 bytes per instance

const INSTANCE_LAYOUT: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: 44,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &[
        // center
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x2,
        },
        // data values
        wgpu::VertexAttribute {
            offset: 8,
            shader_location: 6,
            format: wgpu::VertexFormat::Float32x2,
        },
        // color
        wgpu::VertexAttribute {
            offset: 16,
            shader_location: 7,
            format: wgpu::VertexFormat::Float32x4,
        },
        // size
        wgpu::VertexAttribute {
            offset: 32,
            shader_location: 8,
            format: wgpu::VertexFormat::Float32,
        },
        // shape
        wgpu::VertexAttribute {
            offset: 36,
            shader_location: 9,
            format: wgpu::VertexFormat::Uint32,
        },
        // selected
        wgpu::VertexAttribute {
            offset: 40,
            shader_location: 10,
            format: wgpu::VertexFormat::Uint32,
        },
    ],
};
```

## 3. Aggregated Rendering Output

For large datasets, the compute shader outputs a density texture instead of vertices.

```rust
// Compute shader output is a 2D texture
pub struct AggregationOutput {
    pub density_texture: wgpu::Texture,    // R32Uint or R32Float format
    pub texture_size: (u32, u32),          // e.g., (512, 512)
    pub data_bounds: PlotBounds,           // Maps texture to data space
}

// The density texture is then rendered as a single quad with the texture applied
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedQuadVertex {
    pub position: [f32; 2],    // Screen position
    pub uv: [f32; 2],         // Texture coordinates
}
```

## 4. Uniform Buffer Layouts

All rendering modes share common uniform buffers for transformation.

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniforms {
    // Transform from data space to clip space
    pub view_projection: [[f32; 4]; 4],    // 64 bytes
    
    // Viewport information
    pub viewport_size: [f32; 2],           // 8 bytes
    pub zoom_level: f32,                   // 4 bytes
    pub time: f32,                         // 4 bytes (for animations)
    
    // Interaction state
    pub mouse_pos: [f32; 2],               // 8 bytes
    pub hover_id: u32,                     // 4 bytes
    pub _padding: u32,                     // 4 bytes
}
// Total: 96 bytes (aligned to 16 bytes)

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlotUniforms {
    // Plot-specific parameters
    pub point_scale: f32,                  // 4 bytes
    pub opacity: f32,                      // 4 bytes
    pub outline_width: f32,                // 4 bytes
    pub selection_glow: f32,               // 4 bytes
    
    // Color mapping
    pub color_range: [f32; 2],             // 8 bytes
    pub color_map_id: u32,                 // 4 bytes
    pub _padding: u32,                     // 4 bytes
}
// Total: 32 bytes
```

## 5. Pipeline Creation

Example code for creating the appropriate pipeline:

```rust
impl GpuPlotRenderer {
    pub fn create_direct_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Direct Plot Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/direct.wgsl")),
        });
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Direct Plot Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DIRECT_VERTEX_LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
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
                count: 4, // 4x MSAA
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
```

## 6. Memory Alignment Rules

- All vertex attributes must be naturally aligned (f32 = 4 bytes, vec2 = 8 bytes, etc.)
- Vertex buffer stride must be a multiple of 4 bytes
- Uniform buffers must be 16-byte aligned
- Storage buffers for compute shaders should be 16-byte aligned for optimal performance

## 7. Performance Considerations

1. **Vertex Size**: Keep vertices under 64 bytes for optimal GPU cache utilization
2. **Attribute Packing**: Pack related attributes together (position + data values)
3. **Padding**: Add padding to ensure proper alignment rather than wasting attributes
4. **Instance Culling**: Implement frustum culling on CPU before uploading instance data
5. **Dynamic Buffers**: Use `wgpu::BufferUsages::COPY_DST` for frequently updated data

## 8. Platform-Specific Notes

### Windows (DirectX 12)
- Vertex buffers must be 256-byte aligned for optimal performance
- Use `wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES` for broader format support

### Future Considerations
- Add support for 3D plots (add z-coordinate to vertices)
- Consider half-precision floats for color/size on bandwidth-limited GPUs
- Implement vertex pulling for ultimate flexibility 