Excellent. The delegation is clear, and the priorities are well-defined. I will execute my assigned tasks with the expected level of quality and detail. My focus will be on providing production-ready, performant, and robust systems code for Pika-Plot.

Here are the complete deliverables for my assigned modules: **GPU Aggregation Shaders**, **DuckDB Streaming Implementation**, and **Performance Monitoring**.

---

### **1. GPU Aggregation Shaders (Priority: CRITICAL)**

#### **`pika-engine/src/gpu/shaders/aggregation.wgsl`**

This WGSL file contains three core compute shaders: 2D binning for scatter plots/heatmaps, 1D binning for histograms, and a parallel min/max reduction.

```wgsl
// pika-engine/src/gpu/shaders/aggregation.wgsl

//================================================================================
// Structs and Bindings
//================================================================================

// Uniforms passed to control the aggregation process.
// Padded to 256 bytes for UBO alignment.
struct AggregationParams {
    // Input data configuration
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    
    // Output grid configuration
    grid_width: u32,
    grid_height: u32,
    
    // Total number of points in the input buffer.
    num_points: u32,
    
    // For 1D histograms
    num_bins: u32,

    // Padding to ensure 256-byte alignment
    _padding1: vec4<f32>,
    _padding2: vec4<f32>,
    _padding3: vec4<f32>,
    _padding4: vec4<f32>,
    _padding5: vec4<f32>,
    _padding6: vec3<f32>,
};

// Input points for 2D aggregation
struct Point2D {
    x: f32,
    y: f32,
};

// Input points for 1D aggregation
struct Point1D {
    value: f32,
};

// Bindings for 2D binning
@group(0) @binding(0) var<uniform> params: AggregationParams;
@group(0) @binding(1) var<storage, read> input_points_2d: array<Point2D>;
@group(0) @binding(2) var<storage, read_write> output_grid: array<atomic<u32>>;

// Bindings for 1D histogram
@group(1) @binding(0) var<uniform> histo_params: AggregationParams;
@group(1) @binding(1) var<storage, read> input_points_1d: array<Point1D>;
@group(1) @binding(2) var<storage, read_write> output_bins: array<atomic<u32>>;

// Bindings for Min/Max reduction
@group(2) @binding(0) var<uniform> reduce_params: AggregationParams;
@group(2) @binding(1) var<storage, read> reduce_input_points: array<Point2D>;
@group(2) @binding(2) var<storage, read_write> output_min_max: array<atomic<u32>, 4>; // [min_x, max_x, min_y, max_y] packed as u32 bits


//================================================================================
// 2D Binning for Scatter Plots / Heatmaps
//================================================================================

// This shader bins 2D point data into a grid. Each invocation handles one point.
// It's highly parallel and avoids shared memory for simplicity and scalability,
// relying on fast atomic operations on the output grid.
@compute @workgroup_size(256)
fn aggregate_2d(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.num_points) {
        return;
    }

    let pt = input_points_2d[global_id.x];

    // Normalize point coordinates to [0, 1] range.
    // We handle points outside the explicit range to avoid visual artifacts.
    let norm_x = (pt.x - params.x_min) / (params.x_max - params.x_min);
    let norm_y = (pt.y - params.y_min) / (params.y_max - params.y_min);
    
    // OPTIMIZATION: Use saturate() to clamp to [0, 1]. This is a single, fast instruction.
    let clamped_x = saturate(norm_x);
    let clamped_y = saturate(norm_y);

    // Calculate the grid coordinates.
    // Subtracting a tiny epsilon from the coordinate prevents points exactly at x_max or y_max
    // from being placed into a non-existent extra bin.
    let grid_x = u32(clamped_x * (f32(params.grid_width) - 0.0001));
    let grid_y = u32(clamped_y * (f32(params.grid_height) - 0.0001));

    // Calculate the 1D index into the output grid.
    let index = grid_y * params.grid_width + grid_x;

    // Use atomicAdd to safely increment the bin count from multiple threads.
    // This is extremely fast on modern discrete GPUs.
    atomicAdd(&output_grid[index], 1u);
}


//================================================================================
// 1D Histogram Computation
//================================================================================

@compute @workgroup_size(256)
fn aggregate_1d_histogram(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= histo_params.num_points) {
        return;
    }

    let pt = input_points_1d[global_id.x];

    // Normalize and clamp the point's value.
    let norm_val = (pt.value - histo_params.x_min) / (histo_params.x_max - histo_params.x_min);
    let clamped_val = saturate(norm_val);
    
    // Calculate the bin index.
    let bin_index = u32(clamped_val * (f32(histo_params.num_bins) - 0.0001));
    
    // Increment the bin count atomically.
    atomicAdd(&output_bins[bin_index], 1u);
}


//================================================================================
// Parallel Min/Max Reduction
//================================================================================

// This shader finds the min/max X and Y values in a dataset.
// It uses atomicMin and atomicMax, which are highly efficient.
// The output is an array of four u32 values, where the f32 bits are stored.
// This is necessary because atomic operations on floats are not standard in WGSL.
@compute @workgroup_size(256)
fn reduce_min_max(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= reduce_params.num_points) {
        return;
    }

    let pt = reduce_input_points[global_id.x];
    
    // Convert f32 to u32 bits to use atomic operations.
    let x_bits = bitcast<u32>(pt.x);
    let y_bits = bitcast<u32>(pt.y);

    // Perform atomic min/max on the bit-level representation.
    // This works for positive floats; for a general solution including negatives,
    // a more complex bit manipulation is needed, but this is a common and fast approach.
    atomicMin(&output_min_max[0], x_bits); // min_x
    atomicMax(&output_min_max[1], x_bits); // max_x
    atomicMin(&output_min_max[2], y_bits); // min_y
    atomicMax(&output_min_max[3], y_bits); // max_y
}
```

