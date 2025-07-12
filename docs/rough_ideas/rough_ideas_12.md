# 🧠 Pika-Plot: Architecture Synthesis & Template Planning Prompt

## 🔷 Project Overview

You are assisting in the development of **Pika-Plot** — a high-performance, offline-first Rust desktop application for **interactive, visual data exploration**. The application is designed to run **entirely locally**, optimized for **Windows 10/11**, and is powered by **DuckDB**, **Apache Arrow**, and **Egui**.

### 🧭 Purpose

Pika-Plot is a **canvas-based visual SQL notebook** that allows users to:
- Import local datasets (primarily CSVs)
- Visually construct and link query nodes on an infinite canvas
- Preview and transform data interactively
- Generate live-updating plots with **millions of points**, optimized via binning/sampling
- Reactively propagate filters across plots, tables, and other views (e.g., brushing)
- Export annotated workspaces as portable files for sharing or archiving

This is **not** a web app. It is a **native GUI app** built in Rust using `egui`, `eframe`, and related libraries. It is entirely **offline** and must be efficient on consumer-level Windows machines. Performance, testability, and clear architecture are paramount.

---

## 🔍 Task

You will analyze all architectural and design ideas under:

```
pika-plot/docs/ideas/rough_ideas_*.md
```

These files include concept explorations, architectural breakdowns, crate suggestions, dataflow patterns, UI behaviors, memory strategies, and module design from multiple perspectives. They cover many diverging and converging approaches.

Your job is to:

- **Read and synthesize the strongest design ideas** from those files  
- **Challenge weak or redundant patterns**  
- **Propose novel, more elegant alternatives** where applicable  
- **Create a unified and production-ready architecture and module plan** for building the real application  
- **Make it directly actionable and readable by developers**

---

## ✅ What To Produce

Place your complete output into:

```
pika-plot/docs/refined_ideas/final_template_plan.md
```

This file must serve as a **master design reference** to implement the project cleanly in Cursor or any Rust editor.

Please include the following in your response:

---

### 1. 📌 High-Level Introduction
- A short paragraph or two describing what Pika-Plot is, why it exists, and what makes it different from other data tools (e.g. notebooks, dashboards, Tableau, etc.)

---

### 2. 🗂️ Proposed Crate & Module Layout
- Design a **Cargo workspace** that separates concerns (e.g. `pika-core`, `pika-canvas`, `pika-compute`, `pika-plot`, `pika-storage`, `pika-ui`, etc.)
- Each crate/module should have a short description of its role
- Include optional features or “escape hatches” if appropriate

---

### 3. 🧠 Core Traits, Systems, and Interactions
- Define key traits and modules (e.g. `QueryEngine`, `SemanticCache`, `CanvasNode`, `PlotRenderer`)
- Show how layers talk: UI ↔ cache ↔ compute ↔ DuckDB ↔ storage
- If useful, include pseudocode or concrete Rust snippets

---

### 4. 📊 Page Layouts & UI Mocks
- Describe or draw each major page in the app (you can use ASCII or markdown mockups)
  - Canvas View
  - Plot Node View
  - Table Preview View
  - Query Editor
  - Type Override Dialog
  - Export Snapshot Dialog
  - Settings / Memory Usage Viewer

- Explain how the UI behaves reactively and how different components are linked together.

---

### 5. 🚦 Message Passing / Async Flow
- How should background query execution and long-running plot jobs be handled?
- Recommend a queue/message-passing system, async task orchestration, etc.

---

### 6. 🧩 Caching + Semantic Fingerprints
- Propose the ideal caching architecture: AST normalization, viewport-aware plots, Arrow slices, etc.
- Include rules for invalidation, reuse, and deduplication

---

### 7. 📦 CSV Ingestion Strategy
- Type inference with sample preview
- Type override support via dialog
- Mapping to DuckDB schema + metadata

---

### 8. 💾 Persistence + Snapshot Format
- How should full workspaces be saved and reopened?
- Recommend a portable format (e.g. `ron` or `bincode` for metadata, `parquet` for batch data)
- Detail what exactly should be persisted

---

### 9. 📐 Performance Guidelines
- Plotting millions of rows: use sampling/batching (e.g. LTTB, binning)
- Minimize memory copying (e.g. Arc-wrapped Arrow batches)
- Memory spill strategy (e.g. DuckDB’s native disk spill)

---

### 10. 🧪 Testing & Documentation Plan
- Suggest a testing strategy:
  - What gets unit tested
  - What gets integration tested
  - Use of `insta`, `criterion`, `proptest`, etc.
- Include a recommendation for documenting the architecture (e.g. diagrams, markdown specs)

---

### 11. 📣 Opinions & Challenges
- If you find any contradictions or inefficiencies in the current rough ideas, **call them out directly**
- If there's a better design pattern, **argue for it**
- Feel free to propose new paradigms if they're **justifiably better**, even if not mentioned in the existing rough ideas

---

### Final Note

Do not hold back on ambition, but remain grounded in what’s achievable and elegant in Rust. The goal is to scaffold a **modular, testable, powerful foundation** for a real desktop data tool, not a prototype. This output will be the definitive guide used to build Pika-Plot starting immediately in Cursor.

```rust
// Reminder: The output file should go here
docs/refined_ideas/final_template_plan.md
```
