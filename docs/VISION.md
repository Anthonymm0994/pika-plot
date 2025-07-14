# Pika-Plot Vision

## Overview

Pika-Plot aims to combine the best aspects of various tools (Jupyter's interactivity, Obsidian's graph view, Excalidraw's drawing capabilities, and GPU-accelerated plotting) into a single application centered around an **infinite canvas workspace**.

## Core Design Principles

- **Canvas-First**: Everything happens on an infinite, zoomable canvas
- **Performance**: Handle gigabytes of data with GPU acceleration
- **Visual**: See your data, queries, and results directly on the canvas
- **Connected**: Link data sources, transformations, and visualizations with visual connections

## Key Features

### 1. Infinite Canvas Workspace
- Pan, zoom, and navigate like Excalidraw
- Place data tables, plots, notes, and queries anywhere
- Draw connections between elements to show data flow
- Group related items visually

### 2. Data Sources as Visual Elements
- **CSV/Parquet files** appear as table nodes on canvas
- **Live database connections** shown as database icons
- **API endpoints** displayed as service nodes
- Preview data directly in the node

### 3. Visual Query Building
- Drag tables onto canvas to start queries
- Connect tables visually to create joins
- Add filter/transform nodes between connections
- See query results update in real-time

### 4. GPU-Accelerated Plots
- **Scatter plots** with millions of points
- **Heatmaps** with real-time updates
- **3D visualizations** with smooth interaction
- **Time series** with intelligent downsampling
- All rendering at 60+ FPS

### 5. Interactive Analysis
- Click on plot points to see raw data
- Brush to select regions and filter connected plots
- Histogram of selected data appears automatically
- Statistical summaries update live

### 6. Drawing and Annotation
- Draw directly on the canvas to annotate findings
- Add arrows, shapes, and text
- Highlight important patterns in plots
- Create visual explanations of your analysis

### 7. Notebook Integration
- Drop Jupyter cells onto the canvas
- Connect data nodes to code cells
- See outputs rendered on canvas
- Mix code, data, and visualizations freely

## User Workflow Example

1. **Import Data**: Drag CSV file onto canvas → table node appears
2. **Explore**: Click table node → see data preview and statistics
3. **Query**: Draw connection to create filter node → specify conditions
4. **Visualize**: Connect filtered data to plot node → instant visualization
5. **Analyze**: Brush select interesting region → connected histogram updates
6. **Annotate**: Draw arrows pointing to anomalies, add text explanations
7. **Share**: Export canvas as interactive HTML or static report

## Technical Architecture

### Frontend (egui-based)
- Custom canvas widget with GPU-accelerated rendering
- Node-based visual programming system
- Real-time collaboration support
- Responsive design that works on tablets

### Backend (Rust)
- Arrow-based columnar data processing
- GPU compute shaders for aggregations
- Streaming query engine
- WebSocket server for real-time updates

### Data Layer
- DuckDB for SQL queries
- Polars for DataFrame operations
- Custom GPU kernels for specific operations
- Memory-mapped files for large datasets

## Inspiration Sources

### From Jupyter
- Interactive code execution
- Rich output displays
- Narrative flow

### From Obsidian
- Graph view of connections
- Bidirectional linking
- Local-first architecture

### From Excalidraw
- Infinite canvas
- Drawing tools
- Visual thinking

### From Tableau/PowerBI
- Drag-and-drop visualization
- Interactive dashboards
- Professional polish

### From Observable
- Reactive updates
- Data flow visualization
- Shareable notebooks

## Success Metrics

- Can load and visualize 1GB+ CSV files smoothly
- Plots render at 60 FPS with millions of points
- Zero learning curve for basic operations
- Power users can build complex analyses visually
- Exports work seamlessly (PNG, SVG, HTML, PDF)

## Future Possibilities

- **AI Assistant**: Natural language queries on canvas
- **Version Control**: Git-like branching for analyses
- **Cloud Sync**: Share canvases across devices
- **Plugin System**: Custom nodes and visualizations
- **Mobile App**: Touch-optimized canvas for tablets

## Design Philosophy

The canvas is not just a place to put things—it's a thinking space. Users should feel like they're sketching out ideas, with data and computation as their medium. The tool should make complex analyses feel as natural as drawing on a whiteboard, while providing the computational capabilities needed for serious data science work. 