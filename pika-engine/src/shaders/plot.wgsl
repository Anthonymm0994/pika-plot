// Plot rendering shader

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Direct rendering vertex shader
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = vec4<f32>(input.position, 0.0, 1.0);
    out.clip_position = uniforms.view_proj * pos;
    out.color = vec4<f32>(0.2, 0.6, 0.9, 1.0); // Blue color
    return out;
}

// Instanced rendering vertex shader
struct InstanceInput {
    @location(1) instance_pos: vec2<f32>,
    @location(2) instance_size: f32,
}

@vertex
fn vs_main_instanced(
    @builtin(vertex_index) vertex_idx: u32,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate quad vertices
    let x = f32((vertex_idx & 1u) * 2u) - 1.0;
    let y = f32((vertex_idx >> 1u) * 2u) - 1.0;
    
    let pos = vec4<f32>(
        instance.instance_pos.x + x * instance.instance_size,
        instance.instance_pos.y + y * instance.instance_size,
        0.0,
        1.0
    );
    
    out.clip_position = uniforms.view_proj * pos;
    out.color = vec4<f32>(0.2, 0.6, 0.9, 0.8); // Semi-transparent blue
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
} 