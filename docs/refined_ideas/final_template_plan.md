# 🚀 Pika-Plot: Master Architecture & Implementation Plan

## 📌 High-Level Introduction

**Pika-Plot** is a high-performance, offline-first desktop application that revolutionizes exploratory data analysis through an intuitive canvas-based interface. Unlike traditional notebooks that force linear workflows, or dashboards that require upfront design, Pika-Plot provides a fluid "data sketchpad" where insights emerge naturally through visual exploration.

What makes Pika-Plot different:
- **Canvas-first**: Infinite workspace where data flows visually between nodes
- **Instant feedback**: Semantic caching ensures zero lag during iterative exploration
- **Scale without limits**: Handles millions of rows interactively through adaptive sampling
- **Truly offline**: No cloud dependencies, no internet required, your data stays local
- **Professional exports**: High-quality PNG/SVG images and data exports for reports and presentations

Built in Rust for Windows 10/11, Pika-Plot combines the analytical power of DuckDB, the efficiency of Apache Arrow, GPU-accelerated rendering via wgpu, and the responsiveness of Egui to create a tool that makes complex data simple.

---

## 🗂️ Proposed Crate & Module Layout

### Cargo Workspace Structure

```toml
[workspace]
members = [
    "crates/pika-core",
    "crates/pika-canvas", 
    "crates/pika-compute",
    "crates/pika-storage",
    "crates/pika-plot",
    "crates/pika-ui",
    "crates/pika-app"
]
```

### Crate Descriptions

#### `pika-core` - Foundation Types & Contracts
- Core traits (`Node`, `DataSource`, `EventHandler`)
- Event definitions and message bus
- Common types (NodeId, Fingerprint, Schema)
- Error types and result aliases

#### `pika-canvas` - Visual Workspace Management
- Node graph data structures
- Spatial indexing for efficient rendering
- Layout algorithms (force-directed with stability)
- Connection validation and data flow

#### `pika-compute` - Query Processing & Caching
- SQL parsing and AST normalization
- Semantic fingerprinting engine
- Multi-tier cache with superset detection
- Background job orchestration

#### `pika-storage` - Data Persistence Layer
- DuckDB integration and connection pooling
- Arrow RecordBatch management
- CSV import with type inference
- Workspace snapshot serialization

#### `pika-plot` - GPU-Accelerated Visualization Engine
- GPU-accelerated rendering pipeline (wgpu-based)
- Adaptive sampling algorithms (LTTB, binning)
- Level-of-detail (LOD) system with GPU buffers
- Viewport-aware data streaming
- Plot type registry and renderers
- Inspired by Rerun but with enhanced caching

#### `pika-ui` - Reusable UI Components
- Table preview with virtual scrolling
- SQL editor with syntax highlighting
- Plot configuration panels
- Type override dialogs

#### `pika-app` - Application Shell
- Main window and event loop
- Theme management
- Settings persistence
- Cross-crate orchestration
- Export orchestration (HTML reports, PDFs)

---

## 🧠 Core Traits, Systems, and Interactions

### Foundation Traits

```rust
// pika-core/src/traits.rs

/// Base trait for all canvas nodes
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn name(&self) -> &str;
    fn position(&self) -> Point2;
    fn ports(&self) -> &Ports;
    
    /// Render the node's UI
    fn render(&mut self, ui: &mut egui::Ui, ctx: &CanvasContext);
    
    /// Handle incoming events
    fn handle_event(&mut self, event: NodeEvent) -> Result<Vec<AppEvent>>;
}

/// Nodes that produce data
pub trait DataSource: Node {
    fn schema(&self) -> &Schema;
    fn compute(&self, ctx: &ComputeContext) -> DataStream;
    fn fingerprint(&self) -> Fingerprint;
}

/// Streaming data iterator
pub trait DataStream: Send {
    fn schema(&self) -> &Schema;
    fn next_batch(&mut self) -> Option<Result<RecordBatch>>;
    fn estimated_rows(&self) -> Option<usize>;
}
```

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         UI Thread (egui)                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │ Canvas  │  │  Table  │  │  Query  │  │  Plot   │       │
│  │ Manager │  │  Node   │  │  Node   │  │  Node   │       │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘       │
│       │            │            │            │              │
│       └────────────┴────────────┴────────────┘              │
│                         │                                    │
│                    Event Bus                                 │
│                         │                                    │
└─────────────────────────┼────────────────────────────────────┘
                          │
                    ┌─────▼─────┐
                    │  Channel  │
                    └─────┬─────┘
                          │
┌─────────────────────────┼────────────────────────────────────┐
│                  Compute Thread Pool (tokio)                 │
│                         │                                    │
│  ┌──────────────────────▼──────────────────────┐           │
│  │          Message Router & Job Queue          │           │
│  └──────┬──────────┬──────────┬──────────┬────┘           │
│         │          │          │          │                 │
│    ┌────▼────┐ ┌──▼───┐ ┌───▼───┐ ┌────▼────┐           │
│    │ Query   │ │Cache │ │ Plot  │ │Snapshot│           │
│    │ Engine  │ │Store │ │Sampler│ │ Writer │           │
│    └────┬────┘ └──┬───┘ └───┬───┘ └────┬────┘           │
│         │          │          │          │                 │
│         └──────────┴──────────┴──────────┘                 │
│                         │                                    │
│                    ┌────▼────┐                             │
│                    │ DuckDB  │                             │
│                    │ Engine  │                             │
│                    └─────────┘                             │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow Example

```rust
// User connects TableNode to QueryNode
impl TableNode {
    fn on_connection(&mut self, to: NodeId, port: PortId) -> Result<Vec<AppEvent>> {
        Ok(vec![
            AppEvent::Compute(ComputeRequest::RegisterDataSource {
                node_id: self.id(),
                table_name: self.table_name.clone(),
            }),
            AppEvent::Canvas(CanvasEvent::ConnectionEstablished {
                from: self.id(),
                to,
            })
        ])
    }
}

// QueryNode receives table info and triggers computation
impl QueryNode {
    fn handle_event(&mut self, event: NodeEvent) -> Result<Vec<AppEvent>> {
        match event {
            NodeEvent::InputConnected { from, data_type } => {
                self.upstream_table = Some(from);
                Ok(vec![AppEvent::Compute(ComputeRequest::ExecuteQuery {
                    query_id: self.id(),
                    sql: self.sql.clone(),
                    upstream: from,
                })])
            }
            NodeEvent::DataReady { result } => {
                self.cached_result = Some(result);
                self.state = NodeState::Ready;
                Ok(vec![AppEvent::Canvas(CanvasEvent::NodeUpdated(self.id()))])
            }
            _ => Ok(vec![])
        }
    }
}
```

