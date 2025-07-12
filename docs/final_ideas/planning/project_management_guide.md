# 🎯 Pika-Plot Project Management Guide

## Overview

This guide identifies the **minimal necessary documentation** to enable automated implementation of the architecture plan while leveraging the existing `pebble` and `frog-viz` codebases. The goal is to provide just enough specification to avoid blocking issues while trusting AI judgment for implementation details.

> **Important**: All documentation for the Pika-Plot project must be placed in the `docs/final_ideas/` directory. This is the centralized location for all project documentation, specifications, and planning documents.

---

## 📋 What MUST Be Explicitly Documented

### 1. **Core Type Definitions & Contracts** (`docs/final_ideas/types/`)

**Required**: Concrete Rust type definitions for all data structures that cross crate boundaries.

```rust
// docs/final_ideas/types/core_types.rs
// Essential types that need explicit definition:

pub struct ImportOptions {
    pub delimiter: char,           // default: ','
    pub has_header: bool,         // default: true
    pub quote_char: char,         // default: '"'
    pub null_values: Vec<String>, // default: ["", "NULL", "null", "N/A"]
    pub sample_size: usize,       // default: 10000
    pub type_inference: bool,     // default: true
}

pub struct PlotConfig {
    pub plot_type: PlotType,
    pub x_column: String,
    pub y_column: Option<String>,
    pub color_by: Option<String>,
    pub size_by: Option<String>,
    pub facet_by: Option<String>,
    pub aggregation: Option<AggregationType>,
}

pub enum PlotType {
    Scatter,
    Line,
    Histogram,
    Bar,
    Heatmap,
    // ... complete enumeration from frog-viz
}
```

**Why**: These types define the API contract between crates. AI cannot infer these - they must match exactly.

### 2. **GPU Data Layout & Shader Interface** (`docs/final_ideas/gpu/`)

**Required**: Vertex buffer layouts and compute shader interfaces.

```wgsl
// docs/final_ideas/gpu/aggregation.wgsl
struct AggregationParams {
    viewport_min: vec2<f32>,
    viewport_max: vec2<f32>,
    bin_size_x: f32,
    bin_size_y: f32,
    max_bins: u32,
}

@group(0) @binding(0) var<storage, read> input_points: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read_write> output_grid: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: AggregationParams;
```

**Why**: GPU code must be precise. Reuse shader patterns from `frog-viz` but adapted for wgpu.

### 3. **Workspace Snapshot Schema** (`docs/final_ideas/formats/snapshot.ron`)

**Required**: Complete RON example of a saved workspace.

```ron
WorkspaceSnapshot(
    version: 1,
    mode: Canvas(
        nodes: [
            (
                id: "a1b2c3d4-...",
                position: (100.0, 200.0),
                node_type: Table(
                    source_path: "data/sales.csv",
                    table_name: "sales_2024",
                    import_options: (
                        delimiter: ',',
                        has_header: true,
                    ),
                ),
            ),
        ],
        connections: [
            (from: ("a1b2c3d4-...", "output"), to: ("e5f6g7h8-...", "input")),
        ],
    ),
    data_sources: [
        (
            path: "data/sales.csv",
            hash: "sha256:abcdef123456...",
            table_name: "sales_2024",
        ),
    ],
)
```

**Why**: File format is a hard API that users depend on. Must be stable from day one.

### 4. **Error Types & Recovery** (`docs/final_ideas/errors/`)

**Required**: Complete error taxonomy with user-facing messages.

```rust
// docs/final_ideas/errors/error_types.rs
#[derive(thiserror::Error, Debug)]
pub enum PikaError {
    #[error("Not enough GPU memory: need {required}MB, have {available}MB")]
    GpuMemory { required: usize, available: usize },
    
    #[error("CSV import failed at line {line}: {reason}")]
    CsvImport { line: usize, reason: String },
    
    #[error("Query timed out after {seconds}s")]
    QueryTimeout { seconds: u64 },
    
    // ... complete enumeration
}
```

**Why**: Error handling directly impacts user experience. Messages must be helpful.

---

## 🎨 What Can Be Left to AI Judgment

### 1. **UI Layout & Styling**
- Exact pixel dimensions for nodes
- Color schemes (use egui defaults)
- Icon choices
- Animation timings

**Rationale**: The existing `pebble` app demonstrates good egui patterns. AI can adapt its UI approach.

### 2. **Internal Algorithms**
- Specific cache eviction strategies
- Query fingerprinting details
- Thread pool sizing
- Buffer allocation strategies

**Rationale**: These are implementation details that can be tuned later without breaking APIs.

### 3. **Performance Thresholds**
- When to switch rendering modes (50k vs 100k points)
- Cache size limits
- Timeout values

**Rationale**: These need empirical tuning anyway. Start with reasonable defaults.

### 4. **Keyboard Shortcuts**
- Beyond basic ones (Ctrl+S for save)
- Canvas navigation keys
- Cell manipulation keys

**Rationale**: Can be refined based on user feedback. Implementation details can evolve.

---

## 🔧 How to Leverage Existing Codebases

### From `pebble` (SQLite Viewer)
**Reuse directly**:
- CSV import dialog UI (`pebble/src/import/`)
- Table schema display widget
- Query result rendering
- File chooser integration

**Adapt for DuckDB**:
- Replace SQLite calls with DuckDB equivalents
- Keep the UI patterns and error handling

### From `frog-viz` (Plot Library)
**Reuse directly**:
- Plot type implementations
- Color scales and palettes
- Axis calculation logic
- Legend generation

