# 🏗️ Unified Architecture: The Data Sketchpad

## 1. 🔍 Diagnosing the Design Space

### Real Tradeoffs Identified

**SQLite vs DuckDB-only for metadata:**
- **SQLite pros**: Familiar, transactional metadata updates, small footprint
- **SQLite cons**: Another moving part, sync complexity, impedance mismatch
- **Winner**: DuckDB-only, but with a twist—use a separate metadata connection pool

**Tiered vs Unified Cache:**
- **Tiered pros**: Clear separation of concerns, predictable memory usage
- **Tiered cons**: Complex invalidation logic, potential duplication
- **Unified pros**: Simpler mental model, automatic deduplication
- **Winner**: Hybrid—unified storage with logical access patterns

**Manual vs Automatic Memory Management:**
- **Manual pros**: Explicit control, predictable behavior
- **Auto pros**: Less code, leverages DuckDB's battle-tested spilling
- **Winner**: Automatic with escape hatches for power users

### Hidden Complexities to Address

1. **Windows file locking**: Need careful handling of memory-mapped files
2. **Reactive propagation**: Must avoid cascade storms in complex graphs
3. **Snapshot portability**: Arrow IPC format varies by version
4. **Type inference stability**: User overrides must persist across sessions

## 2. 🎯 The Merged Architecture

### Core Principle: "Layered Simplicity"

```
┌─────────────────────────────────────────────────────────────────┐
│                          USER SPACE                              │
│                                                                  │
│  Canvas Workspace                                               │
│  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐ │
│  │  Table  │────▶│  Query  │────▶│  Plot   │     │  Note   │ │
│  │  Node   │     │  Node   │     │  Node   │     │  Node   │ │
│  └─────────┘     └─────────┘     └─────────┘     └─────────┘ │
│                                                                  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      INTERACTION LAYER                           │
│                                                                  │
│  Event Bus           Canvas State         Selection Manager     │
│  ┌──────────┐       ┌────────────┐       ┌───────────────┐   │
│  │ UI Events│       │Node Layout │       │Brush & Link   │   │
│  │ Dispatch │       │Connections │       │Propagation    │   │
│  └──────────┘       └────────────┘       └───────────────┘   │
│                                                                  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                         COMPUTE LAYER                            │
│                                                                  │
│  Query Engine         Smart Cache          Plot Pipeline        │
│  ┌────────────┐      ┌────────────┐      ┌──────────────┐    │
│  │AST Parser  │      │Semantic    │      │Adaptive      │    │
│  │Normalizer  │─────▶│Fingerprint │─────▶│Sampling      │    │
│  │Executor    │      │Cache       │      │Aggregation   │    │
│  └────────────┘      └────────────┘      └──────────────┘    │
│                                                                  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                         STORAGE LAYER                            │
│                                                                  │
│  DuckDB Engine         Arrow Store         Workspace Store      │
│  ┌────────────┐      ┌────────────┐      ┌──────────────┐    │
│  │Data Tables │      │Immutable   │      │Session State │    │
│  │Metadata    │      │RecordBatch │      │Snapshots     │    │
│  │Indexes     │      │Cache       │      │User Prefs    │    │
│  └────────────┘      └────────────┘      └──────────────┘    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── main.rs                    # Application entry point
├── app/
│   ├── mod.rs                # App state and lifecycle
│   ├── config.rs             # User preferences
│   └── theme.rs              # Visual theming
│
├── canvas/
│   ├── mod.rs                # Canvas workspace
│   ├── node.rs               # Node types and rendering
│   ├── connection.rs         # Edge rendering and data flow
│   ├── layout.rs             # Auto-layout algorithms
│   └── interaction.rs        # Drag, zoom, pan handlers
│
├── compute/
│   ├── mod.rs                # Compute orchestration
│   ├── query/
│   │   ├── parser.rs         # SQL parsing with sqlparser-rs
│   │   ├── normalizer.rs     # AST normalization
│   │   └── executor.rs       # DuckDB execution
│   ├── cache/
│   │   ├── store.rs          # Unified cache storage
│   │   ├── fingerprint.rs    # Semantic hashing
│   │   └── eviction.rs       # Memory pressure handling
│   └── plot/
│       ├── sampler.rs        # Adaptive sampling strategies
│       ├── aggregator.rs     # Binning and aggregation
│       └── renderer.rs       # Plot-specific transforms
│
├── storage/
│   ├── mod.rs                # Storage abstraction
│   ├── duckdb.rs             # DuckDB connection management
│   ├── arrow.rs              # Arrow cache management
│   ├── workspace.rs          # Workspace serialization
│   └── migration.rs          # Version migration logic
│
├── ui/
│   ├── mod.rs                # UI components
│   ├── table_view.rs         # Data grid widget
│   ├── query_editor.rs       # SQL editor with highlighting
│   ├── plot_view.rs          # Plot rendering widget
│   └── dialogs/
│       ├── type_override.rs  # Column type override dialog
│       ├── memory_warning.rs # Memory pressure dialog
│       └── export.rs         # Export/share dialog
│
└── tests/
    ├── integration/          # End-to-end tests
    ├── benchmarks/          # Performance benchmarks
    └── fixtures/            # Test data
```

