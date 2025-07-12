// Compute shader for aggregating large point datasets into bins

struct Point {
    x: f32,
    y: f32,
}

struct Bin {
    count: atomic<u32>,
    sum_x: atomic<u32>,
    sum_y: atomic<u32>,
}

struct AggregationParams {
    viewport_min: vec2<f32>,
    viewport_max: vec2<f32>,
    bin_count_x: u32,
    bin_count_y: u32,
    point_count: u32,
}

@group(0) @binding(0)
var<storage, read> input_points: array<Point>;

@group(0) @binding(1)
var<storage, read_write> output_bins: array<Bin>;

@group(0) @binding(2)
var<uniform> params: AggregationParams;

@compute @workgroup_size(256)
fn aggregate_points(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let point_idx = global_id.x;
    
    if (point_idx >= params.point_count) {
        return;
    }
    
    let point = input_points[point_idx];
    
    // Check if point is within viewport
    if (point.x < params.viewport_min.x || point.x > params.viewport_max.x ||
        point.y < params.viewport_min.y || point.y > params.viewport_max.y) {
        return;
    }
    
    // Calculate bin indices
    let viewport_size = params.viewport_max - params.viewport_min;
    let normalized_pos = (vec2<f32>(point.x, point.y) - params.viewport_min) / viewport_size;
    
    let bin_x = u32(normalized_pos.x * f32(params.bin_count_x));
    let bin_y = u32(normalized_pos.y * f32(params.bin_count_y));
    
    // Clamp to valid range
    let clamped_bin_x = min(bin_x, params.bin_count_x - 1u);
    let clamped_bin_y = min(bin_y, params.bin_count_y - 1u);
    
    // Calculate linear bin index
    let bin_idx = clamped_bin_y * params.bin_count_x + clamped_bin_x;
    
    // Atomically update bin
    atomicAdd(&output_bins[bin_idx].count, 1u);
    
    // For averaging positions (convert to fixed point for atomic operations)
    let fixed_x = u32(point.x * 1000.0);
    let fixed_y = u32(point.y * 1000.0);
    atomicAdd(&output_bins[bin_idx].sum_x, fixed_x);
    atomicAdd(&output_bins[bin_idx].sum_y, fixed_y);
}

// Second pass to convert aggregated data to renderable points
@compute @workgroup_size(256)
fn bins_to_points(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let bin_idx = global_id.x;
    let total_bins = params.bin_count_x * params.bin_count_y;
    
    if (bin_idx >= total_bins) {
        return;
    }
    
    let bin = output_bins[bin_idx];
    let count = atomicLoad(&bin.count);
    
    if (count == 0u) {
        return;
    }
    
    // Calculate average position
    let sum_x = f32(atomicLoad(&bin.sum_x)) / 1000.0;
    let sum_y = f32(atomicLoad(&bin.sum_y)) / 1000.0;
    let avg_x = sum_x / f32(count);
    let avg_y = sum_y / f32(count);
    
    // Calculate bin position
    let bin_y = bin_idx / params.bin_count_x;
    let bin_x = bin_idx % params.bin_count_x;
    
    // Store result (would write to output buffer in real implementation)
} 