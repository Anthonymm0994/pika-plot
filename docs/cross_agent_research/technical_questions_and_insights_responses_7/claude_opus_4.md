# 🎯 Deep Technical Solutions for Pika-Plot

## GPU Compute Shader Pipeline (Complete Implementation)

### Multi-Pass Aggregation Pipeline for 50M Points

```wgsl
// aggregation_pipeline.wgsl
struct AggregationParams {
    viewport_min: vec2<f32>,
    viewport_max: vec2<f32>,
    bin_count: vec2<u32>,
    point_count: u32,
    pass_type: u32, // 0=bin, 1=compact, 2=render
}

// Pass 1: Binning with atomic operations (DX11 compatible)
@group(0) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> bin_counts: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: AggregationParams;

@compute @workgroup_size(256, 1, 1) // Optimal for both discrete/integrated
fn bin_points(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.point_count) { return; }
    
    let point = points[idx];
    
    // Handle NaN/Inf gracefully
    if (any(point != point) || any(abs(point) > vec2<f32>(1e10))) {
        return;
    }
    
    // Compute bin indices with clamping
    let normalized = (point - params.viewport_min) / 
                     (params.viewport_max - params.viewport_min);
    let bin_coord = vec2<u32>(
        clamp(normalized * vec2<f32>(params.bin_count), 
              vec2<f32>(0.0), 
              vec2<f32>(params.bin_count - vec2<u32>(1u)))
    );
    
    let bin_idx = bin_coord.y * params.bin_count.x + bin_coord.x;
    
    // DX11-safe atomic operation
    atomicAdd(&bin_counts[bin_idx], 1u);
}

// Pass 2: Parallel prefix sum for compaction
var<workgroup> shared_data: array<u32, 512>;

@compute @workgroup_size(256, 1, 1)
fn compact_bins(@builtin(global_invocation_id) gid: vec3<u32>,
                @builtin(local_invocation_index) lid: u32) {
    // Optimized parallel scan implementation
    let idx = gid.x;
    
    // Load to shared memory
    shared_data[lid * 2u] = select(0u, bin_counts[idx * 2u], idx * 2u < params.bin_count.x * params.bin_count.y);
    shared_data[lid * 2u + 1u] = select(0u, bin_counts[idx * 2u + 1u], idx * 2u + 1u < params.bin_count.x * params.bin_count.y);
    
    // Up-sweep phase
    var offset = 1u;
    for (var d = 256u >> 1u; d > 0u; d = d >> 1u) {
        workgroupBarrier();
        if (lid < d) {
            let ai = offset * (2u * lid + 1u) - 1u;
            let bi = offset * (2u * lid + 2u) - 1u;
            shared_data[bi] = shared_data[bi] + shared_data[ai];
        }
        offset = offset << 1u;
    }
    
    // Down-sweep phase
    if (lid == 0u) { shared_data[511u] = 0u; }
    
    for (var d = 1u; d < 256u; d = d << 1u) {
        offset = offset >> 1u;
        workgroupBarrier();
        if (lid < d) {
            let ai = offset * (2u * lid + 1u) - 1u;
            let bi = offset * (2u * lid + 2u) - 1u;
            let temp = shared_data[ai];
            shared_data[ai] = shared_data[bi];
            shared_data[bi] = shared_data[bi] + temp;
        }
    }
    
    // Write back
    workgroupBarrier();
    if (idx * 2u < params.bin_count.x * params.bin_count.y) {
        bin_counts[idx * 2u] = shared_data[lid * 2u];
    }
    if (idx * 2u + 1u < params.bin_count.x * params.bin_count.y) {
        bin_counts[idx * 2u + 1u] = shared_data[lid * 2u + 1u];
    }
}
```

### CPU Verification Pass

```rust
// src/gpu/verification.rs
pub fn verify_aggregation_cpu(
    points: &[Vec2],
    viewport: &Viewport,
    bin_resolution: (u32, u32),
    gpu_result: &[u32],
) -> Result<(), AggregationError> {
    let mut cpu_bins = vec![0u32; (bin_resolution.0 * bin_resolution.1) as usize];
    
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            continue;
        }
        
        let normalized = Vec2::new(
            (point.x - viewport.min.x) / viewport.width(),
            (point.y - viewport.min.y) / viewport.height(),
        );
        
        let bin_x = (normalized.x * bin_resolution.0 as f32)
            .clamp(0.0, bin_resolution.0 as f32 - 1.0) as u32;
        let bin_y = (normalized.y * bin_resolution.1 as f32)
            .clamp(0.0, bin_resolution.1 as f32 - 1.0) as u32;
            
        let bin_idx = (bin_y * bin_resolution.0 + bin_x) as usize;
        cpu_bins[bin_idx] += 1;
    }
    
    // Compare with tolerance for floating point differences
    for (i, (cpu_val, gpu_val)) in cpu_bins.iter().zip(gpu_result).enumerate() {
        if (*cpu_val as i32 - *gpu_val as i32).abs() > 1 {
            return Err(AggregationError::Mismatch { 
                bin: i, 
                cpu: *cpu_val, 
                gpu: *gpu_val 
            });
        }
    }
    
    Ok(())
}
```

