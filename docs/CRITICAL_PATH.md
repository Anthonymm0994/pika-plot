# Critical Path to Pika-Plot MVP

## 🎯 Definition of MVP

A working Pika-Plot that can:
1. Import CSV data onto the canvas
2. Execute SQL queries on nodes
3. Display results as GPU-accelerated scatter plots
4. Connect nodes to show data flow
5. Navigate with breadcrumbs

## 🚨 Blocker #1: SQL Execution Not Connected

**Current State**: Engine has stubs but doesn't actually run queries

**Fix Required**:
```rust
// In pika-engine/src/query.rs
pub async fn execute(database: &Database, sql: &str) -> Result<QueryResult> {
    let conn = database.connection()?;
    let mut stmt = conn.prepare(sql)?;
    
    // Execute and convert to Arrow
    let arrow_result = stmt.query_arrow([])?;
    
    Ok(QueryResult {
        data: Arc::new(arrow_result),
        execution_time: start.elapsed(),
        row_count: arrow_result.num_rows(),
        memory_usage: calculate_size(&arrow_result),
        execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}
```

## 🚨 Blocker #2: No GPU Plot Rendering

**Current State**: Canvas shows nodes but no actual data visualization

**Quick Win**: Port minimal scatter plot from frog-viz
```rust
// Need to copy from frog-viz/crates/dv-views/src/plots/scatter.rs
// Simplified for MVP:
1. Extract point data from QueryResult
2. Create vertex buffer with (x, y, color)
3. Use simple point shader
4. Render inside node bounds
```

## 🚨 Blocker #3: Missing Breadcrumbs

**Current State**: No context navigation

**Implementation** (30 min fix):
```rust
// Add to pika-ui/src/panels/breadcrumbs.rs
pub struct BreadcrumbBar;

impl BreadcrumbBar {
    pub fn show(&mut self, ui: &mut Ui, state: &AppState) {
        ui.horizontal(|ui| {
            // Build trail from selected node
            if let Some(selected) = state.selected_node {
                // Walk backwards through connections
                let trail = build_trail(state, selected);
                
                for (i, item) in trail.iter().enumerate() {
                    if i > 0 {
                        ui.label("→");
                    }
                    if ui.link(&item.label).clicked() {
                        state.selected_node = Some(item.node_id);
                    }
                }
            }
        });
    }
}
```

## 📋 MVP Implementation Checklist

### Day 1: Get Data Flowing
- [ ] Fix SQL execution in engine
- [ ] Add "Execute Query" button to nodes
- [ ] Display row count on success
- [ ] Show errors in status bar

### Day 2: Basic Visualization  
- [ ] Copy scatter plot basics from frog-viz
- [ ] Create GPU vertex buffer from query results
- [ ] Render points inside node rect
- [ ] Add axis labels

### Day 3: Navigation & Polish
- [ ] Implement breadcrumb bar
- [ ] Add keyboard shortcuts (Ctrl+Enter = execute)
- [ ] Color code connections by data type
- [ ] Add loading states to nodes

### Day 4: Testing & Stabilization
- [ ] Test with 1M point dataset
- [ ] Fix memory leaks
- [ ] Handle error cases
- [ ] Basic performance profiling

## 🎨 MVP Visual Requirements

### Node States:
```
┌─────────────────┐
│ 📊 sales_data   │  <- Idle (gray border)
│ 50,000 rows     │
└─────────────────┘

┌─────────────────┐
│ 📊 sales_data   │  <- Loading (pulsing blue)
│ ⏳ Executing... │
└─────────────────┘

┌─────────────────┐
│ 📊 sales_data   │  <- Success (green border)
│ ✓ 50,000 rows  │
│ [scatter plot]  │
└─────────────────┘

┌─────────────────┐
│ 📊 sales_data   │  <- Error (red border)
│ ❌ Query failed │
└─────────────────┘
```

### Connection Types:
- **Blue**: Raw data flow
- **Green**: Filtered/transformed data
- **Orange**: Aggregated data
- **Purple**: Join result

## 🚀 Post-MVP Enhancements

Once MVP works, prioritize:
1. **Performance**: GPU aggregation for 10M+ points
2. **Delight**: Smooth animations, particle effects
3. **Power**: Complex transforms, Python nodes
4. **Polish**: Export, templates, sharing

## ⚡ Quick Wins for Demo

If we need to demo before full MVP:
1. **Fake It**: Hardcode some example data
2. **Focus on Canvas**: Show dragging, connecting
3. **One Working Pipeline**: CSV → Filter → Plot
4. **Emphasize Vision**: "Imagine this with YOUR data"

## 🎯 Success Criteria

MVP is complete when:
- [ ] Can import `sales_data.csv` (1M rows)
- [ ] Can query `SELECT x, y, category FROM sales_data`
- [ ] Can see scatter plot render at 60 FPS
- [ ] Can create multi-node pipeline
- [ ] Can navigate with breadcrumbs 