# 🚀 Pika-Plot: Final Architecture Plan v2

## 📋 Key Requirements (Hard Constraints)

1. **Dual UI Modes from Start**: Both notebook (guided) and canvas (freeform) views must be available immediately. No phased features.
2. **Discrete GPU Required**: Assume users have capable discrete GPUs. No integrated graphics support needed.
3. **Strictly Offline**: No server mode, cloud sync, or web exports. Only file-based exports (images, CSV, JSON).
4. **Memory-Aware**: Datasets typically fit in RAM, but graceful degradation required. Downsampling for visualization is acceptable.
5. **Windows 10/11 Native**: Built with Rust + egui, no web technologies.
6. **Scale Focus**: Handle millions of data points through intelligent aggregation, not raw rendering.

---

## 🎯 Simplified Architecture Overview

Based on critique feedback and clarified requirements, this is a pragmatic yet powerful architecture that delivers on the vision without overengineering.

### Core Philosophy
- **Proven over Novel**: Use battle-tested patterns
- **Simple over Clever**: Favor maintainability
- **Explicit over Magic**: Clear data flow and error handling
- **Performance through Design**: Not premature optimization

---

## 🗂️ Streamlined Crate Structure

```
pika-plot/
├── pika-core/      # Shared types, traits, events
├── pika-engine/    # DuckDB, caching, compute
├── pika-ui/        # All UI components and canvas logic
└── pika-app/       # Binary, window management, orchestration
```

### Crate Responsibilities

#### `pika-core` - Foundation
```rust
// Core types shared across all crates
pub struct NodeId(Uuid);
pub struct QueryResult {
    pub data: Arc<RecordBatch>,
    pub execution_time: Duration,
    pub row_count: usize,
}

// Event system for UI-Engine communication
pub enum AppEvent {
    // UI -> Engine
    ExecuteQuery { id: NodeId, sql: String },
    ImportCsv { path: PathBuf, options: ImportOptions },
    SampleForPlot { source: NodeId, config: PlotConfig },
    
    // Engine -> UI
    QueryComplete { id: NodeId, result: Result<QueryResult> },
    ImportComplete { table_name: String, schema: Schema },
    PlotDataReady { id: NodeId, buffer: GpuBuffer },
}

// Simple traits, no overengineering
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn render(&mut self, ui: &mut egui::Ui, ctx: &AppContext);
}
```

#### `pika-engine` - Data Processing
- DuckDB integration (no abstraction layer)
- Simple 2-tier cache (queries + GPU buffers)
- CSV import with type inference
- Async query execution via Tokio

#### `pika-ui` - User Interface
- Dual-mode workspace (notebook + canvas)
- Node implementations (Table, Query, Plot)
- Export dialogs
- GPU-accelerated plot rendering

#### `pika-app` - Application Shell
- Window lifecycle
- Event routing between UI and Engine
- Settings persistence
- Memory monitoring

---

## 🧠 Simplified System Design

### Two-Tier Cache (Pragmatic Approach)

```rust
pub struct QueryCache {
    // Simple LRU cache for query results
    cache: moka::Cache<QueryFingerprint, Arc<RecordBatch>>,
}

pub struct PlotCache {
    // GPU buffers for visible plots only
    visible_plots: DashMap<NodeId, GpuBuffer>,
}

impl QueryCache {
    pub fn get_or_compute<F>(&self, sql: &str, compute: F) -> Result<Arc<RecordBatch>>
    where F: FnOnce() -> Result<RecordBatch>
    {
        // Simple query normalization (lowercase, trim whitespace)
        let key = self.simple_fingerprint(sql);
        
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }
        
        let result = Arc::new(compute()?);
        self.cache.insert(key, result.clone());
        Ok(result)
    }
}
```

### Event-Driven Architecture (Simplified)

```rust
// Single channel pair for UI <-> Engine communication
pub struct AppRuntime {
    ui_to_engine: mpsc::Sender<AppEvent>,
    engine_to_ui: mpsc::Receiver<AppEvent>,
    engine_thread: JoinHandle<()>,
}

// Engine runs on dedicated thread pool
impl Engine {
    pub async fn run(mut self, rx: mpsc::Receiver<AppEvent>, tx: mpsc::Sender<AppEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                AppEvent::ExecuteQuery { id, sql } => {
                    let result = self.execute_query(&sql).await;
                    let _ = tx.send(AppEvent::QueryComplete { id, result }).await;
                }
                // ... other events
            }
        }
    }
}
```

---

## 📊 UI Design: Dual-Mode Workspace