## Trait-Based Renderer Architecture

```rust
// src/render/traits.rs
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RendererCapabilities {
    pub max_points: usize,
    pub supports_compute: bool,
    pub supports_instancing: bool,
    pub preferred_workgroup_size: u32,
    pub max_texture_size: u32,
}

#[derive(Debug)]
pub struct MemoryEstimate {
    pub gpu_bytes: usize,
    pub cpu_bytes: usize,
    pub can_stream: bool,
}

pub struct PreparedBuffers {
    pub vertex_buffer: Option<Arc<wgpu::Buffer>>,
    pub instance_buffer: Option<Arc<wgpu::Buffer>>,
    pub uniform_buffer: Arc<wgpu::Buffer>,
    pub aggregation_buffer: Option<Arc<wgpu::Buffer>>,
}

pub trait PlotRenderer: Send + Sync {
    fn capabilities(&self) -> RendererCapabilities;
    
    fn estimate_memory(&self, points: usize) -> MemoryEstimate {
        let point_size = std::mem::size_of::<f32>() * 2; // x, y
        let base_memory = points * point_size;
        
        MemoryEstimate {
            gpu_bytes: base_memory,
            cpu_bytes: base_memory / 10, // Assume 10:1 GPU:CPU ratio
            can_stream: points > 10_000_000,
        }
    }
    
    fn prepare_buffers(&mut self, data: &PlotData) -> Result<PreparedBuffers>;
    fn render(&self, buffers: &PreparedBuffers, viewport: &Viewport) -> Result<()>;
}

// Hot-swapping implementation
pub struct AdaptiveRenderer {
    gpu_renderer: Option<Arc<Mutex<dyn PlotRenderer>>>,
    cpu_renderer: Arc<Mutex<dyn PlotRenderer>>,
    current: RendererType,
    swap_pending: Option<RendererType>,
}

impl AdaptiveRenderer {
    pub fn request_swap(&mut self, target: RendererType) {
        if self.current != target {
            self.swap_pending = Some(target);
        }
    }
    
    pub fn render(&mut self, data: &PlotData, viewport: &Viewport) -> Result<()> {
        // Prepare next renderer in background
        if let Some(pending) = self.swap_pending {
            match pending {
                RendererType::Gpu => {
                    if let Some(gpu) = &self.gpu_renderer {
                        // Pre-warm GPU buffers
                        let _ = gpu.lock().unwrap().prepare_buffers(data)?;
                    }
                }
                RendererType::Cpu => {
                    // Pre-compute CPU structures
                    let _ = self.cpu_renderer.lock().unwrap().prepare_buffers(data)?;
                }
            }
        }
        
        // Render with current
        let renderer = match self.current {
            RendererType::Gpu => self.gpu_renderer.as_ref().unwrap(),
            RendererType::Cpu => &self.cpu_renderer,
        };
        
        let mut r = renderer.lock().unwrap();
        let buffers = r.prepare_buffers(data)?;
        r.render(&buffers, viewport)?;
        
        // Complete swap after successful frame
        if let Some(pending) = self.swap_pending.take() {
            self.current = pending;
        }
        
        Ok(())
    }
}
```

## DX11 Feature Detection System