**Adapt for GPU**:
- Convert immediate-mode rendering to GPU buffers
- Add aggregation for large datasets
- Integrate with egui::PaintCallback

---

## 📁 Required Project Structure

```
pika-plot/
├── docs/
│   └── final_ideas/
│       ├── types/
│       │   └── core_types.rs      # All shared types
│       ├── gpu/
│       │   ├── aggregation.wgsl   # Compute shader
│       │   └── vertex_layout.md   # Buffer specifications
│       ├── formats/
│       │   └── snapshot.ron       # Example workspace file
│       └── errors/
│           └── error_types.rs     # Complete error enum
├── fixtures/
│   ├── small.csv             # 100 rows, clean data
│   ├── medium.csv            # 10k rows, some nulls
│   └── large.csv             # 1M rows for benchmarks
├── pebble/                   # Existing SQLite viewer
├── frog-viz/                 # Existing plot library
└── src/                      # New implementation
```

---

## 🚀 Implementation Kickstart Commands

```bash
# 1. Create the 5-crate workspace
cargo new --lib pika-core
cargo new --lib pika-engine  
cargo new --lib pika-ui
cargo new --bin pika-app
cargo new --bin pika-cli

# 2. Copy type definitions
cp docs/final_ideas/types/core_types.rs pika-core/src/types.rs

# 3. Set up dependencies (Cargo.toml provided below)

# 4. Run this test to verify setup
cargo test --workspace
```

### Workspace Cargo.toml

```toml
[workspace]
members = ["pika-core", "pika-engine", "pika-ui", "pika-app", "pika-cli"]
resolver = "2"

[workspace.dependencies]
# Core
egui = "0.24"
eframe = { version = "0.24", features = ["wgpu"] }
wgpu = "0.18"

# Data  
duckdb = { version = "0.10", features = ["bundled"] }
arrow = "50"

# Async
tokio = { version = "1", features = ["full"] }

# Utils
uuid = { version = "1", features = ["v4", "serde"] }
serde = { version = "1", features = ["derive"] }
ron = "0.8"
thiserror = "1"
anyhow = "1"

# Caching
moka = { version = "0.12", features = ["future"] }
dashmap = "5"

# CLI
clap = { version = "4", features = ["derive"] }
```

---

## ✅ Pre-Implementation Checklist

**Must Have** (blocks implementation):
- [ ] Core type definitions in `docs/final_ideas/types/`
- [ ] GPU vertex layout specification in `docs/final_ideas/gpu/`
- [ ] Aggregation shader pseudocode in `docs/final_ideas/gpu/`
- [ ] Snapshot file format example in `docs/final_ideas/formats/`
- [ ] Error types with messages in `docs/final_ideas/errors/`
- [ ] 3 test CSV files in `fixtures/`

**Nice to Have** (can be refined during implementation):
- [ ] Keyboard shortcut list
- [ ] Exact node dimensions
- [ ] Performance threshold values
- [ ] Memory limit percentages

**Don't Need** (AI can infer):
- Detailed UI mockups (use pebble as reference)
- Exact color values
- Animation curves
- Thread pool sizes
- Cache eviction algorithms

---

## 🎯 Key Success Factors

1. **Trust AI for egui patterns** - The existing pebble codebase shows good practices
2. **Specify data formats exactly** - File formats and GPU layouts must be precise
3. **Reuse aggressively** - Both pebble and frog-viz have solved many problems already
4. **Start with working defaults** - Don't over-specify tunable parameters
5. **Focus on data flow** - Clear types and events matter more than pixel-perfect UI

---

## 📦 CLI Interface (Parallel Entry Point)

A lightweight CLI companion to the GUI that exposes core engine functionality for testing, automation, and headless operation.

### Structure

Add a fifth crate to the workspace:
```
pika-cli/
└── src/main.rs  # Uses clap for parsing
```

### Essential Subcommands

```bash
# Ingest CSV and create DuckDB table
pika-cli ingest data.csv --output data.db --table sales_2024

# Execute query and save results  
pika-cli query data.db "SELECT * FROM sales_2024" --output results.arrow

# Generate static plot from query
pika-cli plot data.db "SELECT x, y FROM sales_2024" --type scatter --output plot.png

# Replay a workspace snapshot (headless)
pika-cli replay workspace.ron --export-plots ./output/

# Stress test with synthetic data
pika-cli stress --rows 10000000 --columns 50 --plot scatter
```

### Why This Matters

1. **Enforces Clean Architecture** - Engine must work independently of GUI
2. **Enables Testing** - Each subcommand exercises a specific API
3. **Supports Automation** - CI/CD pipelines, batch processing
4. **Validates Design** - If it's hard to expose via CLI, the API is probably wrong

### Implementation Notes

- Each subcommand maps to a public engine API
- Reuse all engine logic - no duplication
- Output formats: `.arrow`, `.csv`, `.json`, `.png`, `.svg`
- Keep it simple - no interactive modes
- Use structured logging for debugging

This CLI is not a replacement for the GUI but a complementary tool that makes the system more robust and testable.

---

## 📊 Minimal Viable Documentation

If pressed for time, these are the absolute essentials:

1. **Type definitions** for ImportOptions, PlotConfig, and AppEvent
2. **One complete RON example** of a saved workspace  
3. **GPU vertex layout** for point data
4. **Test fixtures** - at least one CSV file

Everything else can be inferred from the plan, existing codebases, and AI judgment.

Remember: The goal is to build an ambitious tool quickly. Over-documentation slows velocity. Trust the AI to make reasonable choices within the constraints provided. 