# Pika-Plot Project Organization

## Overview
Pika-Plot is organized as a Rust workspace with six main crates, each serving a specific purpose in the architecture.

## Crate Structure

### 1. **pika-core** (Foundation Layer)
The foundation crate containing core types, traits, and shared functionality.

**Key Components:**
- `types.rs` - Core data types (NodeId, Point2, Size2, PlotConfig, TableInfo)
- `events.rs` - Event bus implementation using broadcast channels
- `snapshot.rs` - Workspace serialization/deserialization
- `error.rs` - Unified error handling
- `plots.rs` - Plot type definitions and configurations
- `node.rs` - Node system base types
- `workspace.rs` - Workspace management structures

**Purpose:** Provides shared types and functionality used across all other crates.

### 2. **pika-traits** (Interface Layer)
Common trait definitions for extensibility.

**Key Components:**
- Common interfaces for data processing
- Plugin system traits (future)

**Purpose:** Defines contracts between different components.

### 3. **pika-engine** (Processing Layer)
Data processing and computation engine.

**Key Components:**
- `enhanced_csv.rs` - CSV parsing and type inference
- `query.rs` - Query execution engine
- `plot/` - Plot generation and rendering
- `cache.rs` - Query result caching
- `workspace.rs` - Workspace persistence
- `gpu/` - GPU acceleration (placeholder)
- `import.rs` - Data import pipelines

**Purpose:** Handles all data processing, analysis, and visualization generation.

### 4. **pika-ui** (User Interface Layer)
Complete GUI implementation using egui.

**Key Components:**
- `app.rs` - Main application state and lifecycle
- `canvas.rs` - Canvas drawing and interaction system
- `panels/` - UI panels matching simplified_overview:
  - `canvas_toolbar.rs` - Top toolbar with drawing tools
  - `data_sources.rs` - Left panel for data management
  - `properties.rs` - Right panel for property editing
  - `canvas_panel.rs` - Main canvas area
  - `status_bar.rs` - Bottom status information
- `widgets/` - Reusable UI components:
  - `file_import_dialog.rs` - CSV import with preview
  - `multi_csv_import.rs` - Batch CSV import
  - `node_editor.rs` - Node property editing
  - `data_table.rs` - Table data display
  - `query_editor.rs` - SQL query interface
- `nodes/` - Canvas node implementations:
  - `plot_node.rs` - Plot visualization nodes
  - `query_node.rs` - Query result nodes
  - `table_node.rs` - Data table nodes
- `screens/` - Full-screen views:
  - `file_config.rs` - Professional file configuration
- `state.rs` - UI state management
- `theme.rs` - Dark theme styling

**Purpose:** Provides the complete user interface matching the simplified_overview vision.

### 5. **pika-app** (GUI Application)
The main GUI executable.

**Key Components:**
- `main.rs` - Application entry point, window creation, event loop

**Purpose:** Launches the GUI application.

### 6. **pika-cli** (CLI Application)
Command-line interface for automation and scripting.

**Key Components:**
- `main.rs` - CLI commands and argument parsing
- `test_plot_exports.rs` - Plot export testing utilities

**Purpose:** Provides command-line access to Pika-Plot functionality.

## Directory Structure

```
pika-plot/
├── pika-core/          # Foundation types and events
├── pika-engine/        # Data processing engine
├── pika-ui/            # User interface
├── pika-app/           # GUI executable
├── pika-cli/           # CLI executable  
├── pika-traits/        # Common interfaces
├── docs/               # Documentation
├── simplified_overview/ # Vision documents
├── test_data/          # Sample data files
├── fixtures/           # Test fixtures
└── Cargo.toml         # Workspace configuration
```

## Data Flow Architecture

1. **User Action** → **UI Event** → **Event Bus** → **State Update** → **UI Refresh**
2. **CSV Import** → **Engine Processing** → **Table Creation** → **Canvas Node**
3. **Plot Request** → **Data Query** → **Plot Generation** → **Visualization**

## Key Design Principles

### 1. **Event-Driven Architecture**
- All components communicate through the event bus
- Loose coupling between UI and engine
- Reactive updates across the application

### 2. **Modular Design**
- Each crate has a single responsibility
- Clear boundaries between layers
- Easy to extend or replace components

### 3. **Canvas-Centric Workflow**
- Everything happens on the canvas
- Nodes represent data, plots, and annotations
- Connections show data flow

### 4. **Professional UI/UX**
- Dark theme by default
- Consistent with simplified_overview vision
- Paint/Excalidraw-like drawing tools

## Testing Organization

Each crate contains its own tests:
- Unit tests in `src/` files under `#[cfg(test)]` 
- Integration tests in `tests/` directories
- 45 total tests in pika-ui alone

## Build Configuration

- **Release Profile:** Optimized with LTO, single codegen unit
- **Dev Profile:** Fast compilation with debug symbols
- **Dependencies:** Minimal set after removing heavy dependencies

## Future Extensibility

The architecture supports:
- Plugin system via pika-traits
- Additional plot types
- New data sources
- GPU acceleration
- Web deployment via WASM 