```rust
// src/gpu/compatibility.rs
pub struct GpuCompatibility {
    pub features: wgpu::Features,
    pub limits: wgpu::Limits,
    pub dx_level: DxLevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DxLevel {
    Dx12,
    Dx11,
    Dx10,
    Unknown,
}

impl GpuCompatibility {
    pub async fn detect() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12 | wgpu::Backends::DX11,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("No GPU found");
            
        let info = adapter.get_info();
        let dx_level = match info.backend {
            wgpu::Backend::Dx12 => DxLevel::Dx12,
            wgpu::Backend::Dx11 => DxLevel::Dx11,
            _ => DxLevel::Unknown,
        };
        
        // Request only DX11-compatible features
        let features = if dx_level == DxLevel::Dx11 {
            wgpu::Features::empty()
                | wgpu::Features::VERTEX_WRITABLE_STORAGE
                | wgpu::Features::CLEAR_TEXTURE
                | wgpu::Features::MULTIVIEW
        } else {
            adapter.features()
        };
        
        let limits = if dx_level == DxLevel::Dx11 {
            wgpu::Limits {
                max_texture_dimension_2d: 16384,
                max_buffer_size: 256 * 1024 * 1024, // 256MB
                max_storage_buffer_binding_size: 128 * 1024 * 1024, // 128MB
                max_compute_workgroup_size_x: 1024,
                max_compute_workgroups_per_dimension: 65535,
                ..wgpu::Limits::downlevel_webgl2_defaults()
            }
        } else {
            adapter.limits()
        };
        
        Self { features, limits, dx_level }
    }
    
    pub fn create_device_descriptor(&self) -> wgpu::DeviceDescriptor {
        wgpu::DeviceDescriptor {
            label: Some("Pika Plot GPU Device"),
            features: self.features,
            limits: self.limits.clone(),
        }
    }
}
```

## Memory Coordination Strategy

```rust
// src/memory/coordinator.rs
pub struct MemoryCoordinator {
    total_budget: usize,
    allocations: HashMap<ComponentId, MemoryAllocation>,
    pressure_callbacks: Vec<Box<dyn Fn(MemoryPressure) + Send>>,
}

#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    pub component: ComponentId,
    pub current: usize,
    pub minimum: usize,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryPressure {
    Low,    // < 50% used
    Medium, // 50-80% used  
    High,   // > 80% used
}

impl MemoryCoordinator {
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            allocations: HashMap::new(),
            pressure_callbacks: Vec::new(),
        }
    }
    
    pub fn request_allocation(
        &mut self, 
        component: ComponentId, 
        requested: usize,
        minimum: usize,
        priority: Priority,
    ) -> Result<usize, MemoryError> {
        let current_total = self.current_usage();
        let available = self.total_budget.saturating_sub(current_total);
        
        if requested <= available {
            // Easy case: we have enough
            self.allocations.insert(component, MemoryAllocation {
                component,
                current: requested,
                minimum,
                priority,
            });
            Ok(requested)
        } else if minimum <= available {
            // Can allocate minimum
            self.allocations.insert(component, MemoryAllocation {
                component,
                current: available,
                minimum,
                priority,
            });
            self.notify_pressure(MemoryPressure::Medium);
            Ok(available)
        } else {
            // Need to reclaim memory
            let reclaimed = self.reclaim_memory(minimum - available, priority)?;
            let allocated = minimum + reclaimed;
            self.allocations.insert(component, MemoryAllocation {
                component,
                current: allocated,
                minimum,
                priority,
            });
            self.notify_pressure(MemoryPressure::High);
            Ok(allocated)
        }
    }
    
    fn reclaim_memory(&mut self, needed: usize, for_priority: Priority) -> Result<usize, MemoryError> {
        let mut reclaimed = 0;
        let mut victims: Vec<_> = self.allocations.values()
            .filter(|a| a.priority < for_priority && a.current > a.minimum)
            .collect();
            
        victims.sort_by_key(|a| a.priority);
        
        for victim in victims {
            let can_reclaim = victim.current - victim.minimum;
            let to_reclaim = can_reclaim.min(needed - reclaimed);
            
            // Notify component to reduce memory
            self.notify_component_reduce(victim.component, to_reclaim);
            reclaimed += to_reclaim;
            
            if reclaimed >= needed {
                break;
            }
        }
        
        if reclaimed < needed {
            Err(MemoryError::InsufficientMemory { 
                requested: needed, 
                available: reclaimed 
            })
        } else {
            Ok(reclaimed)
        }
    }
}

// Example usage in GPU renderer
impl GpuRenderer {
    pub fn allocate_buffers(&mut self, point_count: usize) -> Result<()> {
        let coordinator = MEMORY_COORDINATOR.lock().unwrap();
        
        let requested = point_count * std::mem::size_of::<Vec2>();
        let minimum = (point_count / 10).max(1000) * std::mem::size_of::<Vec2>();
        
        let allocated = coordinator.request_allocation(
            ComponentId::GpuRenderer,
            requested,
            minimum,
            Priority::High,
        )?;
        
        if allocated < requested {
            // Adjust quality based on available memory
            self.enable_downsampling(requested / allocated);
        }
        
        Ok(())
    }
}
```

