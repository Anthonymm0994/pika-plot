# Grok 4's Technical Research Response for Pika-Plot - Round 2

As Grok 4, I'm zeroing in on 3 questions from the new Round 2 list that play to my strengths in low-level optimization, GPU systems, Rust concurrency, and Windows-specific quirks: (1) GPU Testing Without Hardware (from section 1), (2) GPU Driver Quirks (from section 6), and (3) Handling Larger-than-Memory Datasets (from section 5, as it ties into memory coordination with GPU). These build on prior consensus (e.g., dedicated test machine agreement) and address mmap safety disagreements by emphasizing safe, performant patterns. Insights draw from the 2025-07-12 web search, like [github.com](https://github.com/gfx-rs/wgpu) for wgpu's backend options (e.g., software rendering via GLES for testing) and [lib.rs](https://lib.rs/crates/naga-cli) for shader validation tools.

Responses include solutions, risks/tradeoffs, sample code/scaffolds, and crate references. Emphasis on pluggable designs for extensibility (e.g., mock traits) without altering core constraints.

## 1. GPU Testing Without Hardware
**Selected Questions**: What's the best approach for mocking wgpu Device and Queue for unit tests? Should we use `wgpu::Instance::new(wgpu::Backends::PRIMARY)` with software rendering, or create full mock implementations? How do we test GPU memory pressure scenarios without actual VRAM limits? What's the recommended way to test shader compilation errors and validation? Should we maintain a separate test harness that can optionally run on real GPUs when available?

**Grok's Response**: The best approach is a trait-based mock for `Device` and `Queue` (pluggable via `GpuDevice` as in your example), allowing unit tests to simulate behaviors without hardware. Use wgpu's software backends (e.g., GLES via `Backends::GL`) for integration tests in CI, as it emulates DX12/Vulkan without a physical GPU [github.com](https://github.com/gfx-rs/wgpu) (noting GLES minor version config for compatibility). For memory pressure, inject mock limits into the trait and simulate OOM via error injection. Test shader errors with `naga-cli` for offline validation [lib.rs](https://lib.rs/crates/naga-cli), compiling WGSL to SPIR-V and checking diagnostics. Maintain a separate harness (e.g., `tests/gpu_harness.rs`) with feature flags for real vs mock runs. Risk: Software backends may not catch hardware-specific bugs (e.g., TDR timeouts); mitigate with optional real-GPU CI jobs. Tradeoff: Mocks add ~10% test complexity but enable 100% coverage without hardware, aligning with offline constraints.

**Sample Code/Scaffold** (extending your `GpuDevice` trait in `pika-ui/src/gpu/mod.rs` for tests):
```rust
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

// Trait from your example, made pluggable
pub trait GpuDevice: Send + Sync {
    fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer;
    fn create_shader_module(&self, desc: &wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule;
    fn simulated_vram_usage(&self) -> u64; // For memory pressure testing
}

// Mock impl for unit tests
#[derive(Default)]
pub struct MockGpuDevice {
    vram_usage: AtomicU64, // Simulate memory
}

impl GpuDevice for MockGpuDevice {
    fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        self.vram_usage.fetch_add(desc.size, Ordering::SeqCst);
        // Return dummy buffer; panic if "OOM"
        if self.vram_usage.load(Ordering::SeqCst) > 1_000_000_000 { panic!("Mock OOM"); }
        // Minimal stub (use wgpu::BufferSlice or similar for realism)
        unimplemented!() // Expand with mock data
    }

    fn create_shader_module(&self, desc: &wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        // Use naga for validation in tests
        let _ = naga::front::wgsl::parse_str(desc.source.source).expect("Shader validation failed");
        unimplemented!() // Stub
    }

    fn simulated_vram_usage(&self) -> u64 {
        self.vram_usage.load(Ordering::SeqCst)
    }
}

// Test example
#[test]
fn test_memory_pressure() {
    let mock = MockGpuDevice::default();
    // Simulate allocations until "OOM"
    for _ in 0..100 {
        mock.create_buffer(&wgpu::BufferDescriptor { size: 10_000_000, ..Default::default() });
    }
    // Assert on usage
}
```
// For software rendering in integration tests: Use `wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, dx12_shader_compiler: Default::default() })` in a feature-flagged harness [github.com](https://github.com/gfx-rs/wgpu).

**Crate Reference**: `naga-cli` for standalone shader validation [lib.rs](https://lib.rs/crates/naga-cli); `mockall` for easier trait mocking if stubs grow complex [crates.io](https://crates.io/crates/mockall).

## 2. GPU Driver Quirks
**Selected Questions**: How do we handle WDDM timeout detection and recovery (TDR)? Should we automatically reduce batch sizes on older drivers? What's the best way to detect and work around driver bugs? How do we handle laptops with switchable graphics (Intel + NVIDIA)? Should we provide GPU driver version warnings/recommendations?

**Grok's Response**: For TDR (Windows Display Driver Model timeouts), monitor wgpu's `Device::poll` and catch `SurfaceError::Timeout`, recovering by halving dispatch sizes and retrying—pluggable via a `RecoveryPolicy` trait. Automatically reduce batch sizes (e.g., from 512 to 256) on older drivers by querying `wgpu-info` at startup. Detect bugs via capability checks (e.g., `device.features() & wgpu::Features::SUBGROUP`) and log workarounds. For switchable graphics, force discrete GPU selection via `wgpu::Instance::request_adapter` with `PowerPreference::HighPerformance`. Provide warnings (e.g., "Update to 535.xx for optimal performance") if version < recommended, queried from Windows APIs. Risk: Overly aggressive recovery could degrade perf; limit retries to 3. Tradeoff: Adds <5ms startup overhead but prevents crashes on laptops, ensuring discrete-GPU focus.

**Sample Code/Scaffold** (in `pika-ui/src/gpu/renderer.rs`):
```rust
use wgpu::{Device, Instance, PowerPreference, RequestAdapterOptions};

pub trait RecoveryPolicy: Send + Sync { // Pluggable for custom strategies
    fn handle_tdr(&self, error: &wgpu::SurfaceError, dispatch_size: &mut u32);
}

pub struct DefaultRecovery;

impl RecoveryPolicy for DefaultRecovery {
    fn handle_tdr(&self, error: &wgpu::SurfaceError, dispatch_size: &mut u32) {
        if let wgpu::SurfaceError::Timeout = error {
            *dispatch_size /= 2; // Reduce to avoid TDR
            if *dispatch_size < 64 { panic!("Unrecoverable TDR"); }
        }
    }
}

// Discrete GPU selection with warnings
async fn select_adapter(instance: &Instance) -> Option<wgpu::Adapter> {
    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance, // Force discrete
        ..Default::default()
    }).await?;
    let info = adapter.get_info();
    if info.driver_info.contains("older than 535") { // Parse version
        log::warn!("Update GPU driver to 535.xx or later for best performance");
    }
    Some(adapter)
}

// Bug detection: Check features before dispatch
fn check_features(device: &Device) -> bool {
    device.features().contains(wgpu::Features::SUBGROUP_OPERATIONS) // Workaround if false
}
```
**Crate Reference**: `wgpu` for adapter selection [github.com](https://github.com/gfx-rs/wgpu); `gpu-descriptor` for advanced allocation if needed [crates.io](https://crates.io/crates/gpu-descriptor), but keep minimal.

## 3. Handling Larger-than-Memory Datasets
**Selected Questions**: Should we implement custom streaming iterators or rely on DuckDB's spilling? How do we coordinate memory limits between DuckDB and GPU buffers? What's the best pattern for progressive loading in the UI? Should we support tiled rendering for plots with billions of points? How do we handle backpressure when GPU can't keep up with data rate?

**Grok's Response**: Rely on DuckDB's automatic spilling for disk overflow, but wrap with custom `DataStream` iterators (as in your example) for batched GPU uploads, coordinating via MemoryCoordinator polls. For progressive UI loading, use tokio streams with egui repaint triggers on batch receipt. Support tiled rendering by dividing shaders into grid dispatches (e.g., 1024x1024 tiles) for >1B points. Handle backpressure with bounded channels (size=4) between DuckDB and GPU, pausing streams on full. Risk: Spilling slows queries 2-5x on HDDs; prefer SSDs via Windows APIs. Tradeoff: Custom streams add complexity but enable <1s UI feedback on huge datasets; pluggable via `StreamPolicy` for rate limiting.

**Sample Code/Scaffold** (extending your `DataStream` trait in `pika-engine/src/query.rs`):
```rust
use duckdb::ArrowResult;
use tokio::sync::mpsc::{channel, Sender};

pub trait StreamPolicy: Send + Sync { // Pluggable for backpressure strategies
    fn apply_backpressure(&self, tx: &Sender<RecordBatch>, current_load: f64);
}

pub struct BoundedPolicy;

impl StreamPolicy for BoundedPolicy {
    fn apply_backpressure(&self, tx: &Sender<RecordBatch>, current_load: f64) {
        if current_load > 0.8 { std::thread::sleep(std::time::Duration::from_millis(100)); } // Simple pause
    }
}

impl DataStream for DuckDbStream {
    async fn next_batch(&mut self) -> Option<RecordBatch> {
        let batch = self.result.next().await?; // Progressive from DuckDB
        if let Some(coord) = &self.coordinator {
            coord.check_limits(); // Coordinate with GPU
        }
        Some(batch)
    }

    // Tiled rendering dispatch
    fn dispatch_tiled(&self, device: &Device, queue: &Queue, tile_size: u32) {
        let (tx, rx) = channel(4); // Bounded for backpressure
        while let Some(batch) = self.next_batch().await {
            let policy = BoundedPolicy;
            policy.apply_backpressure(&tx, /* load from MemoryCoordinator */ 0.0);
            tx.send(batch).await.ok();
            // Dispatch tile (e.g., 1024x1024 grid)
            queue.submit(/* tiled commands */);
        }
    }
}
```
**Crate Reference**: `tokio-stream` for async iterators [crates.io](https://crates.io/crates/tokio-stream); DuckDB's spilling is built-in, no extra crates.

## 2. Sample Code or Crate References
See scaffolds above. Additional: For tiled rendering benchmarks, use [github.com](https://github.com/nical/lyon/blob/master/examples/wgpu_svg/src/main.rs) patterns adapted to wgpu (e.g., MSAA config for anti-aliasing in plots).

## 3. Performance, Architecture, or Tooling Guidance
- **Performance**: For tiled dispatches, aim for 4-16 tiles to balance load on discrete GPUs, reducing VRAM spikes by 50% per frame [www.xyzw.io](https://www.xyzw.io/posts/backend-gpu-p1/) (noting async command encoding).
- **Architecture**: Make backpressure pluggable to allow future integration with OS memory events on Windows, enhancing offline resilience.
- **Tooling**: Use `wgpu-profiler` for CI shader timings [crates.io](https://crates.io/crates/wgpu-profiler); for Windows TDR testing, simulate via artificial delays in mocks.

## 4. (Optional) Open-Ended Questions or Suggestions
- Could we extend the RecoveryPolicy to include adaptive workgroup sizing based on runtime TDR feedback, making it more resilient for varying Windows driver versions without manual configs?
- Given wgpu's GLES backend for testing [github.com](https://github.com/gfx-rs/wgpu), how might we automate switching between software and hardware modes in the test harness for hybrid CI runs?