---

## 📊 Page Layouts & UI Mocks

### Main Canvas View
```
┌─────────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Help                                   │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐        │ ┌───────────────────┐ │
│ │ 📊  │ │ 🔍  │ │ 📈  │ │ 💾  │ Memory │ │ Node Inspector    │ │
│ │Table│ │Query│ │Plot │ │Snap │ 2.1GB  │ │                   │ │
│ └─────┘ └─────┘ └─────┘ └─────┘        │ │ Type: Query       │ │
├─────────────────────────────────────────┤ │ Name: Revenue Q4  │ │
│                                         │ │                   │ │
│  ┌─────────────┐      ┌─────────────┐  │ │ SQL:              │ │
│  │ sales.csv   │─────▶│ Revenue Q4  │  │ │ SELECT category,  │ │
│  │             │      │             │  │ │   SUM(amount)     │ │
│  │ 1.2M rows   │      │ Executing...│  │ │ FROM sales        │ │
│  │ 15 columns  │      │             │  │ │ WHERE quarter = 4 │ │
│  └─────────────┘      └─────────────┘  │ │ GROUP BY category │ │
│         │                               │ │                   │ │
│         │      ┌─────────────┐         │ │ [Run] [Save]      │ │
│         └─────▶│ Time Series │         │ └───────────────────┘ │
│                │   Plot      │         │                       │
│                │             │         │ ┌───────────────────┐ │
│                │ ░░░░░░░░░░░ │         │ │ Properties        │ │
│                └─────────────┘         │ │                   │ │
│                                        │ │ • Auto-refresh    │ │
│ ┌────────────────────────────────────┐ │ │ • Cache: 5.2MB    │ │
│ │ 🔍 Zoom: 100%  Pan: (0,0)  Grid: ● │ │ │ • Last run: 2s ago│ │
│ └────────────────────────────────────┘ │ └───────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Plot Node Detail View
```
┌─────────────────────────────────────────┐
│ Revenue by Category - Q4 2024           │
├─────────────────────────────────────────┤
│     ┌─────────────────────────┐         │
│  60K│      ████                │ Config  │
│     │      ████    ████        │ ────────│
│  40K│ ████ ████    ████ ████   │ X: cat  │
│     │ ████ ████ ████████████   │ Y: sum  │
│  20K│ ██████████████████████   │ Type: 📊│
│     └─────────────────────────┘         │
│      Tech Books Toys Food Home          │
├─────────────────────────────────────────┤
│ ○ Adaptive sampling (showing 1K of 50K) │
│ ○ Brush to filter connected nodes       │
│ [📷 Snapshot] [⚙️ Settings] [📤 Export] │
└─────────────────────────────────────────┘
```

### Type Override Dialog
```
┌─────────────────────────────────────────┐
│ Configure Column Types - sales.csv      │
├─────────────────────────────────────────┤
│ Column      Inferred    Override        │
│ ─────────────────────────────────────── │
│ id          Integer     [Integer    ▼] │
│ date        Text        [Date       ▼] │
│ amount      Float       [Float      ▼] │
│ category    Text        [Text       ▼] │
│ is_return   Integer     [Boolean    ▼] │
│                                         │
│ NULL values: [empty, null, N/A     ]   │
│                                         │
│ ☑ Remember these settings               │
│                                         │
│ [Preview with Changes] [Cancel] [Apply] │
└─────────────────────────────────────────┘
```

---

## 🚦 Message Passing / Async Flow

### Event-Driven Architecture

```rust
// pika-core/src/events.rs
pub enum AppEvent {
    Canvas(CanvasEvent),
    Compute(ComputeRequest),
    Storage(StorageEvent),
    UI(UIEvent),
}

pub enum ComputeRequest {
    ExecuteQuery { 
        query_id: NodeId, 
        sql: String,
        upstream: NodeId 
    },
    SamplePlotData { 
        plot_id: NodeId,
        source: RecordBatch,
        viewport: ViewportBounds,
        strategy: SampleStrategy,
    },
    SaveSnapshot {
        workspace: WorkspaceState,
        path: PathBuf,
    },
}

pub enum ComputeResponse {
    QueryComplete {
        query_id: NodeId,
        result: Result<Arc<RecordBatch>>,
        fingerprint: Fingerprint,
    },
    PlotDataReady {
        plot_id: NodeId,
        data: PlotData,
    },
    SnapshotSaved {
        path: PathBuf,
        size_bytes: u64,
    },
}
```

### Async Task Orchestration

```rust
// pika-app/src/compute_runtime.rs
pub struct ComputeRuntime {
    request_rx: mpsc::Receiver<ComputeRequest>,
    response_tx: mpsc::Sender<ComputeResponse>,
    thread_pool: Arc<Runtime>,
    active_jobs: DashMap<NodeId, JoinHandle<()>>,
}

impl ComputeRuntime {
    pub fn spawn(ui_tx: mpsc::Sender<ComputeResponse>) -> mpsc::Sender<ComputeRequest> {
        let (tx, rx) = mpsc::channel(1000);
        
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .unwrap();
                
            runtime.block_on(async {
                let mut compute = ComputeRuntime {
                    request_rx: rx,
                    response_tx: ui_tx,
                    thread_pool: Arc::new(runtime),
                    active_jobs: DashMap::new(),
                };
                
                compute.run().await;
            });
        });
        
        tx
    }
    
    async fn run(&mut self) {
        while let Some(request) = self.request_rx.recv().await {
            match request {
                ComputeRequest::ExecuteQuery { query_id, sql, upstream } => {
                    // Cancel any existing job for this node
                    if let Some((_, handle)) = self.active_jobs.remove(&query_id) {
                        handle.abort();
                    }
                    
                    let response_tx = self.response_tx.clone();
                    let handle = tokio::spawn(async move {
                        let result = execute_query_with_cache(sql, upstream).await;
                        let _ = response_tx.send(ComputeResponse::QueryComplete {
                            query_id,
                            result,
                            fingerprint,
                        }).await;
                    });
                    
                    self.active_jobs.insert(query_id, handle);
                }
                // ... handle other request types
            }
        }
    }
}
```

---

## 🧩 Advanced Caching System

### Multi-Level Cache Architecture

Inspired by Rerun's impressive 20-30x performance gains but enhanced for our specific use case, we implement a more sophisticated caching system that addresses their limitations:

```rust
// pika-compute/src/cache.rs