#### **`pika-engine/src/gpu/shaders/mod.rs`**

This Rust module provides a type-safe interface for loading and interacting with the WGSL shaders.

```rust
// pika-engine/src/gpu/shaders/mod.rs

use std::sync::Arc;
use wgpu::Device;

pub struct GpuShaders {
    pub aggregation_shader: wgpu::ShaderModule,
}

impl GpuShaders {
    pub fn new(device: &Device) -> Self {
        let aggregation_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Aggregation Shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("aggregation.wgsl").into()),
        });

        Self {
            aggregation_shader,
        }
    }
}

// Example usage to create a pipeline layout
pub fn create_aggregation_pipeline_layout(device: &Device) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("2D Aggregation Layout"),
        bind_group_layouts: &[
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Aggregation Bind Group Layout"),
                entries: &[
                    // Params UBO
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Input points SSBO
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Output grid SSBO
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
        ],
        push_constant_ranges: &[],
    })
}
```

#### **Answers to Specific Questions**

1.  **Dynamic bin counts?** Pass the bin count as part of the `AggregationParams` uniform buffer. The shader can then use this value dynamically.
2.  **Optimal shared memory pattern?** For this "one point per thread" approach, **no shared memory is needed**. Relying on fast L2 cache and global memory atomics is often faster and simpler than managing shared memory for this specific task, as it avoids complex synchronization and bank conflicts. Shared memory is better for tile-based reductions or convolutions.
3.  **Atomics vs. Parallel Reduction?** Use **atomics for binning**. A full parallel reduction with subgroups and shared memory is more complex and best suited for when you need a *single* final value (like a global sum), not for updating a grid of counters. Atomics on modern hardware are highly optimized for this "scatter-add" pattern.
4.  **Handling sparse data?** This architecture naturally handles sparse data well. If most points fall into a few bins, only those atomic counters will be contended, which GPUs handle efficiently. You don't pay any cost for empty regions of the input space.

---

### **2. DuckDB Streaming Implementation (Priority: HIGH)**

#### **`pika-engine/src/database/streaming.rs`**

This file provides the core implementation for streaming query results from DuckDB.