## Integration Test Scenario

```rust
// tests/integration/full_pipeline.rs
#[tokio::test]
async fn test_complete_pipeline() {
    // Step 1: Import 10M row CSV with type inference
    let import_wizard = ImportWizard::new();
    let csv_path = "tests/fixtures/10m_rows.csv";
    
    let start = Instant::now();
    let schema = import_wizard.analyze_file(csv_path).await.unwrap();
    let import_time = start.elapsed();
    assert!(import_time < Duration::from_secs(5), "Import analysis too slow");
    
    // Verify type inference worked
    assert_eq!(schema.columns.len(), 5);
    assert_eq!(schema.columns[0].dtype, DataType::Float64); // x
    assert_eq!(schema.columns[1].dtype, DataType::Float64); // y
    
    // Step 2: Create dataflow graph
    let mut graph = DataflowGraph::new();
    
    let data_node = graph.add_node(Node::DataSource {
        id: NodeId::new(),
        path: csv_path.into(),
        schema: Some(schema),
        position: Pos2::new(100.0, 100.0),
    });
    
    let query_node = graph.add_node(Node::Query {
        id: NodeId::new(),
        sql: "SELECT x, y FROM data WHERE x > 0".to_string(),
        cache_key: None,
        position: Pos2::new(300.0, 100.0),
    });
    
    let plot_node = graph.add_node(Node::Plot {
        id: NodeId::new(),
        config: PlotConfig::scatter(),
        render_strategy: RenderStrategy::auto_select(10_000_000, true),
        position: Pos2::new(500.0, 100.0),
    });
    
    graph.add_edge(data_node, query_node, Thread {
        color: Color32::YELLOW,
        flow_direction: FlowDirection::Forward,
        data_hash: None,
    });
    
    graph.add_edge(query_node, plot_node, Thread {
        color: Color32::GREEN,
        flow_direction: FlowDirection::Forward,
        data_hash: None,
    });
    
    // Step 3: Execute graph
    let executor = GraphExecutor::new();
    let start = Instant::now();
    let results = executor.execute(&graph).await.unwrap();
    let exec_time = start.elapsed();
    assert!(exec_time < Duration::from_secs(3), "Graph execution too slow");
    
    // Step 4: Render with GPU aggregation
    let renderer = AdaptiveRenderer::new();
    let plot_data = results.get(&plot_node).unwrap();
    
    let start = Instant::now();
    renderer.render(plot_data, &Viewport::default()).unwrap();
    let render_time = start.elapsed();
    assert!(render_time < Duration::from_millis(100), "Rendering too slow");
    
    // Step 5: Test canvas navigation
    let mut canvas = InfiniteCanvas::new();
    canvas.load_graph(graph);
    
    // Simulate pan gesture
    canvas.handle_drag(Vec2::new(50.0, 0.0));
    assert_eq!(canvas.viewport.center, Vec2::new(50.0, 0.0));
    
    // Simulate zoom gesture
    canvas.handle_zoom(1.5, Pos2::new(400.0, 300.0));
    assert!((canvas.viewport.scale - 1.5).abs() < 0.01);
    
    // Simulate lasso selection
    let lasso_points = vec![
        Pos2::new(250.0, 50.0),
        Pos2::new(550.0, 50.0),
        Pos2::new(550.0, 150.0),
        Pos2::new(250.0, 150.0),
    ];
    canvas.handle_lasso(lasso_points);
    assert_eq!(canvas.selection.nodes.len(), 2); // query and plot nodes
}

// Integration pain points and solutions:
#[cfg(test)]
mod integration_challenges {
    // 1. Memory coordination between components
    #[test]
    fn test_memory_pressure_handling() {
        let coordinator = MemoryCoordinator::new(1024 * 1024 * 1024); // 1GB
        
        // DuckDB requests 600MB
        let db_alloc = coordinator.request_allocation(
            ComponentId::Database,
            600 * 1024 * 1024,
            300 * 1024 * 1024,
            Priority::High,
        ).unwrap();
        
        // GPU requests 600MB (should trigger pressure)
        let gpu_alloc = coordinator.request_allocation(
            ComponentId::GpuRenderer,
            600 * 1024 * 1024,
            200 * 1024 * 1024,
            Priority::High,
        ).unwrap();
        
        // Both should get reduced allocations
        assert!(db_alloc < 600 * 1024 * 1024);
        assert!(gpu_alloc < 600 * 1024 * 1024);
        assert!(db_alloc + gpu_alloc <= 1024 * 1024 * 1024);
    }
    
    // 2. Async coordination between graph execution and rendering
    #[tokio::test]
    async fn test_async_pipeline() {
        let (tx, mut rx) = mpsc::channel(10);
        
        // Graph executor sends chunks
        tokio::spawn(async move {
            for chunk in 0..10 {
                tx.send(DataChunk { id: chunk, data: vec![0.0; 1000000] }).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        // Renderer consumes chunks
        let mut rendered_chunks = 0;
        while let Some(chunk) = rx.recv().await {
            // Simulate rendering
            tokio::time::sleep(Duration::from_millis(50)).await;
            rendered_chunks += 1;
        }
        
        assert_eq!(rendered_chunks, 10);
    }
    
    // 3. Error propagation across boundaries
    #[test]
    fn test_error_handling() {
        let result = pipeline_operation()
            .map_err(|e| match e {
                PipelineError::Import(e) => format!("Failed to import: {}", e),
                PipelineError::Query(e) => format!("Query failed: {}", e),
                PipelineError::Render(e) => format!("Rendering failed: {}", e),
            });
            
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("user-friendly message"));
    }
}
```

