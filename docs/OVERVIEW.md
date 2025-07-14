# Pika-Plot: Multi-Level Overview

## 🎯 Executive Summary (30-second read)

Pika-Plot is a Rust-based data visualization application that combines the creativity of canvas-based tools (like Excalidraw) with the power of data analysis software (like Tableau). Users can import data, create visualizations, and build interactive workflows on an infinite canvas.

**Key Value Proposition**: Visual data exploration meets creative canvas interaction.

## 📊 Product Overview (2-minute read)

### What is Pika-Plot?

Pika-Plot is a desktop application for data visualization that treats everything as nodes on an infinite canvas. Unlike traditional data tools that force you into rigid layouts, Pika-Plot lets you:

- **Import & Visualize**: CSV files become interactive tables and 26 different plot types
- **Draw & Annotate**: Add context with shapes, arrows, and text
- **Connect & Flow**: Link data nodes to create visual workflows
- **Explore Freely**: Pan and zoom across your data landscape

### Who is it for?

- **Data Analysts**: Create explorable data stories
- **Researchers**: Document analysis workflows visually
- **Students**: Learn data visualization interactively
- **Anyone**: Who wants to understand their data better

### Core Features

1. **Canvas-First Design**: Infinite workspace for spatial organization
2. **26 Plot Types**: From basic bar charts to advanced 3D visualizations
3. **Smart Data Import**: Type inference and preview for CSV files
4. **Visual Workflows**: Connect nodes to show data flow
5. **Professional UI**: Dark theme, keyboard shortcuts, context menus

## 🏗️ Technical Architecture (5-minute read)

### System Design

Pika-Plot follows a modular architecture with clear separation of concerns:

```
┌─────────────┐
│   pika-app  │  ← Main application entry point
└──────┬──────┘
       │
┌──────┴──────┐
│   pika-ui   │  ← User interface layer (egui-based)
└──────┬──────┘
       │
┌──────┴──────┐
│ pika-engine │  ← Data processing and visualization
└──────┬──────┘
       │
┌──────┴──────┐
│  pika-core  │  ← Core types and business logic
└─────────────┘
```

### Key Technologies

- **Language**: Rust (for performance and safety)
- **GUI Framework**: egui (immediate mode UI)
- **Graphics**: wgpu (GPU-accelerated rendering)
- **Data Processing**: DuckDB (embedded analytics)
- **Async Runtime**: tokio (for non-blocking I/O)

### Design Principles

1. **Everything is a Node**: Tables, plots, shapes - all are nodes
2. **Event-Driven**: Components communicate via events
3. **Type-Safe**: Leveraging Rust's type system
4. **Performance-First**: GPU acceleration where possible
5. **User-Centric**: Intuitive interactions and visual feedback

## 🔧 Component Deep Dive (10-minute read)

### pika-core
The foundation layer providing:
- **Types**: `Node`, `NodeId`, `Point2`, `Size2`
- **Events**: Broadcast channel-based event system
- **Workspace**: Canvas state management
- **Errors**: Comprehensive error handling

```rust
// Example: Core node trait
pub trait Node: Send + Sync {
    fn id(&self) -> NodeId;
    fn position(&self) -> Point2;
    fn size(&self) -> Size2;
}
```

### pika-engine
The computational powerhouse handling:
- **Data Import**: CSV parsing with type inference
- **Query Engine**: DuckDB integration for SQL queries
- **Plot Rendering**: 26 different visualization types
- **GPU Acceleration**: WGSL shaders for performance
- **Memory Management**: Efficient data handling

```rust
// Example: Plot configuration
pub struct PlotConfig {
    pub plot_type: PlotType,
    pub x_column: String,
    pub y_column: Option<String>,
    pub properties: PlotProperties,
}
```

### pika-ui
The user interface layer featuring:
- **Canvas Panel**: Infinite scrolling workspace
- **Drawing Tools**: Rectangle, circle, line, freehand
- **Data Panel**: Table preview and management
- **Properties Panel**: Node configuration
- **Context Menus**: Right-click actions

```rust
// Example: UI state management
pub struct AppState {
    pub canvas: CanvasState,
    pub selected_nodes: HashSet<NodeId>,
    pub tool: DrawingTool,
}
```

### Event System
Cross-component communication via:
- **Broadcast Channels**: Pub/sub pattern
- **Event Types**: Canvas, Data, UI events
- **Type Safety**: Strongly typed events
- **Async Handling**: Non-blocking event processing

## 📈 Data Flow Architecture (15-minute read)

### Import Pipeline
```
CSV File → Type Inference → Preview → User Config → DuckDB Import → Table Node
```

### Visualization Pipeline
```
Table Node → Column Selection → Plot Config → Data Extraction → GPU Rendering → Canvas Display
```

### Interaction Flow
```
User Input → Event Generation → Event Bus → Handler Processing → State Update → UI Refresh
```

### Memory Architecture
- **Lazy Loading**: Data loaded on demand
- **Caching**: Recently used data kept in memory
- **GPU Buffers**: Visualization data on GPU
- **Garbage Collection**: Automatic cleanup of unused nodes

## 🚀 Performance Optimizations

### Current Optimizations
1. **GPU Acceleration**: Compute shaders for aggregations
2. **Async I/O**: Non-blocking file operations
3. **Incremental Rendering**: Only redraw changed areas
4. **Data Virtualization**: Render only visible data points

### Benchmarks
- CSV Import: 1M rows in <2 seconds
- Plot Rendering: 100K points at 60 FPS
- Canvas Operations: <16ms frame time

## 🔮 Future Architecture

### Planned Improvements
1. **Plugin System**: Extensible visualizations
2. **Collaborative Editing**: Multi-user canvas
3. **Cloud Integration**: Remote data sources
4. **Mobile Support**: Touch-optimized UI

### Scalability Considerations
- **Distributed Processing**: Split large datasets
- **Progressive Loading**: Stream data as needed
- **Level-of-Detail**: Adaptive visualization quality
- **State Persistence**: Efficient workspace saving

## 📚 Related Documentation

### For Different Audiences

**For Users**:
- [README.md](../README.md) - Getting started guide
- [UI Component Guide](UI_COMPONENT_GUIDE.md) - Visual tour of features

**For Contributors**:
- [Code Quality Guide](CODE_QUALITY_GUIDE.md) - Coding standards
- [Architecture Patterns](ARCHITECTURE_PATTERNS.md) - Design decisions

**For Architects**:
- [Architecture Summary](ARCHITECTURE_SUMMARY.md) - Technical deep dive
- [Project Organization](PROJECT_ORGANIZATION.md) - Crate structure

**For Maintainers**:
- [Error Handling](ERROR_HANDLING_IMPLEMENTATION_SUMMARY.md) - Error strategy
- [UX Implementation](UX_IMPLEMENTATION_SUMMARY.md) - UI/UX decisions 