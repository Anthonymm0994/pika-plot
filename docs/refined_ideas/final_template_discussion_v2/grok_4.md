# Assessment for Automating Pika-Plot Implementation

This assessment focuses exclusively on enhancing the automatable aspects of `final_template_plan_v2.md` to enable a powerful dev agent (e.g., in Cursor) to execute the full scope confidently. It identifies precise additions needed for scaffolding, highlights underdefined elements that could block progress, and suggests structural enhancements to the implementation environment. All suggestions align with the plan's hard constraints: full features from day one (dual UI modes, GPU acceleration, offline-first, Windows targeting), reuse of existing SQLite viewer and plot codebases, and no simplifications or deferrals.

I've incorporated insights from recent web search results (2025-07-11) on egui plotting libraries to inform specs for GPU integration with egui, citing them via domain-named markdown links (e.g., [github.com](https://github.com/emilk/egui_plot) for egui_plot's immediate-mode plotting utilities, which can be adapted for the plan's `egui::PaintCallback` approach).

## Additional Documents, Diagrams, Specifications, or Design Clarifications Needed

To make the plan fully scaffolded for an agent, add these artifacts as separate Markdown/PNG files in a `docs/scaffolding/` directory. They provide unambiguous blueprints, reducing ambiguity in code generation.

- **Detailed UI Wireframes and Interaction Specs**: Create annotated diagrams for both WorkspaceModes (Notebook and Canvas), including pixel-precise layouts for elements like Node rendering, mode toggles, and export dialogs. Specify egui interactions (e.g., drag-to-reorder in Notebook, pan/zoom in Canvas) with event sequences. Include specs for shared features like keyboard shortcuts (e.g., Ctrl+S for export). This clarifies `pika-ui` implementations, preventing agent guesswork on visuals.

- **Sequence Diagrams for Event Flows**: UML-style diagrams showing end-to-end flows for key scenarios, such as CSV import → Query execution → Plot rendering → Export. Cover both UI modes and error paths (e.g., MemoryMonitor triggering `Error::InsufficientMemory`). Use tools like Mermaid for embeddable diagrams. This explicates the event-driven architecture, ensuring agents handle async channels correctly.

- **Entity Relationship and Data Model Diagrams**: ER diagrams detailing structs like `QueryResult`, `WorkspaceSnapshot`, and `DataSourceRef`, including field types, relationships (e.g., NodeId foreign keys), and serialization formats (e.g., RON for snapshots). Specify Arrow schema mappings for DuckDB results. This aids `pika-core` and `pika-engine` automation.

