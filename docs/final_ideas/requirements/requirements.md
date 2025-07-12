# Pika-Plot: Non-Negotiable Requirements

This document defines the hard boundaries for the Pika-Plot project. These requirements are considered fixed unless explicitly revised. All planning, design, and implementation decisions must respect them.

## Platform & Runtime

- Native **desktop application**, fully **offline-first**.
- Written entirely in **Rust** using the `egui` GUI framework.
- Must run reliably on **Windows 10 and 11**.
- **No use of web technologies**, including React, WebGL, or any browser-based runtimes.
- **No server mode**, **no collaboration features**, and **no web export** functionality.
- **File-based export and sharing only** (e.g., images, data files).

## GPU & Performance

- Target systems are assumed to have a **discrete GPU**; support for integrated graphics is not required.
- Use **GPU acceleration** aggressively for rendering plots.
- Must handle **millions of data points interactively** (zooming, panning, brushing, selections).
- Performance should be tuned for **responsiveness and speed**, not lowest-common-denominator hardware.

## UI/UX & Workflow

- **Notebook and Canvas modes** must both be available at launch.
  - Notebook mode is the default.
  - Canvas mode enables free spatial layout and interaction between components.
- Avoid "blank canvas" problems: a **guided or hybrid workspace** must be available.
- The canvas should support **infinite zoom and pan**.
- UI components (queries, plots, tables) must be **draggable, linkable**, and support **interactive brushing**.
- Plots must be **interactive** by default (not static renderings).
- Users must be able to **export**:
  - Data as CSV or JSON
  - Plots as PNG or SVG

## Storage, Data, and Backend

- **DuckDB** is the sole backend (no abstraction layer for backend swapping).
- DuckDB should be used for:
  - Type inference
  - Query execution
  - Metadata management
  - Snapshot recipes
- Existing code from other projects should be **reused where appropriate**:
  - `pebble`: useful for SQLite-based egui database browsing (UI/layout ideas)
  - `frog-viz`: includes plotting, interactivity, and visualization logic
- **Do not use SQLite** in the new system.
- Snapshots should store **references to queries or transformations**, not raw embedded data.

## Architecture & Code Quality

- The application must be delivered as a **complete system**, not in staged or incremental versions (no phased v0.1/v0.2 plans).
- The architecture should prioritize **ambitious, well-reasoned decisions** over minimal MVPs.
- Code must be:
  - **Modular**
  - **Testable**
  - Designed for clear API boundaries

## Documentation & Project Planning

- All finalized planning documents must reside in `docs/final_ideas/`
- Use `docs/final_ideas/planning/questions.md` to capture any open design questions or unknowns.
- Follow the structure and expectations laid out in `docs/final_ideas/planning/project_management_guide.md`.
- All planning should reflect a **vision-driven**, ambitious approach.

## Command-Line & Testability

- Provide a set of **public Rust APIs or CLI-accessible functions** to:
  - Run ingestion logic
  - Trigger and validate queries
  - Initiate cache or snapshot operations
- A **command-line workflow for validation** and inspection is encouraged.
- **Unit testing** remains important and should complement CLI-driven workflows. 