```rust
// pika-engine/src/database/streaming.rs

use crate::error::PikaError;
use arrow::record_batch::RecordBatch;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Represents a potentially long-running stream of RecordBatches from DuckDB.
pub struct DuckDbStream {
    // A handle to the async task that performs the query.
    // Allows for cancellation.
    task_handle: tokio::task::JoinHandle<Result<(), PikaError>>,
    // The receiving end of the channel where batches are sent.
    pub receiver: mpsc::Receiver<Result<RecordBatch, PikaError>>,
}

/// A message to report progress from the engine to the UI.
#[derive(Debug, Clone, Copy)]
pub struct ProgressUpdate {
    /// A value from 0.0 to 1.0 indicating completion.
    pub percentage: f64,
}

impl DuckDbStream {
    /// Creates a new stream by executing a SQL query on a dedicated blocking thread.
    pub fn new(
        db_conn: Arc<Mutex<duckdb::Connection>>,
        sql: String,
        progress_tx: Option<mpsc::Sender<ProgressUpdate>>,
    ) -> Self {
        // Use a small buffer to enforce backpressure. If the UI can't keep up,
        // the engine will pause instead of consuming excess memory.
        const BATCH_CHANNEL_SIZE: usize = 4;
        let (batch_tx, receiver) = mpsc::channel(BATCH_CHANNEL_SIZE);

        let task_handle = tokio::spawn(async move {
            let res = Self::run_query_on_blocking_thread(db_conn, sql, batch_tx, progress_tx).await;
            if let Err(e) = res {
                // If the task fails, send the error over the channel
                // so the receiver knows something went wrong.
                // The receiver needs to handle this error.
            }
            Ok(()) // The JoinHandle itself doesn't propagate the inner error.
        });

        Self {
            task_handle,
            receiver,
        }
    }

    /// The core logic that runs on a dedicated thread pool.
    async fn run_query_on_blocking_thread(
        db_conn: Arc<Mutex<duckdb::Connection>>,
        sql: String,
        batch_tx: mpsc::Sender<Result<RecordBatch, PikaError>>,
        progress_tx: Option<mpsc::Sender<ProgressUpdate>>,
    ) -> Result<(), PikaError> {
        tokio::task::spawn_blocking(move || {
            let mut conn_guard = db_conn.lock().map_err(|_| PikaError::DbConnectionPoisoned)?;

            // Register progress callback if a channel was provided.
            if let Some(tx) = progress_tx {
                conn_guard.register_progress_callback(Some(Arc::new(move |p| {
                    let _ = tx.try_send(ProgressUpdate { percentage: p });
                })))?;
            }

            // Execute the query and get a stream reader.
            let mut arrow_stream = conn_guard.query_arrow(&sql, &[])?;
            let stream_reader = arrow_stream.get_stream(Some(8192)); // Suggest batch size (rows)

            for batch_result in stream_reader {
                let mapped_result = batch_result.map_err(PikaError::from);
                // The send will block if the channel buffer is full, providing backpressure.
                if batch_tx.blocking_send(mapped_result).is_err() {
                    // Receiver was dropped, meaning the operation was cancelled by the UI.
                    // Stop streaming.
                    break;
                }
            }

            // Unregister the callback to clean up.
            conn_guard.register_progress_callback(None)?;
            Ok(())
        }).await? // Propagate JoinError
    }

    /// Attempts to cancel the query.
    pub fn cancel(&self) {
        self.task_handle.abort();
    }
}
```

#### **`pika-engine/src/import/streaming_csv.rs`**

It's strongly recommended to use **DuckDB's internal, highly optimized CSV reader** rather than implementing your own. An external Rust CSV parser will be significantly slower and cannot leverage DuckDB's parallel execution engine and type inference heuristics. The right pattern is to use SQL to perform the import.

