# Pika-Plot Action Plan

## Executive Summary

**Current State**: 41,000 lines of overcomplicated code that doesn't work well
**Desired State**: 2,000 lines of simple code that works perfectly
**Path**: Build a minimal prototype to validate the simple approach

## The Core Problem (One Sentence)

You built a visual programming environment when all you needed was a CSV viewer with SQL and charts.

## Immediate Actions (This Week)

### Day 1-2: Prototype Test
Create `prototype/simple_plot.rs` with:
```rust
// Prove the concept in < 500 lines
// One file, three panels, basic functionality
// If this feels good, we continue
// If not, we learned cheaply
```

### Day 3: Decision Point
Based on prototype:
- **Feels right** → Continue with simple approach
- **Still complex** → Reassess what's actually needed
- **Unsure** → Try one more iteration

### Day 4-7: Build MVP (If Prototype Works)
1. Set up new clean project structure
2. Copy only essential code from current project
3. Build simple three-panel interface
4. Test with real data

## What to Build (MVP Features Only)

### Version 0.1 (This Week)
- [ ] Load single CSV file
- [ ] Display in table
- [ ] Execute SQL queries
- [ ] Show query results
- [ ] Create line chart
- [ ] Export as PNG

### Version 0.2 (Next Week)
- [ ] Multiple CSV files
- [ ] Bar charts
- [ ] Scatter plots
- [ ] Export as SVG
- [ ] Save/load queries

### Version 0.3 (Later)
- [ ] Histograms
- [ ] Join multiple tables
- [ ] Export data as CSV
- [ ] Dark/light theme

### NOT in MVP
- ❌ Node editor
- ❌ Canvas
- ❌ Drag and drop
- ❌ GPU anything
- ❌ ML features
- ❌ Real-time updates
- ❌ Collaboration
- ❌ Advanced plots (violin, radar, etc.)

## Code to Reuse

### From Current Project:
```
pika-engine/src/
  ├── import.rs (CSV loading - works great)
  ├── query.rs (SQL execution - solid)
  └── database.rs (DuckDB integration - keep it)

pika-ui/src/plots/
  ├── line_plot.rs (simplify and reuse)
  ├── bar_plot.rs (simplify and reuse)
  └── scatter_plot.rs (simplify and reuse)
```

### Start Fresh:
- Main app structure
- UI layout
- Event handling
- All the complex stuff

## Success Criteria

The new tool succeeds if:
1. **You actually use it** (most important!)
2. It never freezes or lags
3. Going from CSV to chart takes < 30 seconds
4. Code is simple enough to modify easily
5. It solves your actual problem

## Folder Structure (Simple Version)

```
pika-plot-simple/
├── Cargo.toml (minimal deps)
├── src/
│   ├── main.rs (< 200 lines)
│   ├── app.rs (< 500 lines)
│   ├── data.rs (< 300 lines)
│   ├── query.rs (< 300 lines)
│   ├── plots.rs (< 500 lines)
│   └── export.rs (< 200 lines)
├── README.md
└── examples/
    └── sales_analysis.sql
```

Total: ~2,000 lines (vs current 41,000)

## The Mantra

**Every time you want to add a feature, ask:**
1. Do I need this to analyze CSV data?
2. Will I use this every week?
3. Can I add it later if needed?

If any answer is "no", don't build it.

## Commitment

**Week 1**: Build prototype and MVP
**Week 2**: Polish and daily use testing
**Week 3**: Fix issues found during use
**Month 2+**: Only add features you've wished for 3+ times

## Emergency Escape Plan

If after 1 week the simple version isn't working:
1. Use existing tools (Excel + Python/R)
2. Consider if you really need a custom tool
3. Maybe contribute to an existing open-source project instead

## The Most Important Thing

**Ship something that works, even if it's simple.**

A working tool with 20% of features is infinitely more valuable than a broken tool with 200% of features.

---

**Next Step**: Close this document. Create `prototype/simple_plot.rs`. Start coding. Keep it simple. 