- **GPU Pipeline Specifications**: A step-by-step spec document for `GpuPlotRenderer`, including shader code snippets (WGSL for wgpu pipelines) for direct, instanced, and aggregated rendering. Define input/output formats (e.g., vertex buffers from PlotData) and integration points with egui via `PaintCallback` [docs.rs](https://docs.rs/egui_plot/latest/egui_plot/struct.Plot.html) (leveraging egui_plot's utilities for coordinate formatting and grid spacing). Include reuse instructions for existing plot codebase (e.g., adapt scatter/line renderers to wgpu pipelines).

- **Error Handling and Recovery Specs**: A catalog of all error variants (e.g., in `pika-core`), with recovery strategies (e.g., UI toasts for `InsufficientMemory`, retry logic for DuckDB locks on Windows). This ensures agents implement robust, mode-agnostic handling without inventing patterns.

- **Reuse Integration Guide**: A document mapping components from the existing SQLite viewer (e.g., CSV import UI) and plot codebase (e.g., sampling algorithms) to new structs. Specify adaptations, like migrating SQLite queries to DuckDB in `pika-engine`.

These additions make the plan a complete "blueprint" pack, allowing agents to generate code module-by-module without needing clarifications.

## Missing Pieces That Would Prevent Confident Execution

Several elements in the plan are high-level or implicit, which could cause an agent to halt or produce inconsistent code. Address these to enable end-to-end automation.

- **Exact Dependency Lists and Cargo.toml Templates**: Provide pre-filled Cargo.toml files for each crate, including versions for crates like `duckdb`, `wgpu`, `egui`, `moka` (for LRU cache), `tokio`, `arrow`, and `egui_plot` [github.com](https://github.com/emilk/egui_plot) (for plot utilities like `CoordinatesFormatter`). Specify features (e.g., wgpu's "dx12" backend for Windows) and minimum Rust version (e.g., 1.78+ for stable async traits).

- **Test Fixtures and Benchmark Data**: Define a set of sample datasets (e.g., small/medium/large CSVs in a `fixtures/` directory) for testing, including expected outputs (e.g., Arrow schemas post-import). Specify benchmark thresholds (e.g., aggregation <100ms for 50M points) to validate performance tests.

- **Windows-Specific Configurations**: Detail build scripts or configs for Windows (e.g., handling file paths in `DataSourceRef`, wgpu device selection for discrete GPUs). Include a setup script for agent environments (e.g., installing DirectX for wgpu).

- **Node Type Implementations Blueprint**: Expand on the `Node` trait with per-type specs (e.g., `TableNode` must include virtual scrolling [docs.rs](https://docs.rs/egui_plot/latest/egui_plot/), `PlotNode` must support brushing via egui responses). Clarify how nodes share state across modes (e.g., serializing `CanvasNode` to `NotebookCell` during toggles).

- **Snapshot Loading/Validation Logic**: Specify algorithms for validating `DataSourceRef` (e.g., hash checks on reload) and handling mismatches (e.g., prompt re-import).

- **Performance Metrics and Validation Criteria**: Define success metrics for agents (e.g., "render 10M points at 60FPS on discrete GPU") with profiling hooks (e.g., using `tracy` for frame tracing).

Without these, agents might misconfigure dependencies or produce untestable code, blocking full execution.

## Suggestions for Structuring the Implementation Environment

To optimize for agent delegation, structure the repo for modular, parallel tasking. This builds on the 4-crate layout, adding entry points and queues for automation.

- **Crate Layout Enhancements**: In each crate, add a `src/lib.rs` with public exports and a `examples/` subdirectory for standalone demos (e.g., `pika-engine/examples/query_bench.rs`). Use a workspace-level Cargo.toml with `[workspace.dependencies]` for shared crates (e.g., `wgpu = "0.20"`). Include a `build.rs` in `pika-app` for Windows manifests (e.g., GPU requirements).

- **Initial Entry Points**: 
  - `pika-app/src/main.rs`: Skeleton with egui window setup, event loop, and mode toggling.
  - `pika-core/src/lib.rs`: Define all enums/structs first (e.g., `AppEvent`, `WorkspaceMode`).
  - `pika-engine/src/engine.rs`: Entry for DuckDB connection pooling.
  - `pika-ui/src/workspace.rs`: Central hub for rendering both modes.

- **Task Queues for Agent Delegation**: Organize a `tasks/` directory with YAML/JSON files listing delegable units, e.g.:
  ```
  - task: Implement QueryCache in pika-engine
    inputs: [cache.rs skeleton, simple_fingerprint spec]
    outputs: [src/cache.rs, tests/cache_tests.rs]
    validation: Run benchmark with 1M queries
  ```
  Sequence by phases (e.g., Phase 1 tasks first). Use a script (e.g., `delegate.sh`) to feed tasks to the agent via prompts.

- **Environment Setup Script**: A `setup_env.rs` or Bash script to initialize the repo (e.g., `cargo new --lib pika-core`, install wgpu deps, set up Vulkan SDK for Windows testing). Include agent prompts like "Generate code for this task, ensuring it compiles with cargo check."

This structure enables agents to work on isolated crates/tasks, with clear build/test cycles.

## Inputs or Guidance an Agent Might Expect That Aren't Yet Spelled Out

Agents need explicit, prompt-friendly guidance to avoid ambiguity. Add a `prompts/` directory with templated inputs.

- **Prompt Templates for Key Tasks**: E.g., "Implement the GpuPlotRenderer struct in pika-ui/src/gpu.rs, using wgpu for pipelines as specified. Reuse sampling from existing plot codebase. Ensure integration with egui_plot's [Points](https://docs.rs/egui_plot/latest/egui_plot/struct.Points.html) for outliers [docs.rs](https://docs.rs/egui_plot/latest/egui_plot/). Output: Full Rust code with doc comments."

- **Validation and Iteration Guidance**: Specify "After generation, run `cargo test` and report failures; iterate until all pass." Include expected output formats (e.g., "Provide code diffs against skeleton files").

- **Reuse Directives**: Prompts like "Adapt the CSV import wizard from existing SQLite viewer codebase (path: /path/to/pebble/src/csv_import.rs) to use DuckDB's read_csv_auto in pika-engine."

- **Hardware/Env Assumptions**: Clarify "Assume Windows 11 with NVIDIA RTX 30-series GPU; test wgpu pipelines via headless mode if needed."

- **Debugging Hooks**: Require agents to insert logging (e.g., via `tracing`) at key points (e.g., event sends, cache hits) for post-generation verification.

These ensure agents have self-contained, actionable inputs, making the ambitious plan executable without back-and-forth.

## What Remains Unclear or Underdefined

- Exact WGSL shader code for aggregation in `PlotPipelines` (e.g., binning logic).
- Serialization details for `WorkspaceSnapshot` (e.g., RON vs. Bincode).
- Precise thresholds for rendering switches (e.g., 50_000 points) and how to tune them via tests.
- Integration of existing plot codebase (e.g., function mappings for LTTB sampling).
- Handling of egui_plot extensions [github.com](https://github.com/kylc/egui_wgpu_plot) for custom wgpu plotting if needed.

Addressing these via the suggested docs/specs will eliminate all roadblocks, enabling full automation.