pub struct HierarchicalCache {
    // L1: Primary Query Cache - Exact matches with versioning
    primary_cache: Arc<DashMap<PrimaryKey, CachedEntry>>,
    
    // L2: Derived Data Cache - Computation results, filtered views
    derived_cache: Arc<DashMap<DerivedKey, Arc<RecordBatch>>>,
    
    // L3: Spatial Cache - Plot data at different resolutions
    spatial_cache: Arc<DashMap<SpatialKey, LodData>>,
    
    // L4: GPU Buffer Cache - Ready-to-render vertex buffers
    gpu_cache: Arc<RwLock<GpuBufferCache>>,
    
    // Cache coordinator
    coordinator: CacheCoordinator,
}

#[derive(Hash, Eq, PartialEq)]
pub struct PrimaryKey {
    query_hash: u64,           // Normalized query hash
    schema_version: u64,       // Table schema version
    time_range: Option<(i64, i64)>, // Optional time bounds
}

#[derive(Hash, Eq, PartialEq)]
pub struct DerivedKey {
    parent_key: PrimaryKey,    // Source query
    operation: TransformOp,    // Filter, aggregate, sample, etc.
    parameters: Vec<u8>,       // Serialized parameters
}

#[derive(Hash, Eq, PartialEq)]
pub struct SpatialKey {
    data_key: DerivedKey,      // Source data
    viewport: ViewportBounds,  // Current view
    resolution: u32,           // Target resolution
}

impl HierarchicalCache {
    pub async fn query_with_lineage<F>(
        &self,
        request: QueryRequest,
        compute_fn: F,
    ) -> Result<QueryResult>
    where
        F: FnOnce() -> BoxFuture<'static, Result<RecordBatch>>,
    {
        // 1. Build cache key with full context
        let primary_key = self.build_primary_key(&request)?;
        
        // 2. Check if we have exact match
        if let Some(entry) = self.primary_cache.get(&primary_key) {
            self.coordinator.record_hit(CacheLevel::Primary, &primary_key);
            return Ok(QueryResult::Cached(entry.clone()));
        }
        
        // 3. Check if we can derive from existing data
        if let Some(derived) = self.find_derivable(&request).await {
            let result = self.derive_from_cached(derived, &request).await?;
            self.coordinator.record_hit(CacheLevel::Derived, &primary_key);
            return Ok(QueryResult::Derived(result));
        }
        
        // 4. Check for temporal locality - can we extend existing time series?
        if let Some(extended) = self.try_temporal_extension(&request).await {
            self.coordinator.record_hit(CacheLevel::Temporal, &primary_key);
            return Ok(QueryResult::Extended(extended));
        }
        
        // 5. Compute fresh and update all cache levels
        let result = Arc::new(compute_fn().await?);
        self.update_all_levels(primary_key, result.clone()).await;
        
        Ok(QueryResult::Fresh(result))
    }
    
    async fn find_derivable(&self, request: &QueryRequest) -> Option<DerivableEntry> {
        // Advanced matching: subset queries, temporal overlaps, spatial contains
        self.coordinator.find_best_match(request, &self.primary_cache)
    }
}

/// Cache coordinator implements smart eviction and prefetching
pub struct CacheCoordinator {
    access_patterns: Arc<RwLock<AccessPatternTracker>>,
    memory_monitor: MemoryMonitor,
    prefetch_engine: PrefetchEngine,
}

impl CacheCoordinator {
    /// Predictive prefetching based on access patterns
    pub async fn prefetch_related(&self, accessed_key: &PrimaryKey) {
        let patterns = self.access_patterns.read().await;
        
        // Identify likely next accesses
        let predictions = patterns.predict_next(accessed_key);
        
        for prediction in predictions {
            if prediction.confidence > 0.7 {
                self.prefetch_engine.schedule(prediction.key);
            }
        }
    }
    
    /// Smart eviction considering relationships
    pub async fn evict_smart(&self, target_bytes: usize) -> usize {
        let mut evicted = 0;
        
        // Build dependency graph
        let graph = self.build_cache_dependency_graph().await;
        
        // Evict leaves first, preserving shared base data
        for entry in graph.leaves_by_weight() {
            if evicted >= target_bytes {
                break;
            }
            
            // Only evict if not recently used
            if entry.last_access < Instant::now() - Duration::from_secs(300) {
                evicted += entry.size_bytes;
                self.remove_with_dependents(entry).await;
            }
        }
        
        evicted
    }
}

/// Key improvements over Rerun's approach:
/// 
/// 1. **Simpler data model**: We focus on tabular data, avoiding multimodal complexity
/// 2. **GPU buffer caching**: Direct GPU memory management for instant rendering
/// 3. **Predictive prefetching**: Learn user patterns and preload likely queries
/// 4. **Spatial awareness**: Cache plot data at multiple zoom levels
/// 5. **Zero-copy everywhere**: Arrow batches shared via Arc throughout
/// 6. **Query normalization**: Better semantic matching than simple text comparison
/// 
/// Unlike Rerun's bit sets for filtering, we use Arrow's native predicate pushdown
/// for more efficient filtering at the storage layer.
```

### AST Normalization Rules

```rust
// pika-compute/src/normalizer.rs

impl AstNormalizer {
    pub fn normalize(&self, sql: &str) -> Result<NormalizedQuery> {
        let ast = Parser::parse_sql(&MySqlDialect {}, sql)?;
        
        let mut normalized = NormalizedQuery::default();
        
        for statement in ast {
            match statement {
                Statement::Query(query) => {
                    self.normalize_query(*query, &mut normalized)?;
                }
                _ => return Err(Error::UnsupportedStatement),
            }
        }
        
        Ok(normalized)
    }
    