## 3. 📊 Data Flow Diagram

### User Journey: CSV to Insight

```
User Action                     System Response
───────────                     ───────────────

Drag sales.csv ─────┐
                    ▼
              ┌─────────────────┐
              │ FileDropHandler │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐        ┌──────────────┐
              │ Type Inspector  │───────▶│ Type Dialog? │
              │ (sample 1000)   │        │ (if needed)  │
              └────────┬────────┘        └──────────────┘
                       ▼
              ┌─────────────────┐
              │ DuckDB Import   │
              │ CREATE TABLE... │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Canvas adds     │
              │ Table Node      │
              └────────┬────────┘
                       │
User clicks table ─────┘
                    ▼
              ┌─────────────────┐
              │ Preview Query   │
              │ SELECT * LIMIT  │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐        ┌──────────────┐
              │ Check Cache     │───────▶│ Found?       │──No──┐
              │ (fingerprint)   │        └──────────────┘      │
              └────────┬────────┘                              │
                       │                                       ▼
                       │                              ┌────────────────┐
                       │                              │Execute Preview │
                       │                              │Cache Result    │
                       │                              └────────┬───────┘
                       ▼                                       │
              ┌─────────────────┐◀─────────────────────────────┘
              │ Render Grid     │
              │ (virtual scroll)│
              └─────────────────┘

User writes SQL ─────┐
                     ▼
              ┌─────────────────┐
              │ SQL Editor      │
              │ (syntax highlight)
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Parse + Normalize│
              │ AST fingerprint │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Semantic Cache  │
              │ Check/Store     │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Update Preview  │
              │ Notify Plots    │
              └─────────────────┘

User drags to plot ──┐
                     ▼
              ┌─────────────────┐
              │ Plot Type Menu  │
              │ (context-aware) │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Data Pipeline   │
              │ Sample/Aggregate│
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │ Plot Renderer   │
              │ (GPU accelerated)
              └─────────────────┘
```

## 4. 🧩 Key Architectural Innovations

### 1. The Semantic Cache

```rust
// Not just SQL text hashing—true semantic understanding
pub struct SemanticCache {
    store: Arc<DashMap<Fingerprint, CachedResult>>,
    normalizer: AstNormalizer,
}

impl SemanticCache {
    pub fn fingerprint(&self, sql: &str) -> Result<Fingerprint> {
        let ast = self.normalizer.parse_and_normalize(sql)?;
        
        // These all produce the same fingerprint:
        // - SELECT * FROM sales WHERE price > 100
        // - SELECT * FROM sales WHERE price > 100.0
        // - select * from SALES where PRICE>100
        
        Ok(Fingerprint {
            structure_hash: hash_ast_structure(&ast),
            tables: extract_table_versions(&ast),
            predicates: normalize_predicates(&ast),
        })
    }
    
    pub fn find_superset(&self, fingerprint: &Fingerprint) -> Option<CachedResult> {
        // If we have "WHERE x > 100" cached
        // and user asks for "WHERE x > 100 AND y < 50"
        // we can filter the cached superset instead of re-querying
        
        self.store.iter()
            .filter(|entry| entry.fingerprint.is_superset_of(fingerprint))
            .min_by_key(|entry| entry.result_size)
            .map(|entry| entry.value().clone())
    }
}
```

