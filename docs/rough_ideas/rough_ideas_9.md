# 🚀 Architecting Pika-Plot: From Vision to Implementation

I love the vision here - a truly fluid data exploration tool that makes the complex feel effortless. Let me help architect this beast.

## 🏗️ Architecture Analysis & Improvements

Your layered architecture is solid, but I'd suggest a few refinements:

### 1. **Event-Driven Core with Actor Model**
Instead of direct coupling, use an event bus with typed messages. This enables:
- Clean separation between UI and compute
- Easy testing (just send/receive messages)
- Natural async boundaries
- Undo/redo practically writes itself

### 2. **Trait-Based Node System**
Make nodes composable through traits:

```rust
// Core node traits
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn position(&self) -> Point2;
    fn bounds(&self) -> Rect;
    fn render(&self, ui: &mut egui::Ui, ctx: &CanvasContext);
}

pub trait DataNode: Node {
    fn output_schema(&self) -> &Schema;
    fn compute(&self, ctx: &ComputeContext) -> Result<RecordBatch>;
    fn fingerprint(&self) -> u64;
}

pub trait Interactive: Node {
    fn on_click(&mut self, pos: Point2) -> Option<Event>;
    fn on_drag(&mut self, delta: Vec2) -> Option<Event>;
}

// Specific node types compose traits
pub struct TableNode {
    id: NodeId,
    source_path: PathBuf,
    schema: Schema,
    // ...
}

impl Node for TableNode { /* ... */ }
impl DataNode for TableNode { /* ... */ }
```

### 3. **Streaming-First Data Pipeline**
For 10M+ row interactivity, think in terms of data streams:

```rust
pub trait DataStream: Send {
    fn schema(&self) -> &Schema;
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn estimated_rows(&self) -> Option<usize>;
}

pub trait PlotSampler {
    fn sample_for_viewport(
        &self,
        stream: Box<dyn DataStream>,
        viewport: ViewportBounds,
        target_points: usize,
    ) -> Result<PlotData>;
}
```

## 🎯 Improved Module Structure

```
pika-plot/
├── Cargo.toml
├── crates/
│   ├── pika-core/           # Core types, traits, events
│   ├── pika-canvas/         # Canvas rendering, node management
│   ├── pika-compute/        # SQL parsing, caching, execution
│   ├── pika-storage/        # DuckDB interface, Arrow management
│   ├── pika-plot/           # Plot rendering, sampling algorithms
│   ├── pika-ui/             # Reusable UI components
│   └── pika-app/            # Main application, orchestration
├── assets/
└── tests/
    └── integration/
```

### Key Module Responsibilities:

#### `pika-core`
```rust
// Event system
pub enum AppEvent {
    Canvas(CanvasEvent),
    Compute(ComputeEvent),
    Storage(StorageEvent),
}

// Shared types
pub struct NodeId(Uuid);
pub struct QueryFingerprint(u64);

// Core traits
pub trait EventHandler {
    fn handle(&mut self, event: AppEvent) -> Result<Vec<AppEvent>>;
}
```

#### `pika-compute`
```rust
pub struct ComputeEngine {
    cache: SemanticCache,
    executor: DuckDbExecutor,
}

impl ComputeEngine {
    pub async fn execute_query(&self, query: Query) -> Result<QueryResult> {
        let fingerprint = self.normalize_and_fingerprint(&query)?;
        
        if let Some(cached) = self.cache.get(&fingerprint).await {
            return Ok(cached);
        }
        
        let result = self.executor.run(query).await?;
        self.cache.insert(fingerprint, result.clone()).await;
        Ok(result)
    }
}
```

#### `pika-plot`
```rust
pub struct PlotEngine {
    samplers: HashMap<PlotType, Box<dyn PlotSampler>>,
    gpu_backend: Option<WgpuBackend>,
}

impl PlotEngine {
    pub fn prepare_plot_data(
        &self,
        data: RecordBatch,
        plot_type: PlotType,
        viewport: ViewportBounds,
    ) -> Result<PlotRenderData> {
        let sampler = self.samplers.get(&plot_type)?;
        
        // Smart sampling based on data size and viewport
        match data.num_rows() {
            0..=10_000 => PlotRenderData::Full(data),
            10_001..=100_000 => {
                let sampled = sampler.sample_lttb(data, viewport.pixel_count())?;
                PlotRenderData::Sampled(sampled)
            }
            _ => {
                let aggregated = sampler.aggregate_bins(data, viewport)?;
                PlotRenderData::Aggregated(aggregated)
            }
        }
    }
}
```