    fn normalize_query(&self, query: Query, out: &mut NormalizedQuery) -> Result<()> {
        // 1. Extract and sort table references
        // 2. Normalize column references (qualify with table)
        // 3. Canonicalize predicates (x>5 == x > 5.0)
        // 4. Sort ORDER BY and GROUP BY clauses
        // 5. Remove cosmetic differences (whitespace, casing)
        
        // This ensures:
        // - "SELECT * FROM t WHERE x>5" 
        // - "select * from T where X > 5.0"
        // Both produce identical fingerprints
        
        todo!("Full implementation")
    }
}
```

---

## 📦 CSV Ingestion Strategy

### Intelligent Import Pipeline

```rust
// pika-storage/src/csv_import.rs

pub struct CsvImporter {
    duckdb: Arc<Connection>,
    type_detector: TypeDetector,
}

impl CsvImporter {
    pub async fn import_csv(&self, path: &Path, options: ImportOptions) -> Result<TableInfo> {
        // 1. Sample first 1000 rows for type inference
        let sample = self.read_sample(path, 1000)?;
        
        // 2. Detect types with confidence scores
        let detected_schema = self.type_detector.infer_schema(&sample)?;
        
        // 3. Show preview dialog if low confidence or user requested
        let final_schema = if options.show_preview || detected_schema.has_low_confidence() {
            self.show_type_override_dialog(detected_schema).await?
        } else {
            detected_schema
        };
        
        // 4. Create DuckDB table with final schema
        let table_name = self.create_table(&final_schema)?;
        
        // 5. Stream data using DuckDB's optimized CSV reader
        let import_sql = format!(
            "INSERT INTO {} SELECT * FROM read_csv('{}', 
                columns = {}, 
                nullstr = '{}'
            )",
            table_name,
            path.display(),
            final_schema.to_duckdb_columns(),
            options.null_values.join(",")
        );
        
        self.duckdb.execute(&import_sql, [])?;
        
        // 6. Create indexes for likely filter columns
        self.create_smart_indexes(&table_name, &final_schema)?;
        
        Ok(TableInfo {
            name: table_name,
            schema: final_schema,
            row_count: self.get_row_count(&table_name)?,
            size_bytes: self.get_table_size(&table_name)?,
        })
    }
}

pub struct TypeDetector {
    patterns: HashMap<DataType, Vec<Regex>>,
}

impl TypeDetector {
    pub fn infer_schema(&self, sample: &RecordBatch) -> Result<Schema> {
        let mut fields = vec![];
        
        for (col_idx, column) in sample.columns().iter().enumerate() {
            let field_name = sample.schema().field(col_idx).name();
            let inferred_type = self.infer_column_type(column)?;
            
            fields.push(Field::new(field_name, inferred_type.arrow_type(), true));
        }
        
        Ok(Schema::new(fields))
    }
    
    fn infer_column_type(&self, column: &ArrayRef) -> Result<InferredType> {
        // Smart inference based on:
        // 1. Value patterns (dates, booleans, etc)
        // 2. Numeric range and precision
        // 3. Uniqueness ratio (potential categories)
        // 4. NULL distribution
        
        todo!("Full implementation")
    }
}
```

---

## 💾 Persistence + Snapshot Format

### Workspace Snapshot Architecture

Snapshots are designed for saving and resuming your analytical work, not for real-time collaboration. They create a complete, self-contained archive of your workspace state.

```rust
// pika-storage/src/snapshot.rs

#[derive(Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    version: u32,
    metadata: WorkspaceMetadata,
    canvas_state: CanvasState,
    data_chunks: Vec<DataChunkRef>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    created_at: DateTime<Utc>,
    pika_version: String,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CanvasState {
    nodes: Vec<SerializedNode>,
    connections: Vec<Connection>,
    viewport: Viewport,
    annotations: Vec<Annotation>,
}

#[derive(Serialize, Deserialize)]
pub struct DataChunkRef {
    id: Uuid,
    fingerprint: Fingerprint,
    format: DataFormat,
    compression: CompressionType,
    size_bytes: u64,
}

pub struct SnapshotWriter {
    base_path: PathBuf,
}

impl SnapshotWriter {
    pub async fn save_workspace(&self, workspace: &Workspace) -> Result<PathBuf> {
        let snapshot_dir = self.create_snapshot_directory()?;
        
        // 1. Serialize metadata and canvas state to RON
        let metadata = WorkspaceSnapshot {
            version: SNAPSHOT_VERSION,
            metadata: workspace.metadata(),
            canvas_state: workspace.canvas_state(),
            data_chunks: vec![],
        };
        
        let ron_path = snapshot_dir.join("workspace.ron");
        let mut ron_file = File::create(&ron_path)?;
        ron::ser::to_writer_pretty(&mut ron_file, &metadata, Default::default())?;
        
        // 2. Save data chunks as compressed Parquet files
        let mut chunk_refs = vec![];
        for node in workspace.data_nodes() {
            if let Some(data) = node.cached_data() {
                let chunk_ref = self.save_data_chunk(&snapshot_dir, node.id(), data).await?;
                chunk_refs.push(chunk_ref);
            }
        }
        
        // 3. Update metadata with chunk references
        let mut metadata = metadata;
        metadata.data_chunks = chunk_refs;
        ron::ser::to_writer_pretty(&mut ron_file, &metadata, Default::default())?;
        
        // 4. Create single .pikaplot file (zip archive)
        let archive_path = self.create_archive(&snapshot_dir)?;
        
        Ok(archive_path)
    }
    
    async fn save_data_chunk(
        &self,
        dir: &Path,
        node_id: NodeId,
        data: Arc<RecordBatch>,
    ) -> Result<DataChunkRef> {
        let chunk_path = dir.join(format!("{}.parquet", node_id));
        
        // Use Arrow's Parquet writer with compression
        let file = File::create(&chunk_path)?;
        let mut writer = ArrowWriter::try_new(
            file,
            data.schema(),
            Some(WriterProperties::builder()
                .set_compression(Compression::ZSTD)
                .build()),
        )?;
        
        writer.write(&data)?;
        writer.close()?;
        
        Ok(DataChunkRef {
            id: node_id.into(),
            fingerprint: compute_data_fingerprint(&data),
            format: DataFormat::Parquet,
            compression: CompressionType::Zstd,
            size_bytes: chunk_path.metadata()?.len(),
        })
    }
}
```

---

## 📤 Export Capabilities

### Image Export System

The primary export mechanism focuses on high-quality static images suitable for inclusion in reports and presentations.

```rust
// pika-app/src/export/image_export.rs