### WorkspaceMode Enum
```rust
pub enum WorkspaceMode {
    Notebook {
        cells: Vec<NotebookCell>,
        active_cell: Option<usize>,
    },
    Canvas {
        nodes: HashMap<NodeId, CanvasNode>,
        connections: Vec<Connection>,
        camera: Camera2D,
    },
}

pub struct Workspace {
    mode: WorkspaceMode,
    // Mode can be switched at any time
    mode_toggle: bool,
}
```

### Notebook Mode
- Linear cell-based interface
- Auto-layout with drag-to-reorder
- Collapsible cells
- Familiar for data analysts

### Canvas Mode  
- Free-form node placement
- Visual connections between nodes
- Pan/zoom navigation
- Mini-map for orientation

### Shared Features
- Same node types in both modes
- Consistent keyboard shortcuts
- Unified export functionality
- Mode switching preserves content

---

## 🚀 GPU-Accelerated Rendering

Since we're targeting discrete GPUs, we can be ambitious with rendering.

### Plot Rendering Pipeline

```rust
pub struct GpuPlotRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Different pipelines for different data scales
    pipelines: PlotPipelines,
}

pub struct PlotPipelines {
    // Direct rendering for small datasets
    direct: RenderPipeline,
    
    // Instanced rendering for medium datasets  
    instanced: RenderPipeline,
    
    // Compute shader aggregation for large datasets
    aggregation: ComputePipeline,
}

impl GpuPlotRenderer {
    pub fn render(&self, plot_data: &PlotData, target: &wgpu::TextureView) {
        match plot_data.point_count() {
            0..=50_000 => self.render_direct(plot_data, target),
            50_001..=5_000_000 => self.render_instanced(plot_data, target),
            _ => self.render_aggregated(plot_data, target),
        }
    }
}
```

### Integration with egui

```rust
// Use egui::PaintCallback for GPU rendering
impl PlotNode {
    fn render(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            self.size,
            egui::Sense::click_and_drag(),
        );
        
        // GPU rendering via paint callback
        ui.painter().add(egui::paint::PaintCallback {
            rect,
            callback: Arc::new(egui::paint::CallbackFn::new(move |info, painter| {
                self.gpu_renderer.render_to_egui(info, painter);
            })),
        });
    }
}
```

---

## 📦 Data Handling & Memory Management

### DuckDB Integration

```rust
pub struct StorageEngine {
    conn: Arc<Mutex<duckdb::Connection>>,
    import_options: ImportOptions,
}

impl StorageEngine {
    pub async fn import_csv(&self, path: &Path) -> Result<TableInfo> {
        // Use DuckDB's native CSV reader
        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_csv_auto('{}', sample_size=100000)",
            generate_table_name(path),
            path.display()
        );
        
        self.conn.lock().await.execute(&sql, [])?;
        
        // Return schema info for UI
        self.get_table_info(table_name).await
    }
    
    pub async fn execute_query(&self, sql: &str) -> Result<RecordBatch> {
        // Execute and convert to Arrow format
        let arrow_result = self.conn.lock().await
            .execute_arrow(&sql)?;
        
        // Check memory before materializing
        if self.would_exceed_memory_limit(&arrow_result) {
            return Err(Error::MemoryLimit);
        }
        
        Ok(arrow_result.collect())
    }
}
```

### Memory-Aware Loading

```rust
pub struct MemoryMonitor {
    warning_threshold: f64, // 0.8 = 80% RAM
    max_threshold: f64,     // 0.95 = 95% RAM
}

impl MemoryMonitor {
    pub fn check_before_operation(&self, estimated_bytes: usize) -> Result<()> {
        let available = self.available_memory();
        let after_op = self.used_memory() + estimated_bytes;
        
        if after_op > available * self.max_threshold {
            Err(Error::InsufficientMemory { 
                required: estimated_bytes,
                available: available - self.used_memory(),
            })
        } else if after_op > available * self.warning_threshold {
            // Show warning toast but proceed
            Ok(())
        } else {
            Ok(())
        }
    }
}
```

---

## 📤 Export System (File-Based Only)

### Supported Exports

```rust
pub enum ExportFormat {
    // Images
    Png { resolution: Resolution, dpi: u32 },
    Svg { embed_fonts: bool },
    
    // Data
    Csv { delimiter: char },
    Json { pretty: bool },
    
    // Workspace
    PikaPlot { version: u32 }, // Recipe-based, no data
}

pub struct ExportManager {
    pub async fn export_plot(&self, plot: &PlotNode, format: ExportFormat) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Png { resolution, dpi } => {
                self.render_plot_to_png(plot, resolution, dpi).await
            }
            ExportFormat::Svg { .. } => {
                self.render_plot_to_svg(plot).await
            }
            _ => Err(Error::InvalidExportFormat),
        }
    }
    
    pub async fn export_data(&self, data: &RecordBatch, format: ExportFormat) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Csv { delimiter } => {
                let mut writer = csv::Writer::from_writer(vec![]);
                // Write Arrow data to CSV
                Ok(writer.into_inner()?)
            }
            ExportFormat::Json { pretty } => {
                // Convert to JSON
                Ok(serde_json::to_vec_pretty(data)?)
            }
            _ => Err(Error::InvalidExportFormat),
        }
    }
}
```

