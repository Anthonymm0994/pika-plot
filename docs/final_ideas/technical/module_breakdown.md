# 📦 Pika-Plot Module Breakdown

## Overview

This document provides a detailed breakdown of each module within the Pika-Plot crates, including responsibilities, key types, and implementation guidelines.

## 🟢 pika-core

Core shared types and contracts used across all crates.

### `/src/lib.rs`
```rust
pub mod types;
pub mod events;
pub mod errors;
pub mod traits;

pub use types::*;
pub use events::*;
pub use errors::*;
pub use traits::*;
```

### `/src/types.rs`
**Purpose**: All shared type definitions

**Key Exports**:
- `NodeId` - UUID wrapper for node identification
- `ImportOptions` - CSV import configuration
- `QueryResult` - Query execution results
- `PlotConfig` - Plot configuration
- `WorkspaceMode` - Notebook vs Canvas mode
- `ExportFormat` - Supported export formats

### `/src/events.rs`
**Purpose**: Event definitions for UI-Engine communication

**Key Types**:
```rust
pub enum AppEvent {
    // UI -> Engine
    ImportCsv { path: PathBuf, options: ImportOptions },
    ExecuteQuery { id: NodeId, sql: String },
    PreparePlot { id: NodeId, source: NodeId, config: PlotConfig },
    
    // Engine -> UI
    ImportComplete { table_name: String, schema: Arc<Schema> },
    QueryComplete { id: NodeId, result: Result<QueryResult> },
    PlotDataReady { id: NodeId, data: Arc<PlotData> },
}
```

### `/src/errors.rs`
**Purpose**: Comprehensive error types with user-friendly messages

**Implementation**: See `docs/final_ideas/errors/error_types.rs`

### `/src/traits.rs`
**Purpose**: Core trait definitions

**Key Traits**:
```rust
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn render(&mut self, ui: &mut egui::Ui, ctx: &AppContext);
    fn inputs(&self) -> Vec<PortId>;
    fn outputs(&self) -> Vec<PortId>;
}
```

## 🔵 pika-engine

Data processing engine running on separate thread.

### `/src/lib.rs`
```rust
mod engine;
mod storage;
mod cache;
mod import;
mod query;
mod memory;

pub use engine::Engine;
pub use storage::StorageEngine;
```

### `/src/engine.rs`
**Purpose**: Main engine orchestration

**Key Implementation**:
```rust
pub struct Engine {
    storage: StorageEngine,
    query_cache: QueryCache,
    plot_cache: PlotCache,
    memory_monitor: Arc<MemoryMonitor>,
}

impl Engine {
    pub async fn run(mut self, rx: mpsc::Receiver<AppEvent>, tx: mpsc::Sender<AppEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                AppEvent::ImportCsv { path, options } => {
                    let result = self.import_csv(path, options).await;
                    let _ = tx.send(AppEvent::ImportComplete { ... }).await;
                }
                // ... handle other events
            }
        }
    }
}
```

### `/src/storage.rs`
**Purpose**: DuckDB integration

**Key Features**:
- Connection pooling (single connection for embedded)
- Table management
- Schema introspection

**Example**:
```rust
impl StorageEngine {
    pub async fn create_table_from_csv(&self, path: &Path, options: &ImportOptions) -> Result<TableInfo> {
        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_csv_auto('{}', sample_size={})",
            sanitize_table_name(path),
            path.display(),
            options.sample_size
        );
        
        self.conn.lock().await.execute(&sql, [])?;
        self.get_table_info(table_name).await
    }
}
```

### `/src/cache.rs`
**Purpose**: Two-tier caching implementation

**Components**:
1. **QueryCache**: LRU cache for query results
2. **PlotCache**: GPU buffer cache for visible plots

**Implementation**:
```rust
pub struct QueryCache {
    inner: moka::Cache<QueryFingerprint, Arc<RecordBatch>>,
}

impl QueryCache {
    pub fn new() -> Self {
        let cache = moka::Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(300))
            .build();
        Self { inner: cache }
    }
    
    pub fn simple_fingerprint(&self, sql: &str) -> QueryFingerprint {
        // Normalize: lowercase, trim whitespace, remove comments
        let normalized = sql.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        QueryFingerprint(normalized)
    }
}
```

### `/src/import.rs`
**Purpose**: CSV import with type inference

**Key Features**:
- Streaming parser for large files
- Automatic type detection
- Progress reporting
- Error recovery

**Implementation Guide**:
- Use `arrow::csv::ReaderBuilder`
- Leverage DuckDB's `read_csv_auto` for inference
- Report progress via events

### `/src/query.rs`
**Purpose**: SQL query execution

**Key Features**:
- Query validation
- Execution with timeout
- Result conversion to Arrow
- Memory checking before materialization

### `/src/memory.rs`
**Purpose**: Memory monitoring and management

**Platform-specific**:
```rust
#[cfg(target_os = "windows")]
pub fn available_memory() -> usize {
    use windows::Win32::System::SystemInformation::*;
    // Implementation
}
```

## 🎨 pika-ui

User interface components and GPU rendering.

### `/src/lib.rs`
```rust
mod workspace;
mod nodes;
mod canvas;
mod notebook;
mod gpu;
mod export;
mod theme;

pub use workspace::Workspace;
pub use gpu::GpuPlotRenderer;
```

### `/src/workspace.rs`
**Purpose**: Main workspace management

**Key Types**:
```rust
pub struct Workspace {
    pub mode: WorkspaceMode,
    pub events_tx: mpsc::Sender<AppEvent>,
    pub events_rx: mpsc::Receiver<AppEvent>,
    gpu_renderer: GpuPlotRenderer,
}

impl Workspace {
    pub fn ui(&mut self, ctx: &egui::Context) {
        match &mut self.mode {
            WorkspaceMode::Notebook { .. } => self.notebook_ui(ctx),
            WorkspaceMode::Canvas { .. } => self.canvas_ui(ctx),
        }
    }
}
```

