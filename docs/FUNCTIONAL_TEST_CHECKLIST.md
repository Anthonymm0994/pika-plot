# Canvas Functionality Test Checklist

## Core Functionality Tests

### ✅ Node Operations
- [x] Create nodes via right-click context menu
  - [x] Add Note
  - [x] Add Shape (Rectangle, Circle)
  - [x] Add Data Source (from available tables)
- [x] Select nodes by clicking
- [x] Drag nodes to move them
- [x] Resize nodes by dragging corners (all 4 handles)
- [x] Delete nodes via context menu
- [x] Deselect nodes by clicking empty space

### ✅ Shape Drawing
- [x] Rectangle tool creates rectangles with live preview
- [x] Circle tool creates circles with live preview
- [x] Shapes show selection boxes when selected
- [x] Shapes preview immediately during drag (not just on release)

### ✅ Data Source Nodes
- [x] Display table name in header
- [x] Show data preview (headers and rows)
- [x] Query editor with visual cursor
- [x] "Ctrl+Enter to run" hint
- [x] Alternating row colors in preview
- [x] Limited rows/columns for performance

### ⚠️ Connection System
- [ ] Create connections between nodes
- [ ] Bezier curve rendering for wires
- [ ] Connection type visualization (DataFlow, Transform, Join)
- [ ] Delete connections
- [ ] Validate connection compatibility

### ⚠️ Plot Nodes
- [ ] Create plots from table nodes via context menu
- [ ] All 10 plot types available
- [ ] Plot preview rendering
- [ ] Data flows from table to plot
- [ ] Multiple plots per data source

### ✅ Canvas Navigation
- [x] Pan with middle mouse button
- [x] Zoom with scroll wheel
- [x] Grid display (toggleable)
- [x] Grid caching for performance

### ✅ UI/UX Features
- [x] Right-click context menus work everywhere
- [x] Double-click to add data sources from panel
- [x] Larger default sizes (400x300 tables, 350x250 plots)
- [x] Professional dark theme styling
- [x] Responsive interaction (no lag)

### ⚠️ Query System
- [ ] Edit queries in table nodes
- [ ] Execute queries with Ctrl+Enter
- [ ] Update connected plots on query execution
- [ ] Error handling for invalid queries

### ❌ Missing Features
- [ ] Undo/Redo system
- [ ] Copy/Paste nodes
- [ ] Node templates/presets
- [ ] Group selection (box select)
- [ ] Export canvas as image/JSON
- [ ] Import saved canvases
- [ ] Keyboard shortcuts
- [ ] Node alignment tools
- [ ] Connection routing options
- [ ] Node locking

## Performance Tests

### ✅ Optimization Features
- [x] Frustum culling (only visible nodes rendered)
- [x] Grid caching
- [x] Spatial indexing infrastructure
- [x] Reduced allocations in render loop
- [x] Frame tracking for adaptive updates

### Performance Benchmarks
- [ ] 100+ nodes at 60 FPS
- [ ] 1000+ nodes with smooth pan/zoom
- [ ] Large data previews (10k+ rows)
- [ ] Complex connection networks

## Integration Tests

### Data Pipeline
- [ ] CSV import → Table node → Query → Plot
- [ ] Multiple data sources → Join node → Combined plot
- [ ] Real-time data updates
- [ ] Export results

### Persistence
- [ ] Save canvas state
- [ ] Load saved canvases
- [ ] Auto-save functionality
- [ ] Version compatibility

## Edge Cases

### Error Handling
- [ ] Invalid file paths
- [ ] Malformed CSV data
- [ ] SQL syntax errors
- [ ] Missing data connections
- [ ] Circular dependencies

### UI Edge Cases
- [ ] Nodes outside viewport
- [ ] Overlapping nodes
- [ ] Very long node names/queries
- [ ] Extreme zoom levels
- [ ] Rapid interaction sequences

## Recommendations for Improvement

### 1. Consider egui-snarl Integration
- Better wire rendering with customizable bezier curves
- Built-in serialization support
- More sophisticated pin system
- Better performance for large graphs

### 2. Implement Core Missing Features
- Undo/Redo system using Command pattern
- Copy/Paste with clipboard integration
- Multi-select with box selection
- Node templates for common patterns

### 3. Enhance Data Flow
- Visual data flow indicators
- Progress indicators for long queries
- Caching system for query results
- Streaming data support

### 4. Improve Developer Experience
- Better error messages
- Comprehensive logging
- Performance profiling tools
- Unit tests for canvas operations

### 5. Advanced Features
- Custom node types via traits
- Plugin system for extensions
- Collaboration features
- Version control integration 