pub struct ImageExporter {
    gpu_renderer: Arc<GpuPlotRenderer>,
    svg_renderer: SvgRenderer,
}

impl ImageExporter {
    pub async fn export_plot_as_png(
        &self,
        plot_node: &PlotNode,
        options: PngExportOptions,
    ) -> Result<Vec<u8>> {
        // Render plot at specified resolution using GPU
        let render_target = self.gpu_renderer.create_render_target(
            options.width,
            options.height,
            options.dpi,
        )?;
        
        // Render with export-specific settings (e.g., white background, larger fonts)
        let export_config = plot_node.config.clone_for_export();
        self.gpu_renderer.render_to_target(
            &plot_node.cached_data,
            &export_config,
            &render_target,
        ).await?;
        
        // Extract PNG bytes
        let png_data = render_target.to_png()?;
        Ok(png_data)
    }
    
    pub fn export_plot_as_svg(
        &self,
        plot_node: &PlotNode,
        options: SvgExportOptions,
    ) -> Result<String> {
        // For vector export, we might use a different rendering path
        // that produces cleaner lines and text
        self.svg_renderer.render(
            &plot_node.cached_data,
            &plot_node.config,
            options,
        )
    }
    
    pub async fn export_visible_workspace(
        &self,
        workspace: &Workspace,
        options: WorkspaceExportOptions,
    ) -> Result<ExportBundle> {
        let mut bundle = ExportBundle::new();
        
        // Export each visible plot
        for node in workspace.get_visible_nodes() {
            if let Some(plot_node) = node.as_plot() {
                let png = self.export_plot_as_png(plot_node, options.png_options).await?;
                bundle.add_plot(plot_node.id(), plot_node.name(), png);
            }
        }
        
        // Export data tables as CSV
        for node in workspace.get_visible_nodes() {
            if let Some(data) = node.get_table_data() {
                let csv = export_to_csv(data)?;
                bundle.add_data(node.id(), node.name(), csv);
            }
        }
        
        // Create a simple markdown summary with embedded images
        let summary = self.create_summary_document(&bundle, workspace)?;
        bundle.set_summary(summary);
        
        Ok(bundle)
    }
}
```

### Export Dialog UI

```rust
// pika-ui/src/dialogs/export.rs

pub struct ExportDialog {
    export_scope: ExportScope,
    export_format: ExportFormat,
    export_options: ExportOptions,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ExportScope {
    CurrentPlot,      // Export only the selected plot
    VisibleCanvas,    // Export all visible plots
    FullWorkspace,    // Export everything
}

#[derive(Clone, Copy, PartialEq)]
pub enum ExportFormat {
    PngImage,         // High-res PNG (default)
    SvgVector,        // Scalable vector graphics
    PdfDocument,      // PDF with multiple plots
    CsvData,          // Raw data export
}

pub struct ExportOptions {
    // PNG specific
    pub png_resolution: Resolution,
    pub png_dpi: u32,
    pub transparent_background: bool,
    
    // General options
    pub include_annotations: bool,
    pub include_title: bool,
    pub include_timestamp: bool,
}

impl ExportDialog {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<ExportRequest> {
        let mut result = None;
        
        egui::Window::new("Export")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("What to Export");
                
                ui.radio_value(&mut self.export_scope, ExportScope::CurrentPlot, "Current Plot");
                ui.radio_value(&mut self.export_scope, ExportScope::VisibleCanvas, "Visible Canvas Area");
                ui.radio_value(&mut self.export_scope, ExportScope::FullWorkspace, "Full Workspace");
                
                ui.separator();
                ui.heading("Format");
                
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.export_format, ExportFormat::PngImage, "PNG");
                    ui.selectable_value(&mut self.export_format, ExportFormat::SvgVector, "SVG");
                    ui.selectable_value(&mut self.export_format, ExportFormat::PdfDocument, "PDF");
                    ui.selectable_value(&mut self.export_format, ExportFormat::CsvData, "CSV");
                });
                
                ui.separator();
                
                match self.export_format {
                    ExportFormat::PngImage => {
                        ui.heading("PNG Options");
                        
                        ui.horizontal(|ui| {
                            ui.label("Resolution:");
                            ui.selectable_value(&mut self.export_options.png_resolution, 
                                Resolution::HD, "1920×1080");
                            ui.selectable_value(&mut self.export_options.png_resolution, 
                                Resolution::UHD, "3840×2160");
                            ui.selectable_value(&mut self.export_options.png_resolution, 
                                Resolution::Custom, "Custom");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("DPI:");
                            ui.selectable_value(&mut self.export_options.png_dpi, 72, "72 (Screen)");
                            ui.selectable_value(&mut self.export_options.png_dpi, 150, "150 (Print)");
                            ui.selectable_value(&mut self.export_options.png_dpi, 300, "300 (High Quality)");
                        });
                        
                        ui.checkbox(&mut self.export_options.transparent_background, 
                            "Transparent Background");
                    }
                    ExportFormat::SvgVector => {
                        ui.heading("SVG Options");
                        ui.label("Vector graphics suitable for scaling and editing");
                    }
                    _ => {}
                }
                
                ui.separator();
                
                ui.checkbox(&mut self.export_options.include_title, "Include Plot Title");
                ui.checkbox(&mut self.export_options.include_annotations, "Include Annotations");
                ui.checkbox(&mut self.export_options.include_timestamp, "Include Export Timestamp");
                
                ui.separator();
                
                if ui.button("Export").clicked() {
                    result = Some(ExportRequest {
                        scope: self.export_scope,
                        format: self.export_format,
                        options: self.export_options.clone(),
                    });
                }
            });
        
        result
    }
}
```

---

## 📐 Performance Guidelines

### GPU-Accelerated Plot Rendering

```rust
// pika-plot/src/gpu_renderer.rs

pub struct GpuPlotRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline_cache: HashMap<PlotType, RenderPipeline>,
    buffer_allocator: BufferAllocator,
    texture_atlas: TextureAtlas,
}