### `/src/nodes/mod.rs`
**Purpose**: Node implementations

**Structure**:
```
nodes/
├── mod.rs
├── table.rs
├── query.rs
├── plot.rs
├── transform.rs
└── export.rs
```

**Example Node**:
```rust
// table.rs
pub struct TableNode {
    data: TableNodeData,
}

impl Node for TableNode {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &AppContext) {
        ui.group(|ui| {
            ui.heading(&self.data.table_name);
            
            if let Some(schema) = &self.data.schema {
                ui.collapsing("Schema", |ui| {
                    for field in schema.fields() {
                        ui.label(format!("{}: {}", field.name(), field.data_type()));
                    }
                });
            }
            
            if let Some(count) = self.data.row_count {
                ui.label(format!("{} rows", count));
            }
        });
    }
}
```

### `/src/canvas.rs`
**Purpose**: Canvas mode implementation

**Key Features**:
- Node dragging
- Connection drawing
- Pan/zoom with minimap
- Grid snapping

**Implementation**:
```rust
pub struct CanvasView {
    camera: Camera2D,
    selected_nodes: HashSet<NodeId>,
    drag_state: Option<DragState>,
}
```

### `/src/notebook.rs`
**Purpose**: Notebook mode implementation

**Key Features**:
- Cell management
- Execution order
- Markdown rendering
- Keyboard navigation

### `/src/gpu/mod.rs`
**Purpose**: GPU-accelerated plot rendering

**Structure**:
```
gpu/
├── mod.rs
├── renderer.rs
├── pipelines.rs
├── shaders/
│   ├── direct.wgsl
│   ├── instanced.wgsl
│   └── aggregation.wgsl
```

**Renderer Integration**:
```rust
impl PlotNode {
    fn render_plot(&mut self, ui: &mut egui::Ui, renderer: &GpuPlotRenderer) {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::click_and_drag());
        
        ui.painter().add(egui::paint::PaintCallback {
            rect,
            callback: Arc::new(egui_wgpu::CallbackFn::new(move |info, painter| {
                renderer.render_to_egui(info, painter, &self.cached_data);
            })),
        });
    }
}
```

### `/src/export.rs`
**Purpose**: Export functionality

**Supported Formats**:
- Images: PNG, SVG
- Data: CSV, JSON, Arrow, Parquet
- Workspace: .pikaplot snapshots

### `/src/theme.rs`
**Purpose**: Visual styling and theming

**Implementation**:
```rust
pub fn apply_theme(ctx: &egui::Context, theme: PlotTheme) {
    let mut style = (*ctx.style()).clone();
    
    match theme {
        PlotTheme::Dark => {
            style.visuals = egui::Visuals::dark();
            // Custom dark theme adjustments
        }
        PlotTheme::Light => {
            style.visuals = egui::Visuals::light();
            // Custom light theme adjustments
        }
        PlotTheme::Auto => {
            // Detect system theme
        }
    }
    
    ctx.set_style(style);
}
```

## 🚀 pika-app

Application shell and platform integration.

### `/src/main.rs`
**Purpose**: Application entry point

**Key Responsibilities**:
- Window creation
- Event loop setup
- Runtime initialization
- Panic handling

### `/src/app.rs`
**Purpose**: Main application struct

**Implementation**:
```rust
pub struct PikaPlotApp {
    runtime: AppRuntime,
    workspace: Workspace,
    settings: AppSettings,
}

impl eframe::App for PikaPlotApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle global shortcuts
        self.handle_shortcuts(ctx);
        
        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            self.workspace.ui(ui);
        });
        
        // Process engine events
        while let Ok(event) = self.runtime.engine_to_ui.try_recv() {
            self.workspace.handle_engine_event(event);
        }
    }
}
```

### `/src/runtime.rs`
**Purpose**: Engine thread management

**Key Features**:
- Spawn engine on separate thread
- Channel setup
- Graceful shutdown

### `/src/settings.rs`
**Purpose**: Application settings persistence

**Storage Location**:
- Windows: `%APPDATA%/PikaPlot/settings.json`

## 📟 pika-cli

Command-line interface for automation.

### `/src/main.rs`
**Purpose**: CLI entry point using clap

**Subcommands**:
```rust
#[derive(Parser)]
enum Commands {
    /// Import CSV file into DuckDB
    Ingest {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Execute SQL query
    Query {
        #[arg(short, long)]
        database: PathBuf,
        query: String,
    },
    
    /// Generate plot from query
    Plot {
        #[arg(short, long)]
        database: PathBuf,
        query: String,
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Replay workspace snapshot
    Replay {
        snapshot: PathBuf,
    },
    
    /// Stress test
    Stress {
        #[arg(long, default_value = "1000000")]
        rows: usize,
    },
}
```

### Implementation Guidelines

Each subcommand should:
1. Create an Engine instance
2. Execute operations directly
3. Handle errors gracefully
4. Provide progress output
5. Exit with appropriate codes

## 🧪 Testing Strategy by Module

### pika-core
- Property tests for type serialization
- Exhaustive error message tests

### pika-engine
- Integration tests with real CSV files
- Query correctness tests
- Cache behavior tests
- Memory limit tests

### pika-ui
- Rendering tests using headless backend
- Event handling tests
- GPU shader compilation tests

### pika-app
- End-to-end workflow tests
- Settings persistence tests
- Crash recovery tests

### pika-cli
- Command parsing tests
- Output format validation
- Exit code verification

This modular structure ensures clean separation of concerns while enabling efficient development and testing of each component independently. 