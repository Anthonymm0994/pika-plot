# 📋 Implementation Readiness Assessment for Pika-Plot

## 🔴 Critical Missing Documents

### 1. **Concrete Type Definitions & Data Schemas**
The plan references many types but doesn't define them precisely. An agent needs:

```rust
// File: docs/types/core_types.rs
// Complete definitions for:
pub struct ImportOptions {
    pub delimiter: char,
    pub has_header: bool,
    pub null_values: Vec<String>,
    pub type_overrides: HashMap<String, DataType>,
    pub sample_size: usize,
    // ... all fields
}

pub struct PlotConfig {
    pub plot_type: PlotType,
    pub x_column: String,
    pub y_column: String,
    pub color_column: Option<String>,
    pub size_column: Option<String>,
    pub aggregation: Option<AggregationType>,
    // ... all fields
}

pub struct GpuBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub instance_count: u32,
    // ... all fields
}
```

### 2. **Node Serialization Formats**
The snapshot system mentions `NodeSnapshot` but doesn't define it:

```rust
// File: docs/serialization/node_formats.rs
#[derive(Serialize, Deserialize)]
pub struct TableNodeSnapshot {
    pub id: NodeId,
    pub position: Point2,
    pub source_path: PathBuf,
    pub table_name: String,
    pub import_options: ImportOptions,
    pub schema: Schema,
}

#[derive(Serialize, Deserialize)]
pub struct QueryNodeSnapshot {
    pub id: NodeId,
    pub position: Point2,
    pub sql: String,
    pub input_connections: Vec<NodeId>,
    pub cached_schema: Option<Schema>,
}
// ... for each node type
```

### 3. **GPU Shader Specifications**
The plan mentions compute shaders but provides no details:

```wgsl
// File: docs/shaders/aggregation.wgsl
// Compute shader for point aggregation
@group(0) @binding(0) var<storage, read> input_points: array<Point2>;
@group(0) @binding(1) var<storage, read_write> output_bins: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: AggregationParams;

struct AggregationParams {
    viewport_min: vec2<f32>,
    viewport_max: vec2<f32>,
    bin_count_x: u32,
    bin_count_y: u32,
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Aggregation logic
}
```

### 4. **Complete Error Taxonomy**
The agent needs all possible error types:

```rust
// File: docs/errors/error_types.rs
#[derive(thiserror::Error, Debug)]
pub enum PikaError {
    #[error("DuckDB error: {0}")]
    DuckDb(#[from] duckdb::Error),
    
    #[error("CSV import failed: {reason}")]
    CsvImport { reason: String, line: Option<usize> },
    
    #[error("GPU initialization failed: {0}")]
    GpuInit(String),
    
    #[error("Memory limit exceeded: need {required} bytes, have {available}")]
    MemoryLimit { required: usize, available: usize },
    
    // ... complete enumeration
}
```

## 🟡 Missing Implementation Details

### 1. **Canvas Layout Algorithm**
The canvas mode needs specific layout rules:

```rust
// File: docs/algorithms/canvas_layout.md
## Node Layout Rules
- New nodes appear at cursor position or (100, 100) if no cursor
- Minimum node spacing: 50 pixels
- Connection routing: Manhattan distance with rounded corners
- Snap-to-grid: 10px grid, optional
- Node dimensions: Table(300x200), Query(350x250), Plot(400x400)
```

### 2. **Plot Type Specifications**
Each plot type needs exact configuration:

```yaml
# File: docs/plots/plot_specifications.yaml
scatter:
  required_columns: [x, y]
  optional_columns: [color, size, label]
  gpu_mode: instanced_points
  max_raw_points: 50000
  aggregation_strategy: density_grid
  
histogram:
  required_columns: [value]
  optional_columns: [weight]
  gpu_mode: compute_bars
  bin_calculation: sturges_rule
  max_bins: 1000
  
line:
  required_columns: [x, y]
  optional_columns: [group_by]
  gpu_mode: line_strip
  sampling_strategy: lttb
  max_points_per_line: 10000
```

### 3. **Memory Calculation Rules**
Precise memory estimation formulas:

```rust
// File: docs/memory/calculation_rules.rs
impl MemoryEstimator {
    pub fn estimate_import(csv_size_bytes: usize, column_count: usize) -> usize {
        // DuckDB typically uses 2-3x CSV size in memory during import
        csv_size_bytes * 3 + (column_count * 1024 * 1024) // column overhead
    }
    
    pub fn estimate_query_result(schema: &Schema, row_count: usize) -> usize {
        schema.fields().iter()
            .map(|f| Self::bytes_per_value(f.data_type()) * row_count)
            .sum::<usize>()
            + 1024 * 1024 // Arrow metadata overhead
    }
    
    pub fn estimate_gpu_buffer(point_count: usize, attributes_per_point: usize) -> usize {
        point_count * attributes_per_point * 4 // f32 per attribute
            + point_count * 6 * 2 // indices for quad expansion
    }
}
```

## 🟢 Environment Setup Requirements

### 1. **Workspace Cargo.toml Template**
```toml
# File: templates/workspace_cargo.toml
[workspace]
members = ["pika-core", "pika-engine", "pika-ui", "pika-app"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Core
tokio = { version = "1", features = ["full"] }
egui = "0.24"
eframe = { version = "0.24", features = ["wgpu"] }
wgpu = "0.18"

# Data
arrow = "50"
duckdb = { version = "0.10", features = ["bundled", "arrow"] }
serde = { version = "1", features = ["derive"] }

# Common
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
anyhow = "1"
```

### 2. **Initial Entry Points**
The agent needs these files to start:

