# 🔍 Architecture Review: Pika-Plot

## Storage Concerns

### DuckDB Single Point of Failure
The architecture commits entirely to DuckDB without clear fallback strategies. While DuckDB excels at analytics, this creates several risks:

**Hidden Assumptions**: The author assumes DuckDB's embedded mode will handle concurrent access patterns from multiple async workers without issues. DuckDB's concurrency model differs significantly from SQLite's - it uses optimistic concurrency control that can lead to transaction conflicts under heavy parallel load.

**Alternative**: Implement a storage abstraction layer:
```rust
trait DataBackend: Send + Sync {
    async fn execute(&self, query: &str) -> Result<RecordBatch>;
    async fn import_csv(&self, path: &Path, options: ImportOptions) -> Result<()>;
}
```
This allows swapping backends or even using DuckDB for analytics while SQLite handles metadata - a proven pattern.

### CSV Import Memory Explosion
The CSV import strategy shows concerning optimism about memory usage:
```rust
// "Stream data using DuckDB's optimized CSV reader"
self.duckdb.execute(&import_sql, [])?;
```

**Reality Check**: DuckDB's CSV reader loads entire chunks into memory before processing. For a 10GB CSV with wide columns, this can easily explode memory on typical developer machines.

**Better Approach**: Implement chunked streaming:
```rust
let reader = csv::Reader::from_path(path)?;
let mut batch_builder = RecordBatchBuilder::new(schema);

for chunk in reader.chunks(10_000) {
    let batch = batch_builder.push_chunk(chunk)?;
    duckdb.append_batch(&table, batch)?;
    batch_builder.reset();
}
```

## Query Engine Design

### Semantic Cache Overengineering
The semantic fingerprinting system is intellectually appealing but practically fragile:

**Hidden Complexity**: 
- AST normalization rules will constantly need updates as SQL features are added
- "WHERE x>5" vs "WHERE x>=6" semantic equivalence detection is non-trivial
- Maintenance burden grows exponentially with SQL feature coverage

**Simpler Alternative**: Use query parameterization:
```rust
struct ParameterizedQuery {
    template: String,  // "SELECT * FROM t WHERE x > ?"
    params: Vec<Value>,
}
```
This handles 90% of cache hits with 10% of the complexity.

### Cache Hierarchy Complexity
The 4-tier cache system (Primary, Derived, Spatial, GPU) creates numerous failure modes:

**Coordination Overhead**: Each cache level needs invalidation logic when data changes. The proposed "smart eviction" that builds dependency graphs will thrash on real workloads where users rapidly iterate.

**Alternative**: Two-tier cache:
1. **Query Results** - Simple LRU with size bounds
2. **GPU Buffers** - Pinned for visible plots only

## UI/UX Weaknesses

### Canvas Metaphor Confusion
The infinite canvas with nodes/connections mimics Blender/Houdini but doesn't match typical data exploration patterns:

**Assumption Mismatch**: The author assumes users think in dataflow graphs. Most data analysts think in notebooks (linear) or dashboards (grid-based).

**Better Metaphor**: Adaptive workspace that starts as a grid but allows free-form arrangement:
```rust
enum WorkspaceMode {
    Grid { columns: usize },      // Default: auto-layout
    Canvas { positions: HashMap<NodeId, Point> },  // User takes control
}
```

### Plot Node Overload
Stuffing configuration, data preview, and visualization into one "Plot Node" creates information overload:

**Cleaner Separation**:
- **Data Node**: Shows schema, row count, preview
- **Transform Node**: Shows SQL, execution status  
- **Viz Node**: Only shows the plot, minimal chrome

This matches how users think: data → transform → visualize.

## Performance Assumptions

### GPU Acceleration Optimism
The GPU rendering strategy assumes all users have capable discrete GPUs:

```rust
// "Use compute shader for GPU-accelerated binning"
let compute_pipeline = self.get_aggregation_pipeline()?;
```

**Reality**: Many Windows business laptops have Intel integrated graphics that struggle with compute shaders.

**Required**: CPU fallback for every GPU path:
```rust
match self.gpu_capability {
    GpuTier::Discrete => self.render_gpu_accelerated(),
    GpuTier::Integrated => self.render_cpu_with_gpu_presentation(),  
    GpuTier::None => self.render_pure_cpu(),
}
```

