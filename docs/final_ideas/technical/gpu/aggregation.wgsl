// GPU Aggregation Compute Shader for Pika-Plot
// This shader performs 2D histogram binning for massive datasets

struct AggregationParams {
    viewport_min: vec2<f32>,      // Minimum bounds in data space
    viewport_max: vec2<f32>,      // Maximum bounds in data space
    bin_size_x: f32,              // Bin width in data units
    bin_size_y: f32,              // Bin height in data units
    max_bins: u32,                // Maximum bins per dimension (typically 256-512)
    point_count: u32,             // Total number of input points
}

// Input: raw points in data space
@group(0) @binding(0) var<storage, read> input_points: array<vec2<f32>>;

// Output: 2D histogram grid (flattened)
@group(0) @binding(1) var<storage, read_write> output_grid: array<atomic<u32>>;

// Parameters
@group(0) @binding(2) var<uniform> params: AggregationParams;

// Helper function to compute bin index from data point
fn point_to_bin(point: vec2<f32>) -> vec2<u32> {
    let normalized = (point - params.viewport_min) / (params.viewport_max - params.viewport_min);
    let bin_x = u32(clamp(normalized.x * f32(params.max_bins), 0.0, f32(params.max_bins - 1u)));
    let bin_y = u32(clamp(normalized.y * f32(params.max_bins), 0.0, f32(params.max_bins - 1u)));
    return vec2<u32>(bin_x, bin_y);
}

// Convert 2D bin coordinates to 1D array index
fn bin_to_index(bin: vec2<u32>) -> u32 {
    return bin.y * params.max_bins + bin.x;
}

@compute @workgroup_size(256)
fn aggregate_points(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let point_idx = global_id.x;
    
    // Check bounds
    if (point_idx >= params.point_count) {
        return;
    }
    
    // Read input point
    let point = input_points[point_idx];
    
    // Skip points outside viewport
    if (point.x < params.viewport_min.x || point.x > params.viewport_max.x ||
        point.y < params.viewport_min.y || point.y > params.viewport_max.y) {
        return;
    }
    
    // Compute bin and increment atomically
    let bin = point_to_bin(point);
    let idx = bin_to_index(bin);
    atomicAdd(&output_grid[idx], 1u);
}

// ===== Additional shaders for density estimation =====

// Gaussian kernel for smooth density estimation
fn gaussian_kernel(distance_sq: f32, bandwidth: f32) -> f32 {
    let h_sq = bandwidth * bandwidth;
    return exp(-0.5 * distance_sq / h_sq) / (2.0 * 3.14159265359 * h_sq);
}

@compute @workgroup_size(16, 16)
fn density_estimation(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bin_coord = vec2<u32>(global_id.xy);
    
    if (bin_coord.x >= params.max_bins || bin_coord.y >= params.max_bins) {
        return;
    }
    
    let idx = bin_to_index(bin_coord);
    let bin_center = vec2<f32>(
        params.viewport_min.x + (f32(bin_coord.x) + 0.5) * params.bin_size_x,
        params.viewport_min.y + (f32(bin_coord.y) + 0.5) * params.bin_size_y
    );
    
    var density = 0.0;
    let bandwidth = max(params.bin_size_x, params.bin_size_y) * 2.0;
    
    // Sample nearby points for density calculation
    // This is a simplified version - real implementation would use spatial indexing
    let sample_radius = 3u; // Check 3x3 grid around current bin
    
    for (var dy = -sample_radius; dy <= sample_radius; dy++) {
        for (var dx = -sample_radius; dx <= sample_radius; dx++) {
            let neighbor_x = i32(bin_coord.x) + dx;
            let neighbor_y = i32(bin_coord.y) + dy;
            
            if (neighbor_x >= 0 && neighbor_x < i32(params.max_bins) &&
                neighbor_y >= 0 && neighbor_y < i32(params.max_bins)) {
                let neighbor_idx = u32(neighbor_y) * params.max_bins + u32(neighbor_x);
                let count = f32(atomicLoad(&output_grid[neighbor_idx]));
                
                let neighbor_center = vec2<f32>(
                    params.viewport_min.x + (f32(neighbor_x) + 0.5) * params.bin_size_x,
                    params.viewport_min.y + (f32(neighbor_y) + 0.5) * params.bin_size_y
                );
                
                let dist_sq = dot(bin_center - neighbor_center, bin_center - neighbor_center);
                density += count * gaussian_kernel(dist_sq, bandwidth);
            }
        }
    }
    
    // Store density value (would need separate output buffer in real implementation)
    // atomicStore(&density_grid[idx], u32(density * 1000.0));
}

// ===== Shader for adaptive binning based on zoom level =====

struct AdaptiveParams {
    zoom_level: f32,              // Current zoom level
    focus_point: vec2<f32>,       // Center of user focus
    base_bin_count: u32,          // Base number of bins at zoom level 1.0
    min_bin_size: f32,            // Minimum bin size in screen pixels
    max_bin_size: f32,            // Maximum bin size in screen pixels
}

@group(0) @binding(3) var<uniform> adaptive: AdaptiveParams;

fn compute_adaptive_bin_size() -> vec2<f32> {
    // Adjust bin count based on zoom level
    let effective_bins = f32(adaptive.base_bin_count) * sqrt(adaptive.zoom_level);
    let viewport_size = params.viewport_max - params.viewport_min;
    let bin_size = viewport_size / effective_bins;
    
    // Clamp to reasonable sizes
    return clamp(bin_size, vec2<f32>(adaptive.min_bin_size), vec2<f32>(adaptive.max_bin_size));
}

// ===== Shader for LOD (Level of Detail) selection =====

struct LODParams {
    screen_size: vec2<f32>,       // Viewport size in pixels
    point_density_threshold: f32,  // Points per pixel threshold
    current_point_count: u32,      // Number of visible points
}

@group(0) @binding(4) var<uniform> lod: LODParams;

fn select_render_mode() -> u32 {
    let pixels = lod.screen_size.x * lod.screen_size.y;
    let density = f32(lod.current_point_count) / pixels;
    
    if (density < 0.1) {
        return 0u; // Direct rendering
    } else if (density < 10.0) {
        return 1u; // Instanced rendering
    } else {
        return 2u; // Aggregated rendering
    }
} 