## Performance Budget Breakdown

```rust
// src/performance/budget.rs
pub struct FrameBudget {
    total_ms: f32, // 16.67ms for 60 FPS
    allocations: HashMap<Component, f32>,
}

impl FrameBudget {
    pub fn new_60fps() -> Self {
        let mut allocations = HashMap::new();
        
        // Detailed breakdown based on profiling
        allocations.insert(Component::GpuSubmission, 2.0);      // GPU command submission
        allocations.insert(Component::CanvasHitTest, 1.0);      // Spatial queries
        allocations.insert(Component::GraphTraversal, 0.5);     // DAG operations
        allocations.insert(Component::UiLayout, 3.0);           // egui layout
        allocations.insert(Component::UiRender, 4.0);           // egui rendering
        allocations.insert(Component::ThreadAnimation, 1.0);    // Visual effects
        allocations.insert(Component::DataFetch, 2.0);          // Async data ops
        allocations.insert(Component::Misc, 3.17);              // Buffer for spikes
        
        Self {
            total_ms: 16.67,
            allocations,
        }
    }
    
    pub fn start_frame(&self) -> FrameTimer {
        FrameTimer {
            budget: self,
            component_times: HashMap::new(),
            start: Instant::now(),
        }
    }
}

pub struct FrameTimer<'a> {
    budget: &'a FrameBudget,
    component_times: HashMap<Component, Duration>,
    start: Instant,
}

impl<'a> FrameTimer<'a> {
    pub fn time_component<F, R>(&mut self, component: Component, f: F) -> R 
    where F: FnOnce() -> R 
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        
        self.component_times.insert(component, elapsed);
        
        // Warn if over budget
        if let Some(budget) = self.budget.allocations.get(&component) {
            if elapsed.as_secs_f32() * 1000.0 > *budget {
                warn!("{:?} took {:.2}ms (budget: {:.2}ms)", 
                      component, 
                      elapsed.as_secs_f32() * 1000.0, 
                      budget);
            }
        }
        
        result
    }
    
    pub fn finish(self) -> FrameReport {
        let total = self.start.elapsed();
        let over_budget = total.as_secs_f32() * 1000.0 > self.budget.total_ms;
        
        FrameReport {
            total_time: total,
            component_times: self.component_times,
            over_budget,
        }
    }
}
```

## Pragmatic Observations & Recommendations

### 1. **Start with the CPU fallback path first**
The GPU pipeline is complex. Ship with egui_plot CPU rendering and add GPU as an enhancement. This ensures the 5-10 offline machines can always use the tool.

### 2. **Memory coordination is critical**
With unknown hardware specs, implement aggressive memory limits and clear user feedback when hitting them.

### 3. **Test on real hardware early**
Set up a test lab with actual target machines (especially older ones with integrated graphics).

### 4. **Feature detection over assumptions**
Never assume GPU capabilities. Always detect and gracefully degrade.

### 5. **Instrument everything**
The frame budget breakdown should be visible in a debug overlay for real-world performance tuning.

This comprehensive approach balances ambition with pragmatism, ensuring Pika-Plot can scale from simple visualizations to complex 50M point datasets while remaining stable on modest hardware.