## 🧪 Testing Strategy

### 1. **Property-Based Testing**
Use `proptest` for semantic cache correctness:
```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn equivalent_queries_same_fingerprint(
            query1 in arb_query(),
            permutation in prop::collection::vec(0..10usize, 0..10)
        ) {
            let query2 = permute_query(query1.clone(), &permutation);
            let fp1 = fingerprint(&query1);
            let fp2 = fingerprint(&query2);
            
            if semantically_equivalent(&query1, &query2) {
                prop_assert_eq!(fp1, fp2);
            }
        }
    }
}
```

### 2. **Snapshot Testing**
For UI components and plot rendering:
```rust
#[test]
fn test_scatter_plot_rendering() {
    let data = generate_test_data(1000);
    let plot = ScatterPlot::new(data);
    
    insta::assert_yaml_snapshot!(plot.render_metadata());
    insta::assert_debug_snapshot!(plot.sample_points(100));
}
```

### 3. **Benchmarking Suite**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_plot_sampling(c: &mut Criterion) {
    c.bench_function("lttb_1m_points", |b| {
        let data = generate_points(1_000_000);
        b.iter(|| sample_lttb(black_box(&data), 1000))
    });
}
```

## 📋 First 5 Implementation Tasks

### Task 1: Core Event System & Node Traits
```rust
// File: crates/pika-core/src/events.rs
// Implement the event bus, node traits, and basic message passing
```

### Task 2: Canvas State Management
```rust
// File: crates/pika-canvas/src/graph.rs
// Node graph, spatial indexing, connection management
// Start with just TableNode and QueryNode
```

### Task 3: DuckDB Integration Layer
```rust
// File: crates/pika-storage/src/duckdb_engine.rs
// Wrap DuckDB with async interface, implement CSV import
// Reuse your existing import wizard UI
```

### Task 4: Semantic Query Cache
```rust
// File: crates/pika-compute/src/semantic_cache.rs
// AST normalization, fingerprinting, LRU cache
```

### Task 5: Basic Scatter Plot with LTTB
```rust
// File: crates/pika-plot/src/scatter.rs
// Implement LTTB sampling, basic egui_plot integration
// Target: 1M points interactive at 60fps
```

## 🎨 Interactive Plot Strategy for 10M+ Rows

Based on [duckdb's](https://blobs.duckdb.org/docs/duckdb-docs-0.9.2.pdf) capabilities and [local-first principles](https://www.powersync.com/blog/local-first-is-a-big-deal-especially-for-the-web):

1. **Level-of-Detail (LOD) System**
   - Precompute multiple resolution levels
   - Switch based on zoom level
   - Use DuckDB's window functions for binning

2. **Progressive Rendering**
   - Render low-res immediately
   - Stream in higher detail
   - Cancel/restart on viewport change

3. **Smart Caching**
   - Cache commonly viewed regions
   - Prefetch adjacent areas
   - Use Arrow's zero-copy for speed

Example implementation:
```rust
pub struct AdaptivePlotData {
    levels: Vec<LodLevel>,
    active_level: usize,
}

struct LodLevel {
    resolution: f64,
    data: RecordBatch,
    index: SpatialIndex,
}

impl AdaptivePlotData {
    pub fn query_viewport(&self, bounds: Rect) -> PlotPoints {
        let level = self.select_level(bounds);
        level.index.query_rect(bounds)
    }
}
```


## 🚦 Getting Started

1. Set up the workspace with the crate structure
2. Implement `pika-core` with basic event system
3. Port your CSV import to use the event system
4. Build a minimal canvas with just table nodes
5. Add query nodes with results preview

The beauty of this architecture is that each piece can be built and tested independently. Start simple, ship often, and let the architecture guide you toward that buttery-smooth UX.

Want me to elaborate on any specific part or provide more detailed code examples for a particular module?