### Workspace Snapshots (Recipe-Based)

```rust
#[derive(Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    version: u32,
    mode: WorkspaceMode,
    nodes: Vec<NodeSnapshot>,
    
    // References to source files, not embedded data
    data_sources: Vec<DataSourceRef>,
}

#[derive(Serialize, Deserialize)]
pub struct DataSourceRef {
    original_path: PathBuf,
    table_name: String,
    import_options: ImportOptions,
    file_hash: String, // To detect changes
}
```

---

## 🧪 Testing Strategy (Pragmatic)

### Distribution
- **50%** Correctness tests (unit + property)
- **30%** Integration tests (Windows-specific edge cases)
- **20%** Performance benchmarks

### Key Test Areas

```rust
// Correctness: Query cache behavior
#[test]
fn test_query_cache_normalization() {
    let cache = QueryCache::new();
    
    // These should hit the same cache entry
    let q1 = "SELECT * FROM sales WHERE price > 100";
    let q2 = "select * from sales where price > 100";
    let q3 = "SELECT * FROM sales WHERE price>100";
    
    assert_eq!(
        cache.simple_fingerprint(q1),
        cache.simple_fingerprint(q2)
    );
}

// Integration: Full workflow
#[tokio::test]
async fn test_csv_to_plot_workflow() {
    let app = TestApp::new();
    
    // Import CSV
    app.import_csv("fixtures/test_data.csv").await?;
    
    // Execute query
    let result = app.execute_query("SELECT x, y FROM test_data").await?;
    
    // Create plot
    let plot_data = app.prepare_plot(result, PlotType::Scatter).await?;
    
    // Verify GPU buffer created
    assert!(plot_data.gpu_buffer.is_some());
}

// Performance: Benchmark aggregation strategies
fn bench_aggregation_algorithms(c: &mut Criterion) {
    for size in [1_000_000, 10_000_000, 50_000_000] {
        let data = generate_scatter_data(size);
        
        c.bench_function(&format!("aggregate_{}", size), |b| {
            b.iter(|| aggregate_for_viewport(&data, ViewportBounds::default()))
        });
    }
}
```

---

## 🎯 Implementation Priorities

Given all features are needed from the start, here's the build order:

### Phase 1: Foundation (Weeks 1-2)
1. Set up 4-crate workspace
2. DuckDB integration with Arrow
3. Basic event system
4. Memory monitoring

### Phase 2: Core Features (Weeks 3-4)
1. CSV import with type inference
2. Query execution with simple cache
3. Basic node types (Table, Query)
4. Both UI modes (notebook + canvas)

### Phase 3: GPU Visualization (Weeks 5-6)
1. wgpu setup with egui::PaintCallback
2. Plot rendering pipelines
3. Aggregation compute shaders
4. Plot node implementation

### Phase 4: Polish (Weeks 7-8)
1. Export functionality
2. Workspace snapshots
3. Error handling
4. Performance optimization

---

## 🔍 Key Architectural Decisions

### What We're Building
- **Pragmatic caching**: 2-tier instead of 4-tier hierarchy
- **GPU-first rendering**: Leveraging discrete GPU assumption
- **Dual-mode UI**: Both paradigms available immediately
- **Simple event system**: Single channel pair, clear flow
- **Recipe-based snapshots**: References, not embedded data

### What We're NOT Building
- Storage abstraction layers
- Integrated GPU fallbacks
- Server/cloud features
- Complex predictive caching
- Progressive feature rollout

### Trade-offs Accepted
- Requires discrete GPU (excludes some users)
- No collaboration features (pure offline)
- Memory limits mean some datasets won't work
- Snapshots require original files present

---

## 🚀 Summary

This architecture delivers a powerful, GPU-accelerated data exploration tool without unnecessary complexity. By accepting the constraints (discrete GPU, offline-only, memory limits) we can focus on building an exceptional experience for the target users.

The dual-mode UI gives users choice without forcing a paradigm. The simplified caching provides most benefits without the complexity. GPU acceleration from the start ensures smooth interaction with large datasets.

Most importantly, this is achievable. Every component uses proven patterns, existing libraries, and clear interfaces. The result will be a fast, stable, and genuinely useful tool for data exploration. 