impl GpuPlotRenderer {
    pub async fn render_plot(
        &self,
        plot_data: &PlotData,
        config: &PlotConfig,
        target: &RenderTarget,
    ) -> Result<()> {
        let pipeline = self.get_or_create_pipeline(config.plot_type)?;
        
        // Prepare GPU buffers based on data size
        let buffers = match plot_data.point_count() {
            0..=100_000 => {
                // Direct vertex buffer for small datasets
                self.buffer_allocator.create_vertex_buffer(plot_data)?
            }
            100_001..=10_000_000 => {
                // Instanced rendering for medium datasets
                self.prepare_instanced_buffers(plot_data)?
            }
            _ => {
                // Compute shader aggregation for massive datasets
                self.prepare_aggregation_buffers(plot_data).await?
            }
        };
        
        // GPU command encoding
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Plot Render Pass"),
                color_attachments: &[Some(target.color_attachment())],
                depth_stencil_attachment: target.depth_attachment(),
            });
            
            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, &config.bind_group, &[]);
            
            match &buffers {
                PlotBuffers::Direct { vertices, .. } => {
                    render_pass.set_vertex_buffer(0, vertices.slice(..));
                    render_pass.draw(0..plot_data.point_count() as u32, 0..1);
                }
                PlotBuffers::Instanced { instances, .. } => {
                    render_pass.set_vertex_buffer(0, instances.slice(..));
                    render_pass.draw(0..4, 0..plot_data.point_count() as u32);
                }
                PlotBuffers::Aggregated { texture, quad } => {
                    // Render aggregated heatmap/density
                    render_pass.set_vertex_buffer(0, quad.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
    
    async fn prepare_aggregation_buffers(&self, data: &PlotData) -> Result<PlotBuffers> {
        // Use compute shader for GPU-accelerated binning
        let compute_pipeline = self.get_aggregation_pipeline()?;
        let output_texture = self.create_density_texture()?;
        
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &data.compute_bind_group, &[]);
            
            let workgroups = (data.point_count() as u32 + 255) / 256;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        
        Ok(PlotBuffers::Aggregated {
            texture: output_texture,
            quad: self.create_fullscreen_quad(),
        })
    }
}

/// Adaptive LOD system for smooth zooming
pub struct LodManager {
    levels: Vec<LodLevel>,
    gpu_renderer: Arc<GpuPlotRenderer>,
}

impl LodManager {
    pub async fn prepare_lod_pyramid(&mut self, data: Arc<RecordBatch>) -> Result<()> {
        // Build multiple resolution levels
        self.levels.clear();
        
        let base_points = data.num_rows();
        let mut current_data = data;
        let mut resolution = 1.0;
        
        while current_data.num_rows() > 1000 {
            let level = LodLevel {
                resolution,
                gpu_buffers: self.gpu_renderer.prepare_buffers(&current_data).await?,
                bounds: calculate_bounds(&current_data),
            };
            
            self.levels.push(level);
            
            // Downsample for next level
            current_data = Arc::new(downsample_lttb(&current_data, current_data.num_rows() / 4)?);
            resolution *= 0.5;
        }
        
        Ok(())
    }
    
    pub fn select_lod(&self, viewport: &ViewportBounds) -> &LodLevel {
        // Select appropriate level based on zoom
        let viewport_density = viewport.data_points_per_pixel();
        
        self.levels.iter()
            .find(|level| level.resolution <= viewport_density)
            .unwrap_or(self.levels.last().unwrap())
    }
}

// Memory efficiency rules:
// 1. Use Arc<RecordBatch> everywhere - zero-copy sharing
// 2. Stream large results - never load full dataset
// 3. Downsample before sending to GPU
// 4. Cache aggressively but with bounds
```

### Memory Management

```rust
// pika-app/src/memory_monitor.rs

pub struct MemoryMonitor {
    warning_threshold: f64, // e.g., 0.8 = 80% of available RAM
    critical_threshold: f64, // e.g., 0.95 = 95%
}

impl MemoryMonitor {
    pub fn start(ui_tx: mpsc::Sender<MemoryEvent>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut monitor = MemoryMonitor {
                warning_threshold: 0.8,
                critical_threshold: 0.95,
            };
            
            loop {
                let stats = monitor.get_memory_stats();
                
                if stats.usage_ratio > monitor.critical_threshold {
                    // Force cache eviction
                    let _ = ui_tx.send(MemoryEvent::Critical {
                        used_gb: stats.used_bytes as f64 / 1e9,
                        available_gb: stats.available_bytes as f64 / 1e9,
                    }).await;
                } else if stats.usage_ratio > monitor.warning_threshold {
                    // Show warning
                    let _ = ui_tx.send(MemoryEvent::Warning {
                        used_gb: stats.used_bytes as f64 / 1e9,
                    }).await;
                }
                
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        })
    }
}
```

---

## 🧪 Testing & Documentation Plan

### Comprehensive Testing Strategy

Our testing approach goes beyond correctness to include performance optimization and parameter tuning:

#### 1. Correctness Tests (40% of tests)
```rust
// Unit tests for core functionality
#[cfg(test)]
mod cache_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn normalized_queries_have_same_fingerprint(
            spaces in 0..5,
            case_variant in 0..3,
        ) {
            let query1 = "SELECT * FROM table WHERE x > 5";
            let query2 = vary_query(query1, spaces, case_variant);
            
            let fp1 = fingerprint(query1)?;
            let fp2 = fingerprint(query2)?;
            
            prop_assert_eq!(fp1, fp2);
        }
    }
}
```

#### 2. Performance Optimization Tests (30% of tests)
```rust
// tests/performance/cache_tuning.rs
use criterion::{black_box, criterion_group, Criterion, BenchmarkId};

/// Test different cache configurations to find optimal parameters
fn benchmark_cache_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    // Test different cache sizes
    for cache_size_mb in [100, 500, 1000, 2000] {
        // Test different eviction strategies
        for strategy in ["lru", "lfu", "smart_dependency"] {
            group.bench_with_input(
                BenchmarkId::new(strategy, cache_size_mb),
                &(cache_size_mb, strategy),
                |b, &(size, strat)| {
                    let cache = build_cache(size, strat);
                    let workload = generate_realistic_workload();
                    
                    b.iter(|| {
                        simulate_workload(&cache, &workload)
                    });
                },
            );
        }
    }
    
    // Find optimal parameters
    group.finish();
    
    // Generate tuning report
    println!("Optimal cache size: {} MB", find_knee_in_curve());
    println!("Best eviction strategy: {}", best_strategy());
}

