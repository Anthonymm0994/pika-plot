# Pika-Plot Restructuring Plan

## Current Situation

### What We Have
- **41,787 lines of code** across 135 files
- 6 separate crates with complex dependencies
- Node-based canvas UI that's slow and overcomplicated
- Many experimental features (GPU, ML, streaming, etc.)
- Working core components (data import, SQL queries, basic plotting)

### Main Problems
1. **Performance**: Canvas UI is sluggish and unresponsive
2. **Complexity**: Drag-and-drop nodes are overkill for the use case
3. **Scope Creep**: Too many features, lost focus on core functionality
4. **Technical Debt**: 133 compilation errors in UI layer
5. **User Experience**: Current interface is confusing and slow

## Core Purpose (What We Actually Need)

A simple tool that:
1. Imports CSV files
2. Runs SQL queries on the data
3. Creates basic visualizations
4. Exports plots and data

That's it. Everything else is optional.

## Options Assessment

### Option 1: Incremental Cleanup (Not Recommended)
- Fix the existing UI compilation errors
- Remove unused features one by one
- Refactor the canvas to be faster

**Pros**: Preserves all work
**Cons**: Will take weeks, might still be complex, performance issues remain

### Option 2: Fresh UI, Keep Backend (Recommended)
- Archive the current `pika-ui` crate
- Create new simple `pika-ui-simple` with minimal interface
- Reuse working `pika-core` and `pika-engine`
- No canvas, no nodes, just panels

**Pros**: Fast to implement, keeps working code, clean slate for UI
**Cons**: Some UI work is "lost" (but it's not working anyway)

### Option 3: Complete Rewrite
- Start a new project from scratch
- Only implement essential features
- Learn from current implementation

**Pros**: Cleanest approach, no legacy code
**Cons**: Throws away working components, takes longest

## Recommended Path: Option 2

### Phase 1: Preparation (Today)
1. Create feature branch: `git checkout -b simplified-ui`
2. Archive current UI: `mv pika-ui pika-ui-archived`
3. Create new minimal UI crate: `cargo new pika-ui-simple --lib`

### Phase 2: Minimal UI Implementation (Week 1)
Build a simple three-panel interface:
```
┌─────────────┬─────────────────┬──────────────┐
│ Data Tables │ SQL Query       │ Plot Preview │
│             │ ┌─────────────┐ │              │
│ sales.csv   │ │SELECT * FROM│ │   [Chart]    │
│ users.csv   │ │sales LIMIT 5│ │              │
│             │ └─────────────┘ │              │
│             │ Results:        │ Plot Type:   │
│ [+] Import  │ [Table View]    │ [⚬] Line     │
└─────────────┴─────────────────┴──────────────┘
```

### Phase 3: Core Features (Week 2)
1. CSV import (reuse from `pika-engine`)
2. SQL execution (already works)
3. 4 plot types: Line, Bar, Scatter, Histogram
4. Export to PNG/SVG

### Phase 4: Polish & Release (Week 3)
1. Keyboard shortcuts
2. Save/load workspace
3. Basic themes
4. Documentation

## What to Keep

### From pika-core:
- Data structures
- Type definitions
- Basic workspace management

### From pika-engine:
- `import.rs` - CSV importing
- `database.rs` - DuckDB integration
- `query.rs` - SQL execution
- `plot/renderer.rs` - Basic plot rendering

### From pika-ui (selectively):
- Plot type implementations (simplified)
- Export functionality
- Color schemes

## What to Remove/Archive

### Definitely Remove:
- Node-based canvas system
- Drag and drop functionality
- GPU acceleration
- ML/Neural network features
- Streaming/real-time features
- Collaboration features

### Maybe Keep (Decide Later):
- Advanced plot types (box, violin, radar)
- Data transformation nodes
- Visual query builder

## Success Criteria

The new version should:
1. **Start in < 1 second**
2. **Import a 1MB CSV in < 2 seconds**
3. **Run queries with instant feedback**
4. **Create plots with 2-3 clicks**
5. **Use < 100MB RAM for typical workflows**

## Implementation Checklist

- [ ] Create feature branch
- [ ] Archive current UI
- [ ] Set up new UI crate
- [ ] Implement main window layout
- [ ] Add data import panel
- [ ] Add SQL query editor
- [ ] Add plot configuration
- [ ] Implement Line plot
- [ ] Implement Bar plot  
- [ ] Implement Scatter plot
- [ ] Implement Histogram
- [ ] Add PNG export
- [ ] Add SVG export
- [ ] Add keyboard shortcuts
- [ ] Write user documentation
- [ ] Performance testing
- [ ] Release v0.2.0

## Alternative: Quick Prototype First

Before committing to the full restructure, we could:
1. Create a `prototype/` directory
2. Build a minimal version in ~500 lines
3. Test if the simple approach meets needs
4. Then decide whether to refactor or restart

## Next Step

**Decision needed**: Should we:
1. Start the restructuring on a branch?
2. Build a quick prototype first?
3. Try fixing the current UI instead?

The key is to **act fast** and **stay focused** on the core use case. 