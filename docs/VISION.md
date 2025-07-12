# Pika-Plot Vision & Architecture

## 🎯 Core Vision: "Excalidraw for Gigabytes of Data"

Pika-Plot is an advanced data visualization tool that fuses the best of **pebble** (type inference & configuration) and **frog-viz** (GPU-accelerated plotting) into a single, powerful application centered around an **infinite canvas workspace**.

### Key Principles
- **Intuitive**: Drag-and-drop canvas interface, visual connections
- **Powerful**: Handle gigabytes of data with GPU acceleration
- **Beautiful**: Polished UI with delightful interactions
- **Fast**: 60 FPS with 1M–50M points
- **Offline**: No internet dependencies, works on Windows 10/11

## ✅ UI Paradigm

### 1. Infinite Canvas Workspace
**Status**: ✅ Implemented in `pika-ui/src/panels/canvas.rs`
- Pan with middle mouse button
- Zoom with scroll wheel
- Grid background for spatial reference
- Nodes can be dragged and positioned freely

### 2. Thread System (Visual Connections)
**Status**: ✅ Basic implementation complete
- Color-coded connections between nodes
- Bezier curves for smooth visual flow
- Connection types: DataFlow, Transform, Join
- **TODO**: Color coding by data type/connection type

### 3. Breadcrumb Trails
**Status**: ❌ Not yet implemented
- **TODO**: Add breadcrumb bar showing: Table → Query → Plot
- Should update based on selected node
- Click to navigate back through pipeline

## 🚀 Technical Architecture

### From pebble (Reused/Enhanced):
- ✅ Type inference logic (`ImportOptions.infer_schema`)
- ✅ File configuration dialog (`FileImportDialog`)
- ✅ CSV import with delimiter detection
- ✅ Clean UI patterns and theme system

### From frog-viz (Reused/Enhanced):
- ✅ Tokio async runtime architecture
- ✅ Event-driven UI<->Engine communication
- ✅ Multi-crate structure (core, engine, ui)
- ✅ GPU manager with memory tracking
- ❌ TODO: Port scatter/line plot implementations
- ❌ TODO: Port time series navigation

### New in Pika-Plot:
- ✅ Node-based canvas (like Excalidraw)
- ✅ Memory coordinator (unified RAM/VRAM management)
- ✅ DuckDB integration for SQL queries
- ✅ Streaming data architecture
- ❌ TODO: GPU-accelerated aggregation
- ❌ TODO: Smart caching with cost-based eviction

## 📊 Performance Targets

### Current Status:
- ✅ GPU device validation and memory tracking
- ✅ Streaming architecture for large datasets
- ✅ Memory coordinator prevents OOM
- ❌ TODO: Benchmark with 1M+ points
- ❌ TODO: Level-of-detail rendering
- ❌ TODO: GPU instancing for massive datasets

### Goals:
- 60 FPS with 1-5M points (scatter/line)
- 30 FPS with 5-50M points (aggregated)
- < 100ms query response for cached data
- < 1s for uncached queries on GB datasets

## 🎨 Delight & Polish Features

### Implemented:
- ✅ Smooth pan/zoom on canvas
- ✅ Node hover effects
- ✅ Progress indicators
- ✅ Memory usage visualization

### TODO:
- ❌ "Spark" gesture for quick plots
- ❌ Lasso select for data points
- ❌ Data lighthouse (minimap)
- ❌ Animated transitions
- ❌ Sound effects for actions
- ❌ Keyboard shortcuts overlay

## 🧪 Testing Strategy

### Current:
- ✅ Unit tests for core types
- ✅ Integration tests for engine
- ❌ TODO: UI snapshot tests
- ❌ TODO: Performance benchmarks
- ❌ TODO: Memory leak tests

### Needed:
1. GPU fallback testing (software renderer)
2. Large dataset stress tests
3. Multi-node pipeline tests
4. Export/import round-trip tests

## 🔧 Implementation Roadmap

### Phase 1: Foundation (Current)
- ✅ Basic architecture setup
- ✅ Canvas with nodes
- ✅ Import CSV/Parquet
- ✅ Memory management
- ⏳ Basic SQL queries

### Phase 2: Visualization
- [ ] Port scatter plot from frog-viz
- [ ] Port line plot from frog-viz
- [ ] Implement heatmaps
- [ ] GPU-accelerated rendering
- [ ] Level-of-detail system

### Phase 3: Polish
- [ ] Breadcrumb navigation
- [ ] Minimap/lighthouse
- [ ] Keyboard shortcuts
- [ ] Export system
- [ ] Performance optimizations

### Phase 4: Advanced
- [ ] Custom transforms
- [ ] Python integration
- [ ] Collaborative features
- [ ] Plugin system

## 🚨 Risk Areas & Mitigations

### GPU Compatibility
**Risk**: Not all Windows machines have discrete GPUs
**Mitigation**: 
- ✅ Fallback to CPU rendering
- ✅ Mock GPU for testing
- TODO: Software rasterizer option

### Memory Management
**Risk**: OOM with large datasets
**Mitigation**:
- ✅ Central memory coordinator
- ✅ Cost-based eviction
- ✅ Streaming architecture
- TODO: Disk-based overflow

### Performance
**Risk**: Slow with millions of points
**Mitigation**:
- TODO: GPU aggregation shaders
- TODO: Spatial indexing
- TODO: Progressive rendering

## 🤝 Cross-Agent Delegation

Questions for specialized agents:

### GPU Agent:
1. Optimal vertex layout for 50M+ points?
2. Aggregation shader strategies?
3. DX11 vs DX12 tradeoffs?

### UI/UX Agent:
1. Best practices for infinite canvas?
2. Touch gesture support?
3. Accessibility considerations?

### Performance Agent:
1. Async scheduling strategies?
2. Cache-friendly data layouts?
3. SIMD opportunities?

## 📝 Key Differentiators

What makes Pika-Plot special:
1. **Canvas-first**: Not just plots in tabs, but spatial workspace
2. **SQL-native**: DuckDB integration for complex queries
3. **GPU-powered**: Not just for rendering, but computation
4. **Offline-first**: No cloud dependencies
5. **Windows-optimized**: Native performance on Windows

## 🎯 Success Metrics

We'll know we've succeeded when:
1. Users can explore GB datasets at 60 FPS
2. Creating visualizations feels like sketching
3. Complex pipelines are self-documenting via canvas
4. Performance rivals specialized tools
5. It "just works" on any Windows machine 