```rust
// File: templates/pika-app/src/main.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()?),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_options(),
        ..Default::default()
    };
    
    eframe::run_native(
        "Pika-Plot",
        options,
        Box::new(|cc| Box::new(PikaPlotApp::new(cc))),
    )?;
    
    Ok(())
}

fn wgpu_options() -> egui_wgpu::WgpuConfiguration {
    egui_wgpu::WgpuConfiguration {
        supported_backends: wgpu::Backends::PRIMARY,
        device_descriptor: wgpu::DeviceDescriptor {
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            limits: wgpu::Limits::default(),
            label: Some("Pika-Plot Device"),
        },
        ..Default::default()
    }
}
```

### 3. **Task Queue for Implementation**
```yaml
# File: docs/implementation_tasks.yaml
tasks:
  - id: setup_workspace
    description: Create 4-crate workspace with dependencies
    files_to_create:
      - Cargo.toml
      - pika-core/Cargo.toml
      - pika-engine/Cargo.toml
      - pika-ui/Cargo.toml
      - pika-app/Cargo.toml
    
  - id: implement_core_types
    description: Define all shared types in pika-core
    depends_on: [setup_workspace]
    files_to_create:
      - pika-core/src/lib.rs
      - pika-core/src/types.rs
      - pika-core/src/events.rs
      - pika-core/src/errors.rs
    
  - id: duckdb_integration
    description: Create DuckDB storage engine
    depends_on: [implement_core_types]
    files_to_create:
      - pika-engine/src/storage.rs
      - pika-engine/src/import.rs
      - pika-engine/src/query.rs
```

## 🔧 Missing Specifications

### 1. **UI Component Specifications**
```rust
// File: docs/ui/component_specs.rs
pub struct NodeStyle {
    pub background_color: Color32,
    pub border_color: Color32,
    pub border_width: f32,
    pub corner_radius: f32,
    pub shadow: Shadow,
    pub min_size: Vec2,
    pub max_size: Vec2,
    pub padding: Margin,
}

pub const TABLE_NODE_STYLE: NodeStyle = NodeStyle {
    background_color: Color32::from_rgb(45, 45, 48),
    border_color: Color32::from_rgb(63, 63, 70),
    border_width: 1.0,
    corner_radius: 8.0,
    // ... etc
};
```

### 2. **Keyboard Shortcuts & Commands**
```yaml
# File: docs/ui/keyboard_shortcuts.yaml
global:
  - key: "Ctrl+N"
    action: "new_workspace"
  - key: "Ctrl+O"
    action: "open_csv"
  - key: "F2"
    action: "toggle_mode"
    
canvas_mode:
  - key: "Space+Drag"
    action: "pan_canvas"
  - key: "Ctrl+Scroll"
    action: "zoom_canvas"
  - key: "Delete"
    action: "delete_selected_nodes"
    
notebook_mode:
  - key: "Ctrl+Enter"
    action: "execute_cell"
  - key: "Alt+Up/Down"
    action: "move_cell"
```

### 3. **Connection Rules**
```rust
// File: docs/canvas/connection_rules.rs
pub struct ConnectionRules {
    pub allowed_connections: Vec<(NodeType, PortType, NodeType, PortType)>,
}

impl ConnectionRules {
    pub fn can_connect(&self, from: &Node, from_port: PortId, to: &Node, to_port: PortId) -> Result<()> {
        // TableNode.output -> QueryNode.input ✓
        // QueryNode.output -> PlotNode.input ✓
        // PlotNode.output -> anything ✗
        // Circular connections ✗
    }
}
```

## 📁 Suggested File Structure for Agent

```
pika-plot/
├── docs/
│   ├── types/              # Complete type definitions
│   ├── ui/                 # UI specifications
│   ├── algorithms/         # Layout, aggregation algorithms
│   ├── shaders/            # WGSL shader code
│   ├── errors/             # Error taxonomy
│   ├── implementation/     # Task queue, dependencies
│   └── test_fixtures/      # Sample data for testing
├── templates/              # Boilerplate files
└── existing_code/          # Your SQLite viewer & plot implementations
```

## 🎯 Implementation Guidance for Agent

Create a file `docs/agent_instructions.md`:

```markdown
# Agent Implementation Instructions

## Entry Point
Start by creating the workspace structure in `templates/workspace_cargo.toml`.

## Implementation Order
1. Create all Cargo.toml files
2. Implement pika-core completely (it has no dependencies)
3. Implement pika-engine storage module
4. Create basic UI shell in pika-app
5. Add nodes one at a time to pika-ui
6. Integrate GPU rendering last

## Testing Each Step
After each module:
1. Run `cargo check --workspace`
2. Run `cargo test -p <crate-name>`
3. Create a simple example in examples/ to verify

## Reusing Existing Code
- CSV import logic: Adapt from existing_code/csv_import.rs
- Plot implementations: Port from existing_code/plots/
- Use existing code as reference but rewrite for new architecture

## Key Invariants
- Never hold locks across await points
- Always check memory before materializing large results  
- GPU buffers must be dropped when plots are hidden
- All paths must use PathBuf, not String
```

## ✅ Final Checklist

Before the agent can start, ensure you have:

1. □ Complete type definitions for all referenced types
2. □ Shader code for GPU aggregation (at least pseudocode)
3. □ Exact node dimensions and styling constants
4. □ Connection validation rules
5. □ Memory calculation formulas
6. □ Sample CSV files in `docs/test_fixtures/`
7. □ Your existing plot implementation code available
8. □ Decision on RON vs JSON for snapshots
9. □ Exact wgpu feature flags needed
10. □ List of acceptable SQL syntax limitations

With these additions, an automation agent should be able to implement the system without constantly asking for clarifications.