/// GPU rendering performance tests
fn benchmark_gpu_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_rendering");
    
    // Test different batch sizes for GPU uploads
    for batch_size in [1_000, 10_000, 50_000, 100_000, 500_000] {
        group.bench_with_input(
            BenchmarkId::new("vertex_upload", batch_size),
            &batch_size,
            |b, &size| {
                let renderer = GpuRenderer::new();
                let data = generate_points(size);
                
                b.iter(|| {
                    renderer.upload_vertices(&data)
                });
            },
        );
    }
    
    // Test different aggregation strategies
    for points in [1_000_000, 10_000_000, 50_000_000] {
        for strategy in ["compute_shader", "cpu_binning", "hierarchical"] {
            group.bench_with_input(
                BenchmarkId::new(strategy, points),
                &(points, strategy),
                |b, &(pts, strat)| {
                    let data = generate_dense_scatter(pts);
                    b.iter(|| {
                        aggregate_for_viewport(&data, strat, ViewportBounds::default())
                    });
                },
            );
        }
    }
}
```

#### 3. Parameter Tuning Tests (20% of tests)
```rust
// tests/tuning/adaptive_parameters.rs

/// Automatically tune LOD thresholds based on hardware
#[test]
fn tune_lod_thresholds() {
    let gpu_info = query_gpu_capabilities();
    let cpu_cores = num_cpus::get();
    
    let mut best_thresholds = LodThresholds::default();
    let mut best_fps = 0.0;
    
    // Grid search for optimal thresholds
    for direct_render_limit in [10_000, 50_000, 100_000, 200_000] {
        for instanced_limit in [500_000, 1_000_000, 5_000_000] {
            for compute_threshold in [5_000_000, 10_000_000, 20_000_000] {
                let thresholds = LodThresholds {
                    direct_render_limit,
                    instanced_limit,
                    compute_threshold,
                };
                
                let fps = benchmark_with_thresholds(&thresholds);
                
                if fps > best_fps {
                    best_fps = fps;
                    best_thresholds = thresholds;
                }
            }
        }
    }
    
    // Write tuned parameters to config
    write_tuned_config(&best_thresholds);
}

/// Test cache warming strategies
#[test]
fn optimize_prefetch_patterns() {
    let traces = load_user_interaction_traces();
    
    // Test different prefetch strategies
    let strategies = vec![
        PrefetchStrategy::Temporal(0.5),    // 50% lookahead
        PrefetchStrategy::Temporal(1.0),    // 100% lookahead
        PrefetchStrategy::Spatial(2),       // 2 neighbor prefetch
        PrefetchStrategy::Predictive(0.7),  // 70% confidence threshold
    ];
    
    for strategy in strategies {
        let hit_rate = simulate_with_strategy(&traces, strategy);
        println!("{:?}: {:.2}% hit rate", strategy, hit_rate * 100.0);
    }
}
```

#### 4. Integration & Stress Tests (10% of tests)
```rust
// tests/stress/large_data.rs

#[tokio::test]
async fn stress_test_100m_points() {
    let app = TestApp::new().await;
    
    // Generate massive dataset
    let csv_path = generate_large_csv(100_000_000);
    
    // Measure import performance
    let start = Instant::now();
    let table_id = app.import_csv(&csv_path).await?;
    println!("Import 100M rows: {:?}", start.elapsed());
    
    // Test various operations at scale
    let operations = vec![
        "SELECT COUNT(*) FROM data",
        "SELECT x, y FROM data WHERE z > 0.5",
        "SELECT category, AVG(value) FROM data GROUP BY category",
    ];
    
    for op in operations {
        let start = Instant::now();
        let result = app.execute_query(op).await?;
        println!("{}: {:?} ({} rows)", op, start.elapsed(), result.num_rows());
        
        // Verify cache behavior
        let start = Instant::now();
        let cached = app.execute_query(op).await?;
        assert!(start.elapsed() < Duration::from_millis(50));
    }
}

/// Test memory pressure handling
#[test]
fn test_memory_pressure_response() {
    let monitor = MemoryMonitor::new();
    let cache = HierarchicalCache::new();
    
    // Gradually increase memory usage
    loop {
        let data = generate_large_dataset(10_000_000);
        cache.insert(random_key(), data);
        
        let usage = monitor.current_usage();
        if usage.ratio > 0.8 {
            // Verify smart eviction kicked in
            assert!(cache.size_bytes() < usage.total * 0.7);
            break;
        }
    }
}
```

#### Performance Testing Infrastructure
```rust
// tests/harness/performance.rs

/// Record performance metrics for regression detection
pub struct PerformanceBaseline {
    metrics: HashMap<String, PerformanceMetric>,
}

impl PerformanceBaseline {
    pub fn compare_with_current(&self) -> TestResult {
        let current = self.measure_current();
        
        for (name, baseline) in &self.metrics {
            let current_value = current.get(name)?;
            
            // Fail if performance regressed by more than 10%
            if current_value > baseline.value * 1.1 {
                return TestResult::Regression {
                    metric: name.clone(),
                    baseline: baseline.value,
                    current: current_value,
                };
            }
        }
        
        TestResult::Pass
    }
}
```

### Documentation Plan

#### Architecture Documentation
```markdown
docs/
├── architecture/
│   ├── README.md          # This document
│   ├── data-flow.md       # Detailed data flow diagrams
│   ├── caching.md         # Cache design and strategies
│   └── scaling.md         # Performance at scale
├── api/                   # Generated from doc comments
├── guides/
│   ├── getting-started.md
│   ├── creating-nodes.md
│   └── extending-plots.md
└── examples/
    ├── basic-workflow/
    ├── custom-node/
    └── advanced-queries/
