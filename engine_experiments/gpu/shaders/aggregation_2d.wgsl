// 2D binning aggregation shader for scatter plots
// Based on Gemini 2.5 Pro's recommendations:
// - 256 thread workgroup size
// - Shared memory for local aggregation
// - Atomic operations for global updates

struct Config {
    bin_count_x: u32,
    bin_count_y: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    point_count: u32,
    _padding: u32,
}

@group(0) @binding(0)
var<uniform> config: Config;

@group(0) @binding(1)
var<storage, read> points: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read_write> bins: array<atomic<u32>>;

// Shared memory for workgroup-local aggregation
var<workgroup> local_bins: array<atomic<u32>, 64>;

@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let point_idx = global_id.x;
    
    // Initialize shared memory (first 64 threads)
    if (local_id.x < 64u) {
        atomicStore(&local_bins[local_id.x], 0u);
    }
    workgroupBarrier();
    
    // Process point if within bounds
    if (point_idx < config.point_count) {
        let point = points[point_idx];
        
        // Calculate bin indices
        let norm_x = (point.x - config.min_x) / (config.max_x - config.min_x);
        let norm_y = (point.y - config.min_y) / (config.max_y - config.min_y);
        
        // Clamp to valid range
        let bin_x = u32(clamp(norm_x * f32(config.bin_count_x), 0.0, f32(config.bin_count_x - 1u)));
        let bin_y = u32(clamp(norm_y * f32(config.bin_count_y), 0.0, f32(config.bin_count_y - 1u)));
        
        let bin_idx = bin_y * config.bin_count_x + bin_x;
        
        // Aggregate locally first (reduce global atomic contention)
        let local_bin_idx = bin_idx % 64u;
        atomicAdd(&local_bins[local_bin_idx], 1u);
    }
    
    workgroupBarrier();
    
    // Write local results to global memory (first 64 threads)
    if (local_id.x < 64u) {
        let count = atomicLoad(&local_bins[local_id.x]);
        if (count > 0u) {
            // Each workgroup writes to different bins to reduce contention
            let global_bin_idx = (workgroup_id.x * 64u + local_id.x) % (config.bin_count_x * config.bin_count_y);
            atomicAdd(&bins[global_bin_idx], count);
        }
    }
} 