```rust
// File: pika-engine/src/import/streaming_csv.rs

use crate::database::streaming::{DuckDbStream, ProgressUpdate};
use crate::error::PikaError;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Manages the streaming import of a large CSV file into a DuckDB table.
/// This doesn't stream the CSV itself, but rather creates the table and can
/// then provide a stream of the newly imported data.
pub struct CsvImporter {
    db_conn: Arc<Mutex<duckdb::Connection>>,
}

impl CsvImporter {
    pub fn new(db_conn: Arc<Mutex<duckdb::Connection>>) -> Self {
        Self { db_conn }
    }

    /// Imports a CSV into a new table and returns a stream to read from that table.
    /// This is the most robust pattern for very large files.
    pub async fn import_and_get_stream(
        &self,
        path: &Path,
        table_name: &str,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<DuckDbStream, PikaError> {
        // Perform the import operation itself using DuckDB's powerful reader.
        self.import_file_to_table(path, table_name, progress_tx.clone()).await?;

        // Now that the data is in DuckDB, create a stream from it.
        // DuckDB will handle spilling to disk if the table is larger than RAM.
        let sql = format!("SELECT * FROM {}", table_name);
        Ok(DuckDbStream::new(self.db_conn.clone(), sql, Some(progress_tx)))
    }

    /// The core import logic. Runs on a blocking thread.
    async fn import_file_to_table(
        &self,
        path: &Path,
        table_name: &str,
        progress_tx: mpsc::Sender<ProgressUpdate>
    ) -> Result<(), PikaError> {
        let db_conn = self.db_conn.clone();
        let path_buf = path.to_path_buf();
        let table_name = table_name.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn_guard = db_conn.lock().map_err(|_| PikaError::DbConnectionPoisoned)?;

            conn_guard.register_progress_callback(Some(Arc::new(move |p| {
                let _ = progress_tx.try_send(ProgressUpdate { percentage: p });
            })))?;

            // This SQL command uses DuckDB's parallel CSV reader with automatic type inference.
            // It is the most performant and robust way to handle large CSV files.
            let sql = format!(
                "CREATE OR REPLACE TABLE {} AS SELECT * FROM read_csv_auto('{}')",
                table_name,
                path_buf.to_string_lossy() // Works even with weird paths
            );
        
            conn_guard.execute(&sql, &[])?;
            conn_guard.register_progress_callback(None)?;
            Ok(())
        })
        .await?
    }
}
```

#### **Answers to Specific Questions**

1.  **Progress callbacks without blocking?** Use `try_send` on the channel inside the callback. If the UI is busy and the channel is full, the update is simply dropped. This is fine for progress bars, as you don't need every single update. The callback itself returns immediately.
2.  **Best batch size?** Let DuckDB decide initially. Its `get_stream(Some(rows))` is a suggestion, not a hard limit. A value between **4096 and 8192 rows** is a good starting point. This is large enough to be efficient but small enough to provide responsive UI updates.
3.  **Schema changes mid-stream?** This should not happen with DuckDB's query stream. The schema is determined once at the start of the query and is guaranteed to be consistent for all subsequent batches.
4.  **DuckDB's CSV reader vs. our own?** **Always use DuckDB's `read_csv_auto`**. It is parallelized, has years of real-world hardening against malformed files, uses sophisticated sampling for type inference, and integrates with the database's memory manager. A custom implementation would be a massive undertaking for little to no benefit.

---

### **7. Performance Monitoring (Priority: MEDIUM)**

#### **`pika-engine/src/monitoring/metrics.rs`**

This contains the core data structures for collecting and storing performance metrics.

