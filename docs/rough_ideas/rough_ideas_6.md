Here’s a thorough, comprehensive breakdown of everything we’ve discussed so far regarding the design and architecture of **The Data Sketchpad**, your modular Rust-based data exploration tool:

---

## 🔧 Core Product Vision

**The Data Sketchpad** is a canvas-based, highly interactive desktop app for visual data exploration. It targets analysts, engineers, and domain experts who want to:

* Drag and drop CSVs or datasets.
* Clean, query, and visualize data with minimal friction.
* Share annotations, insights, and interactive snapshots.
* Run entirely offline on Windows 10/11 (no external dependencies).

Your main goal is **zero-friction EDA** (exploratory data analysis) with a modular, testable Rust backend and polished UI. Users shouldn’t have to think about the internals.

---

## 🧱 High-Level Architecture Overview

### 1. **Backend Data Engine**

* **Database**: DuckDB-only (SQLite dropped for simplicity).

  * Ingests CSVs with type inference and user overrides.
  * Stores data, query results, and type metadata.
  * Supports snapshots via RON + Arrow or Parquet.
* **Caching**: Unified semantic cache system.

  * Based on normalized SQL AST + schema + viewport.
  * Caches Preview, Query, and Plot artifacts using enum variants.
  * Uses `Arc<RecordBatch>` for zero-copy sharing.
* **Memory**:

  * Automatic spilling to disk using DuckDB.
  * Optional UI warnings for memory pressure.
  * Lightweight eviction via LRU or usage stats.

### 2. **Frontend UI and Interactivity**

* Built in `egui`, using a canvas-style layout.
* Infinite zoomable workspace (Painter-based).
* Nodes for:

  * CSV Tables
  * SQL Query Panels
  * Interactive Plots
  * Text Annotations
* Draggable links connect nodes (brushing, linking, filtering).
* Reactive update model with DAG-based dependency resolution.

### 3. **Snapshot & Export System**

* Export snapshot = Workspace metadata (RON) + Cached Arrow or Parquet batches.
* Used for reproducible state, sharing insights offline, or loading sessions.
* Designed to be portable and deterministic.

---

## 📁 Recommended Project Structure

```shell
src/
├── main.rs
├── app/                 # Startup, configuration, state
├── backend/             # DuckDB ingestion, type overrides, queries
├── cache/               # Unified semantic cache
├── canvas/              # Node system, layout, interaction
├── compute/             # SQL parsing, normalization, execution
├── plot/                # Sampling, binning, rendering
├── storage/             # Snapshots, file loading, export
├── ui/                  # egui widgets, panels, dialogs
└── tests/               # Unit and integration tests
```

---

## 🧠 Core Innovations and Features

### 🧩 Semantic Cache

* Fingerprints composed of:

  * Normalized SQL AST
  * Referenced schema versions
  * Viewport hash (if applicable)
* Enables smart reuse: e.g., “WHERE x > 100” and “WHERE x>100.0” reuse the same cache entry.
* Potential for superset/subset lookup for cache reuse on more general filters.

### 🧠 Plot System

* Viewport-aware: Samples or bins data based on current zoom.
* Brushing in plots updates linked queries or other plots.
* Rendering is done via `egui_plot` or a custom trait-based renderer.

### 🧠 Canvas Node Graph

* Nodes represent Query, Table, Plot, or Annotation objects.
* Edges represent dependencies, e.g., Query feeds into Plot.
* Live updates propagate through a dependency graph.
* Layout engine prevents overlapping or erratic jumps.

---

## 🖼️ Diagrams & UI Flow

### Module Diagram

```
[Canvas]
  ├── [Table Node] ──▶ [Query Node] ──▶ [Plot Node]
  │                      │                │
  │                      ▼                ▼
  │                 [Semantic Cache]   [Plot Cache]
  │
  ▼
[Snapshot / RON + Arrow]
```

### Typical User Flow

1. **Drop a CSV**

   * Auto-ingested by DuckDB
   * Table Node appears

2. **Create a Query Node**

   * User writes SQL
   * Cache is checked via semantic fingerprint

3. **Drag to Plot**

   * Plots adaptively sample data
   * Brushing allows reverse filtering

4. **Annotate**

   * User pins notes to filters, charts, or data slices

5. **Snapshot**

   * RON for metadata
   * Arrow/Parquet for batch data

---

## 🧪 Testing Strategy

### Pyramid Structure

```
E2E Tests: 5%
  - Load CSV → Query → Plot → Export

Integration Tests: 20%
  - SQL parse + cache
  - Plot render + brushing

Unit Tests: 75%
  - AST normalization
  - Cache hashing
  - Plot binning/sampling
  - Snapshot I/O
```

---

## 🧰 Crate and Tech Stack Notes

* `duckdb`: local analytical DB
* `polars`: optional for sampling, transformations
* `arrow`: RecordBatch memory model
* `sqlparser`: for AST parsing
* `ron`: workspace and config serialization
* `egui`: UI framework
* `tokio`: background threads for async compute
* `dashmap`: concurrent cache
* `parquet`: snapshot export (optional)
* `bumpalo`: optional memory allocator for efficiency (future optimization)

---

## 💡 Path Forward: Templating for Cursor

You’ll want to:

1. **Write down the above architectural vision** in a Markdown file.
2. **Stub out the folder structure** in a Rust project using Cargo workspaces.
3. **Create minimal skeletons** for each module (`mod.rs`, trait files).
4. **Write a master README or DEV\_GUIDE.md** explaining:

   * Module boundaries
   * Example data flow
   * Testing policy
   * Coding conventions
   * Snapshot & export spec
5. **Hand off to Cursor** with a pinned system prompt like:

```markdown
You are my co-architect. Help implement the workspace as defined in this architecture.
Each module should be testable, Rust idiomatic, and follow this design:

<insert summary or point to file>

Start with <module name> and generate interfaces and scaffolding.
```

---

Would you like help generating a full `README.md`, `DEV_GUIDE.md`, or initial Cargo workspace with the folder layout above?
