# Technical Questions and Insights - Round 6: Radical Simplification

## Overview

After reviewing Rerun's impressive capabilities and understanding the actual deployment scenario (5-10 machines, some offline, unknown GPUs), we need to radically simplify our approach. This document focuses on practical solutions that work reliably on modest hardware.

## Key Insights from Rerun

Rerun achieves amazing visualization with:
- Simple `rr.log()` API - no complex setup
- Single viewer that handles all data types
- Time-aware by default
- Works on any hardware (CPU rendering fallback)
- No need to know GPU specs upfront

## 1. Simplified Architecture (Priority: CRITICAL)

**Assigned to: All Agents**

### The New Reality

We've been overengineering. Let's build something that:
- Works on ANY Windows 10/11 machine
- Doesn't require specific GPUs
- Handles offline scenarios gracefully
- Is dead simple to use

### Questions to Answer

1. **Should we abandon GPU aggregation entirely?**
   - Rerun does impressive visualization without requiring GPU compute
   - Could we use CPU-based aggregation with smart caching?
   - Is WebGPU (which works everywhere) sufficient?

2. **Should we use a simpler data model?**
   ```rust
   // Instead of complex node graphs, maybe just:
   pub enum DataSource {
       File(PathBuf),
       Query(String),
   }
   
   pub enum Visualization {
       Scatter { x: String, y: String },
       Line { x: String, y: String },
       Heatmap { x: String, y: String, z: String },
   }
   ```

3. **Can we eliminate the 5-crate structure?**
   - Maybe just 2 crates: `pika` (library) and `pika-cli`
   - Embed UI in the library
   - Single binary distribution

### Deliverable Needed
```rust
// File: docs/simplified_architecture.md
// A complete rethink of the architecture focusing on:
// - Simplicity over performance
// - Reliability over features
// - Working on ANY machine
```

## 2. Data Handling Without Complexity (Priority: HIGH)

**Assigned to: Gemini 2.5 Pro**

### The Problem

We don't need to handle petabytes. We need to handle the actual data these 5-10 machines will process.

### Simplified Approach

1. **Forget Arrow complexity**
   ```rust
   // Just use DuckDB directly
   // No arrow re-exports, no version conflicts
   pub fn query_to_columns(conn: &Connection, sql: &str) -> Result<DataFrame> {
       // Simple columnar format, no Arrow needed
   }
   ```

2. **Simple streaming**
   ```rust
   // Don't overthink it
   pub fn stream_csv(path: &Path, chunk_size: usize) -> impl Iterator<Item = Chunk> {
       // Basic CSV reader with chunking
   }
   ```

3. **In-memory when possible**
   - Most datasets will fit in RAM on modern machines
   - Only stream when file > available RAM

### Deliverable Needed
Working data layer that just uses DuckDB without Arrow complications.

## 3. Visualization That Just Works (Priority: HIGH)

**Assigned to: All Agents**

### Learning from Rerun

Rerun's viewer works because:
- It doesn't try to be perfect
- CPU rendering is fine for most cases
- Simple is better than optimal

### Questions

1. **Should we use egui's built-in plot library?**
   ```rust
   // egui_plot is simple and works everywhere
   ui.plot("my_plot")
       .scatter(points)
       .show();
   ```

2. **Do we even need wgpu directly?**
   - egui can render plots
   - Only use wgpu if we need 3D or millions of points
   - Start simple, optimize later

3. **What about offline machines?**
   - Bundle everything
   - No external dependencies
   - Single executable

### Deliverable Needed
```rust
// File: pika/src/viz/simple_plots.rs
// Dead simple plotting that works on any machine
// No GPU required, no complex setup
```

## 4. Forget Profiling Infrastructure (Priority: LOW)

**Assigned to: Nobody**

### The Reality

With 5-10 machines, you don't need automated profiling infrastructure. You need:
- It to work
- It to not crash
- Basic timing info when debugging

### What We Actually Need

```rust
// This is enough:
let start = Instant::now();
let result = do_something();
println!("Took: {:?}", start.elapsed());
```

## 5. Installation and Distribution (Priority: HIGH)

**Assigned to: GPT-4.5 & Grok 4**

### The Problem

These machines might not have internet, might have IT restrictions, might be air-gapped.

### Questions

1. **Single binary distribution?**
   ```bash
   # This should be the entire install process:
   Copy pika.exe to machine
   Double-click
   ```

2. **How to handle DuckDB?**
   - Bundle it statically?
   - Use rusqlite instead?
   - What's simplest?

3. **Config files?**
   - Maybe no config files
   - Everything through UI
   - Save preferences in user directory

### Deliverable Needed
Distribution strategy that works on locked-down corporate Windows machines.

## 6. Error Handling for Humans (Priority: MEDIUM)

**Assigned to: Any Agent**

### The Users

These aren't developers. They're analysts on specific machines. They need:
- Clear error messages
- Obvious solutions
- No stack traces

### Examples

```rust
// Instead of: "Arrow type mismatch in schema evolution"
// Show: "The CSV file has a different format than expected. 
//        Check that all files have the same columns."

// Instead of: "GPU buffer allocation failed"
// Show: "Not enough graphics memory. Try closing other programs 
//        or reducing the data size."
```

### Deliverable Needed
Error message guidelines and implementation.

## 7. Testing on Actual Hardware (Priority: HIGH)

**Assigned to: User (not AI agents)**

### The Reality

We can't test on every possible configuration. We need:
- List of actual machines this will run on
- Their specs (even approximate)
- What data sizes they typically handle

### Questions for User

1. What are the typical data sizes? (MB? GB?)
2. Are these laptops or desktops?
3. Do they have discrete GPUs at all?
4. What's the oldest machine?
5. Are they domain-joined with IT restrictions?

## 8. MVP Feature Set (Priority: CRITICAL)

**Assigned to: All Agents**

### What Do Users Actually Need?

Not "everything possible" but what they'll use daily:

1. **Load CSV/Parquet**
2. **Run SQL queries**
3. **Make basic plots** (scatter, line, bar)
4. **Save plots as images**
5. **Save/load sessions**

That's it. Everything else is nice-to-have.

### Deliverable Needed
```rust
// File: docs/mvp_spec.md
// Exactly what features go in v1.0
// No more, no less
```

## Key Principles Going Forward

1. **It should work on a 5-year-old laptop with integrated graphics**
2. **No configuration required**
3. **Single binary, no installation**
4. **If Rerun can do it simply, so can we**
5. **CSV in, plots out - that's the core**

## Conclusion

We've been solving problems that don't exist. The real challenges are:

1. Making it work reliably on unknown hardware
2. Being simple enough that non-developers can use it
3. Working offline
4. Not crashing

Let's build something simple that works, not something complex that might work.

Agents should focus on:
- **Removing complexity**, not adding it
- **Using boring, proven tech** (SQLite? CSV? egui plots?)
- **Making it foolproof**
- **Learning from Rerun's simplicity**

Remember: These 5-10 users just want to visualize their data. They don't care about our clever architecture. 