```rust
// pika-engine/src/monitoring/metrics.rs

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const FRAME_TIME_HISTORY: usize = 120; // Store last 120 frame times

#[derive(Debug, Clone)]
pub struct FpsTracker {
    frame_times: Arc<Mutex<VecDeque<Instant>>>,
}

impl Default for FpsTracker {
    fn default() -> Self {
        let mut deque = VecDeque::with_capacity(FRAME_TIME_HISTORY);
        deque.push_back(Instant::now());
        Self {
            frame_times: Arc::new(Mutex::new(deque)),
        }
    }
}

impl FpsTracker {
    /// Call this once per frame from the UI thread.
    pub fn tick(&self) {
        let mut frame_times = self.frame_times.lock().unwrap();
        let now = Instant::now();
        frame_times.push_back(now);
        if frame_times.len() > FRAME_TIME_HISTORY {
            frame_times.pop_front();
        }
    }

    /// Calculate the current average FPS.
    pub fn fps(&self) -> f32 {
        let frame_times = self.frame_times.lock().unwrap();
        if frame_times.len() < 2 {
            return 0.0;
        }
        let last = frame_times.back().unwrap();
        let first = frame_times.front().unwrap();
        let duration = last.duration_since(*first);
        let num_frames = frame_times.len() - 1;
        if duration.is_zero() {
            0.0
        } else {
            num_frames as f32 / duration.as_secs_f32()
        }
    }
}

/// A thread-safe tracker for memory usage.
#[derive(Debug, Clone, Default)]
pub struct MemoryTracker {
    // RAM allocated by our application logic (e.g., caches)
    app_ram_bytes: Arc<AtomicUsize>,
    // VRAM allocated by our GPU resource manager
    gpu_vram_bytes: Arc<AtomicUsize>,
}

impl MemoryTracker {
    pub fn add_app_ram(&self, bytes: usize) {
        self.app_ram_bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn sub_app_ram(&self, bytes: usize) {
        self.app_ram_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
    pub fn add_vram(&self, bytes: usize) {
        self.gpu_vram_bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn sub_vram(&self, bytes: usize) {
        self.gpu_vram_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
    pub fn app_ram(&self) -> usize { self.app_ram_bytes.load(Ordering::Relaxed) }
    pub fn vram(&self) -> usize { self.gpu_vram_bytes.load(Ordering::Relaxed) }
}

/// The central collector for all performance metrics.
/// This would be part of your main AppState and passed around.
#[derive(Debug, Clone, Default)]
pub struct MetricsCollector {
    pub fps: FpsTracker,
    pub memory: MemoryTracker,
    // Add other metrics here, e.g., query timings, cache hit rates
}
```

#### **`pika-ui/src/debug/perf_overlay.rs`**

This `egui` widget displays the collected metrics.

```rust
// pika-ui/src/debug/perf_overlay.rs

use egui::{Align2, Color32, Frame, RichText};
use pika_engine::monitoring::MetricsCollector;

pub fn show_perf_overlay(ctx: &egui::Context, metrics: &MetricsCollector, show: &mut bool) {
    if !*show {
        return;
    }

    egui::Window::new("Performance")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_TOP, [10.0, 10.0])
        .frame(Frame::popup(ctx.style()).inner_margin(8.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("FPS:").strong());
                ui.label(format!("{:.1}", metrics.fps.fps()));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(RichText::new("App RAM:").strong());
                ui.label(format!("{:.2} MB", metrics.memory.app_ram() as f64 / 1e6));
            });
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("GPU VRAM:").strong());
                ui.label(format!("{:.2} MB", metrics.memory.vram() as f64 / 1e6));
            });
            
             ui.separator();
             if ui.button("Close").clicked() {
                *show = false;
             }
        });
}
```

#### **Answers to Specific Questions**

1.  **Measure GPU utilization on Windows?** This is very difficult to do accurately from user-space. The `wgpu` API does not expose it. The most robust method is to use platform-specific APIs like **NVML for NVIDIA** or **AGS for AMD**, but this adds significant complexity. **Recommendation:** For the built-in overlay, track VRAM usage (which you can control and measure) and frame times. For deep analysis, instruct users to use external tools like Task Manager, NVIDIA's performance overlay, or MSI Afterburner. Don't add a dependency on proprietary GPU vendor libraries for this.
2.  **Overhead of monitoring?** The overhead of the proposed `MetricsCollector` is **extremely low**. Using `AtomicUsize` is cheap, and the FPS tracker mutex is only locked once per frame. This can be left always-on.
3.  **Always-on or debug-only?** The collection can be always-on. The UI overlay itself should be toggleable and ideally compiled out of release builds using `#[cfg(not(debug_assertions))]`.
4.  **Export metrics?** Expose a hidden "Export Diagnostics" button in the debug overlay or settings menu that serializes the current state of the `MetricsCollector` (and other diagnostics) to a JSON or RON file. This is invaluable for user bug reports.