```

#### Code Documentation Standards
```rust
/// A node that executes SQL queries against connected data sources.
/// 
/// # Examples
/// 
/// ```
/// let query = QueryNode::new("SELECT * FROM sales WHERE amount > 100");
/// query.connect_input(table_node.output_port());
/// 
/// let result = query.execute().await?;
/// assert!(result.num_rows() > 0);
/// ```
/// 
/// # Performance
/// 
/// Query execution is cached based on semantic fingerprinting.
/// Identical queries will return instantly from cache.
pub struct QueryNode {
    // ...
}
```

---

## 📣 Opinions & Challenges

### Architectural Decisions Made

1. **DuckDB-only backend**: After extensive analysis, the hybrid SQLite approach adds unnecessary complexity. DuckDB handles metadata, analytics, and scale elegantly.

2. **Event-driven with channels**: Superior to direct coupling. Makes testing trivial and enables clean async boundaries.

3. **Unified cache with tiers**: Best of both worlds - simple mental model with logical separation via enum variants.

4. **RON + Parquet snapshots**: Practical and portable. Avoids overengineering while ensuring cross-platform compatibility.

### Challenges Addressed

1. **Egui immediate mode paradox**: Solved by using retained state for canvas positions while keeping UI immediate mode. Custom `Area` wrapper handles the transform math.

2. **Windows file locking**: Mitigated by using `tokio::fs` with explicit sharing modes and keeping file handles open minimally.

3. **10M+ row interactivity**: Achieved through adaptive LOD system. Never send more than 10K points to renderer.

4. **Memory pressure**: DuckDB spilling + proactive monitoring + bounded caches = stable performance.

### Where We Outperform Rerun

Our focused approach on tabular data and canvas-based exploration allows several key advantages:

1. **Cache Efficiency**: 
   - Rerun handles arbitrary multimodal data, requiring complex serialization
   - We optimize for Arrow columnar format throughout, enabling zero-copy everywhere
   - Our semantic query understanding is deeper than Rerun's text matching

2. **GPU Integration**:
   - Rerun's primary focus is CPU rendering with optional GPU
   - We design GPU-first, with compute shaders for aggregation
   - Direct GPU buffer caching eliminates CPU-GPU transfer bottlenecks

3. **Query Optimization**:
   - Rerun executes queries as-is
   - We leverage DuckDB's query planner and our semantic cache for optimization
   - Predictive prefetching based on canvas navigation patterns

4. **Memory Management**:
   - Rerun uses generic bit sets for filtering
   - We use Arrow's native predicate pushdown and column pruning
   - Hierarchical cache with dependency tracking prevents redundant storage

5. **Interactivity**:
   - Rerun focuses on playback and debugging
   - We optimize for exploration with instant viewport changes
   - Canvas-based workflow enables spatial locality optimizations

### Novel Improvements

1. **Superset cache detection**: Beyond simple fingerprinting, we detect when cached queries can answer more specific queries through filtering.

2. **Progressive plot rendering**: Start with low-res overview, stream in detail. Cancel/restart on viewport change.

3. **Stable canvas layout**: Force-directed layout that respects user adjustments. New nodes don't disrupt existing mental map.

4. **Type inference with confidence**: Show override dialog only when inference confidence is low, reducing friction.

### Export & Reporting Features

Since Pika-Plot is a standalone offline application, we focus on high-quality image exports and data extraction:

1. **Primary Export Formats**:
   - **PNG**: High-resolution raster images with configurable DPI for presentations and reports
   - **SVG**: Scalable vector graphics for publication-quality figures
   - **PDF**: Multi-page documents combining multiple plots
   - **CSV**: Raw data export for further analysis

2. **Export Capabilities**:
   - GPU-accelerated rendering ensures exports match on-screen quality
   - Export individual plots, visible canvas area, or entire workspace
   - Configurable resolution, DPI, and background options
   - Batch export for multiple plots

3. **Future Consideration**: HTML reports with embedded interactive plots may be added later, but are not a priority given the large data volumes and canvas-based interface

### Leveraging Existing Implementations & Open Source

Since you have existing tools with plot implementations and features inspired by Rerun, we can accelerate development by:

#### From Your Existing Tools:
1. **Reusing Plot Components**: Your existing plot implementations can be adapted to the new GPU-accelerated pipeline. The key is wrapping them with the new caching and rendering infrastructure.

2. **Migrating CSV Import Logic**: The CSV import wizard from Pebble can be enhanced with DuckDB's superior type inference while keeping the familiar UI.

3. **Extracting Proven Patterns**: Cache strategies, data sampling algorithms, and UI components that have proven successful in your tools should be migrated and enhanced.

#### From Open Source Ecosystem:
1. **Rerun Components** (Apache 2.0 License):
   - Study their store subscriber pattern for cache invalidation
   - Adapt their temporal index structures for time-series data
   - Learn from their multi-threading patterns for data loading

2. **Polars** (MIT License):
   - Leverage their lazy evaluation for query optimization
   - Use their efficient sampling algorithms
   - Adopt their memory pool patterns

3. **Arrow DataFusion**:
   - Consider their query planning for complex analytical queries
   - Use their predicate pushdown optimizations
   - Study their memory management patterns

4. **Plotters** (MIT License):
   - Reference their GPU rendering techniques
   - Adapt their level-of-detail algorithms
   - Learn from their text rendering optimizations

5. **Performance Tools**:
   - **pprof-rs**: For production profiling
   - **tracy**: For frame profiling during development
   - **heaptrack**: For memory optimization

### Future Considerations

1. **HTML Export**: While not a priority due to data volumes and the canvas-based interface, we could eventually support HTML exports for specific use cases like sharing individual plots or small dashboards.

2. **AI-assisted queries**: Natural language to SQL with local LLM (no internet required) could help users explore data without SQL knowledge.

---

## 🚀 Getting Started

### Initial Development Tasks

1. **Set up Cargo workspace** with the crate structure and wgpu dependencies
2. **Implement core event system** and traits in `pika-core`
3. **Create GPU rendering foundation** in `pika-plot` with basic wgpu setup
4. **Port and enhance CSV import** from Pebble with DuckDB integration
5. **Implement hierarchical cache** system with primary and derived levels
6. **Build basic canvas** with node rendering and GPU-accelerated plot display

This architecture provides a solid foundation for building Pika-Plot into a world-class data exploration tool. The modular design ensures each component can be developed and tested independently, while the event-driven architecture keeps the system responsive and maintainable.

Ready to build something amazing! 🚀 