// Plot rendering shader for direct and instanced rendering

struct Uniforms {
    view_proj: mat4x4<f32>,
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct InstanceInput {
    @location(2) instance_pos: vec2<f32>,
    @location(3) instance_scale: f32,
    @location(4) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) point_coord: vec2<f32>,
}

// Direct rendering vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to clip space
    let world_pos = vec4<f32>(in.position, 0.0, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    out.color = in.color;
    out.point_coord = vec2<f32>(0.0, 0.0);
    
    return out;
}

// Instanced rendering vertex shader
@vertex
fn vs_instanced(
    in: VertexInput,
    instance: InstanceInput,
    @builtin(vertex_index) vertex_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    
    // Create a quad around the instance position
    let quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    
    let vertex_pos = quad_vertices[vertex_idx % 6u];
    let scaled_pos = vertex_pos * instance.instance_scale;
    let world_pos = vec4<f32>(instance.instance_pos + scaled_pos, 0.0, 1.0);
    
    out.clip_position = uniforms.view_proj * world_pos;
    out.color = instance.instance_color;
    out.point_coord = vertex_pos;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // For instanced rendering, create circular points
    if (length(in.point_coord) > 0.01) {
        let dist = length(in.point_coord);
        if (dist > 1.0) {
            discard;
        }
        
        // Smooth edge
        let alpha = 1.0 - smoothstep(0.8, 1.0, dist);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }
    
    return in.color;
} 