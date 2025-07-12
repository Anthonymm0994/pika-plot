// Aggregation compute shader for large datasets

struct AggregationParams {
    bin_count_x: u32,
    bin_count_y: u32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    point_count: u32,
}

@group(0) @binding(0)
var<storage, read> points: array<vec2<f32>>;

@group(0) @binding(1)
var<storage, read_write> bins: array<atomic<u32>>;

@group(0) @binding(2)
var<uniform> params: AggregationParams;

@compute @workgroup_size(256)
fn aggregate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.point_count) {
        return;
    }
    
    let point = points[idx];
    
    // Calculate bin indices
    let x_range = params.x_max - params.x_min;
    let y_range = params.y_max - params.y_min;
    
    let x_normalized = (point.x - params.x_min) / x_range;
    let y_normalized = (point.y - params.y_min) / y_range;
    
    let x_bin = u32(clamp(x_normalized * f32(params.bin_count_x), 0.0, f32(params.bin_count_x - 1u)));
    let y_bin = u32(clamp(y_normalized * f32(params.bin_count_y), 0.0, f32(params.bin_count_y - 1u)));
    
    let bin_idx = y_bin * params.bin_count_x + x_bin;
    
    // Atomically increment bin count
    atomicAdd(&bins[bin_idx], 1u);
} 