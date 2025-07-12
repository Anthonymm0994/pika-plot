Got it — here's the updated and **fully comprehensive** prompt with the correct app name, **Pika-Plot**, and all requested details included:

---

````markdown
# 🧠 Help Me Architect "Pika-Plot"

I'm building a high-performance, offline-first **desktop application in Rust** called **Pika-Plot**. It's designed for **powerful and intuitive exploratory data analysis (EDA)** through a **canvas-based interface** that blends SQL querying, CSV inspection, and interactive data visualizations.

The design goal is **"from raw to refined, smooth by design"**—users shouldn’t have to think about databases or engines. They should be able to just **drop in data, explore patterns visually, and share insights easily**.

---

## 🧱 What Already Exists (Pebble Prototype)

I've already built a production-ready prototype in Rust called **Pebble**. It uses `eframe/egui` and includes:

- A **CSV import wizard** with:
  - Column-level type inference and override
  - NULL value configuration
  - Virtualized data preview
- SQLite backend with:
  - Paginated SQL query editor
  - Results grid with export to CSV/JSON
- General UI infrastructure:
  - Dark mode support
  - Modal dialogs and form validation
  - Menu system (File, Mode switching, etc.)

🧩 While the backend is currently SQLite, the **UI components and CSV import pipeline are polished and reusable**. These include:
- `CsvImportPanel`
- `ColumnConfigDialog`
- `DataPreviewTable`
- `QueryDialog`
- Export tools
- Style and layout theming

---

## 🪄 New Architecture: Pika-Plot

We're evolving from a database viewer to a **semantic canvas-based data sketchpad**. Here’s the high-level concept:

- Users drag in CSVs → configure → auto-import into DuckDB
- Each file becomes a **Table Node** on the infinite canvas
- Users create **Query Nodes** using SQL or graphical linking
- Results can be **visualized as Plot Nodes**
- Plots are **interactive, zoomable**, and must handle **10M+ rows efficiently**
- Annotations can be pinned to plots or queries
- Snapshots enable saving/sharing workspaces

### 🧠 Core Design Principles

- **Offline-first** (no internet required)
- **Zero-copy architecture** using Arrow under the hood
- **Modular Rust crates** for reusability and testability
- **Cross-platform** but optimized for **Windows 10/11**
- **Instant feedback** through semantic caching
- **Reactive dataflow** across connected nodes

---

## 🪜 Layered Architecture Overview

```
User Workspace
├── Infinite Canvas (egui)
│   ├── Table Nodes
│   ├── Query Nodes
│   ├── Plot Nodes (Scatter, Histogram, Boxplot, Line)
│   └── Annotation Nodes
│
├── Interaction Layer
│   ├── UI Event Bus
│   ├── Node Linking + Brush Linking
│   └── Canvas State Manager
│
├── Compute Layer
│   ├── SQL Parser + AST Normalizer
│   ├── Fingerprint-based Semantic Cache
│   ├── Plot Sampler/Aggregator (LTTB, binning)
│   └── DuckDB Query Engine (via ffi or subprocess)
│
└── Storage Layer
    ├── DuckDB (data and indexes)
    ├── Arrow Store (RecordBatch cache)
    └── Workspace Save/Load (snapshot format TBD)
```

---

## 🧪 Caching Strategy

We're exploring **semantic fingerprinting** for query equivalence:
- Normalize SQL AST using `sqlparser-rs`
- Fingerprint includes table names, predicates, structure
- Reuse cache for semantically identical queries
- Tiered cache (preview, query, plot) to avoid recomputing everything

---

## 📊 Plot Requirements

- Must support 1–10M+ rows interactively
- Zooming should resample or bin data (e.g., LTTB or adaptive aggregation)
- Linked brushing: highlight a point in one plot → highlight in others
- Must not block UI: updates via `tokio` threads or background tasks
- Eventually GPU acceleration (e.g., via `eframe`'s `egui_plot` or custom wgpu)

---

## 📁 Desired Module Layout

```
src/
├── main.rs
├── app/                 # Theme, routing, window lifecycle
├── canvas/              # Graph model (nodes, ports, layout)
├── ui/                  # Panels, dialogs, egui widgets
├── compute/             # SQL parsing, semantic caching, execution
├── plot/                # Sampling, binning, rendering
├── storage/             # DuckDB interface, Arrow caching
├── export/              # CSV/JSON/export snapshot
└── tests/               # Unit + integration + benchmark suites
```

---

## 💡 Special Design Considerations

- **Windows file locking**: all data access must be robust to file handle contention
- **Semantic caching**: needs AST-level normalization to detect query equivalence
- **Cross-session stability**: type overrides and fingerprints must persist
- **Memory**: DuckDB's spilling helps, but caching layer should include LRU
- **Versioned snapshots**: plan for long-term reproducibility (schema + data + layout)

---

## 🛠️ What I Need Help With

I want help templating this architecture out, designing modules and traits cleanly, and structuring the codebase so that I can let an AI coding assistant (like Cursor or Copilot) implement each module step-by-step.

Please:

1. Analyze this plan and offer improvements—don’t be dogmatic
2. Suggest any better ways to modularize or rethink flow
3. Propose a testing strategy that ensures robustness
4. Lay out the first 3–5 tasks I should delegate to begin prototyping this

Optional but appreciated:
- Annotated Rust trait examples
- A component diagram or node type interface sketch
- Notes on how to support rapid interactivity with 10M+ row plots

The goal is a **masterfully engineered**, **clean**, **testable** Rust app with fast iteration cycles and clear documentation.

Let’s build something great.
````