### 10M Point Interactivity Claims
The promise of "10M+ points interactively" glosses over fundamental limitations:

**Physics Problem**: Even with perfect LOD, user's mouse can only meaningfully interact with ~1000 points on screen. The rest is visual noise.

**Better Framing**: "Smoothly navigate datasets with 10M+ points through intelligent aggregation" - sets proper expectations.

## Architectural Inconsistencies

### Event System Bottleneck
The single event bus pattern will bottleneck under load:

```rust
// Everything flows through one channel
Event::Canvas(..) | Event::Compute(..) | Event::Storage(..)
```

**Issue**: A slow storage operation blocks UI updates.

**Fix**: Separate channels by priority:
```rust
struct EventRouter {
    ui_events: mpsc::Sender<UIEvent>,      // High priority
    compute_events: mpsc::Sender<ComputeEvent>,  // Medium priority
    storage_events: mpsc::Sender<StorageEvent>,  // Low priority
}
```

### Module Boundaries Violation
The proposed module layout has concerning dependencies:

- `pika-plot` depends on `pika-storage` (for Arrow data)
- `pika-canvas` depends on `pika-compute` (for node validation)
- `pika-compute` depends on `pika-storage` (for DuckDB access)

This creates a dependency cycle that will make testing painful.

**Better Layering**:
```
pika-core (types, traits)
    ↑
pika-data (Arrow, schemas)
    ↑
pika-compute | pika-storage (parallel)
    ↑
pika-plot | pika-canvas (parallel)
    ↑
pika-app
```

## Implementation Pitfalls

### Windows File Locking Hand-waving
The plan mentions "tokio::fs with explicit sharing modes" but doesn't address the real issue:

**Problem**: Windows locks memory-mapped files. DuckDB uses memory mapping extensively. Users can't delete/move their CSV files after import.

**Required**: Copy-on-import strategy:
```rust
let staging_dir = app_dirs::workspace_dir().join("staging");
fs::copy(&user_csv, &staging_dir.join("import_temp.csv"))?;
duckdb.import(&staging_path)?;
fs::remove_file(staging_path)?; // Clean up after import
```

### Snapshot Format Lock-in
RON + Parquet snapshots couple the persistence format to Rust's serialization:

**Future Problem**: RON format changes between serde versions break old snapshots.

**Better**: Version-aware format:
```rust
enum SnapshotFormat {
    V1 { metadata: JsonValue, data: Vec<ParquetFile> },
    V2 { /* future */ },
}
```

## Missing Critical Features

### Error Recovery
No mention of how to handle:
- Malformed CSV imports that crash DuckDB
- Queries that consume all memory
- GPU driver crashes (common on Windows)
- Network drives becoming unavailable

### Collaborative Barriers
The "offline-first" philosophy prevents basic collaboration:
- Can't share live sessions
- Can't co-edit queries
- Can't publish read-only views

Even local collaboration (via file sharing) is impossible with the proposed single-file snapshot format.

## If I Were Architecting From Scratch

I'd flip several fundamental decisions:

### 1. **Start with Notebooks, Add Canvas Later**
Users understand notebooks. Start there, add spatial features as an advanced mode:
```rust
enum WorkspaceView {
    Notebook { cells: Vec<Cell> },
    Hybrid { notebook: Vec<Cell>, canvas: CanvasState },
    Canvas { nodes: HashMap<NodeId, Node> },
}
```

### 2. **Browser-Based with Local Backend**
Instead of pure native:
- Rust backend serves localhost
- React/WebGL frontend
- Get web ecosystem benefits while keeping data local

### 3. **Streaming-First Architecture**
Replace the RecordBatch-everywhere approach with true streaming:
```rust
trait DataStream {
    async fn next(&mut self) -> Option<Result<Chunk>>;
}
```

### 4. **Progressive Enhancement**
Start with simple features that work everywhere:
- Basic CSV viewer that works on any hardware
- Add SQL when DuckDB available
- Add GPU acceleration when capable hardware detected
- Add sharing when user configures storage

This creates a graceful degradation path instead of the current all-or-nothing approach.

---

The current architecture shows impressive ambition but needs pragmatic constraints. Focus on building a rock-solid CSV viewer with basic plots first. The advanced features can come later once the foundation proves stable.