### 2. Reactive Canvas with Stable Layout

```rust
pub struct CanvasWorkspace {
    nodes: SlotMap<NodeId, Node>,
    edges: Vec<Edge>,
    layout_engine: ForceDirectedLayout,
    selection: SelectionState,
}

impl CanvasWorkspace {
    pub fn add_node(&mut self, node_type: NodeType, position: Option<Vec2>) -> NodeId {
        let id = self.nodes.insert(Node {
            node_type,
            position: position.unwrap_or_else(|| self.suggest_position()),
            size: Vec2::new(200.0, 150.0),
            ports: node_type.default_ports(),
        });
        
        // Stabilize layout without disrupting user's mental map
        self.layout_engine.add_node_soft(id);
        
        id
    }
    
    pub fn connect(&mut self, from: PortId, to: PortId) -> Result<EdgeId> {
        // Validate connection semantics
        let from_type = self.nodes[from.node].ports[from.port].data_type();
        let to_type = self.nodes[to.node].ports[to.port].data_type();
        
        if !from_type.compatible_with(&to_type) {
            return Err(Error::IncompatibleConnection);
        }
        
        let edge = Edge { from, to, state: EdgeState::Active };
        self.edges.push(edge);
        
        // Trigger data flow
        self.propagate_change(from.node);
        
        Ok(EdgeId(self.edges.len() - 1))
    }
}
```

### 3. Incremental Plot Updates

```rust
pub struct PlotNode {
    config: PlotConfig,
    data_source: NodeId,
    cached_data: Option<Arc<PlotData>>,
    viewport: Viewport,
    renderer: Box<dyn PlotRenderer>,
}

impl PlotNode {
    pub fn update(&mut self, change: &DataChange) -> RenderCommand {
        match change {
            DataChange::RowsAdded(new_rows) => {
                // Incremental update without full recompute
                if let Some(data) = &mut self.cached_data {
                    Arc::make_mut(data).append_rows(new_rows);
                    self.renderer.incremental_update(new_rows)
                } else {
                    self.full_recompute()
                }
            },
            
            DataChange::ViewportChanged(viewport) => {
                self.viewport = *viewport;
                // Only resample if zoom level changed significantly
                if self.needs_resample() {
                    self.renderer.resample_for_viewport(viewport)
                } else {
                    RenderCommand::Redraw
                }
            },
            
            DataChange::FilterChanged(_) => {
                // Full recompute needed
                self.full_recompute()
            }
        }
    }
}
```

## 5. 📋 Modularization & Testing Strategy

### Module Boundaries

```rust
// Each module exposes a clean trait-based API

// compute/query/mod.rs
pub trait QueryEngine: Send + Sync {
    fn parse(&self, sql: &str) -> Result<ParsedQuery>;
    fn execute(&self, query: &ParsedQuery) -> Result<QueryResult>;
    fn explain(&self, query: &ParsedQuery) -> Result<QueryPlan>;
}

// compute/cache/mod.rs
pub trait CacheStrategy: Send + Sync {
    fn get(&self, key: &Fingerprint) -> Option<Arc<CachedData>>;
    fn put(&self, key: Fingerprint, data: Arc<CachedData>);
    fn evict(&self, predicate: impl Fn(&CachedData) -> bool);
}

// canvas/mod.rs
pub trait CanvasBackend {
    fn add_node(&mut self, node: Node) -> NodeId;
    fn remove_node(&mut self, id: NodeId);
    fn connect(&mut self, from: PortId, to: PortId) -> Result<()>;
    fn layout(&mut self, strategy: LayoutStrategy);
}
```

### Testing Pyramid

```
┌─────────────────┐
│   E2E Tests     │  5%
│  (Full app flow)│
├─────────────────┤
│ Integration     │  20%
│ (Module pairs)  │
├─────────────────┤
│    Unit Tests   │  75%
│ (Pure functions)│
└─────────────────┘
