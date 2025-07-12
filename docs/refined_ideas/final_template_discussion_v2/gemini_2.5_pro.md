Excellent. This v2 plan is focused, pragmatic, and provides clear constraints. Treating this as a final specification for an automation agent, here is a detailed breakdown of the additional scaffolding required to make it fully executable.

This assessment is structured to be a direct "to-do" list for preparing the project repository for the agent.

---

### **Pre-Automation Checklist for Pika-Plot**

#### **1. Detailed API Specifications & Data Contracts (`/specs` directory)**

The current plan describes Rust structs and enums, which is a great start. To prevent ambiguity for an agent, these need to be elevated into formal specifications with field-level documentation.

**Required Documents:**

1.  **`specs/core_types.md`**:
    *   **`NodeId`**: Specify the `Uuid` version to use (e.g., `v4`) and its serialization format (e.g., `hyphenated` string).
    *   **`QueryResult`**: Define the exact Arrow schema for `data`. Specify error states. What does the `QueryResult` contain when the query fails? Is it an `Err` variant, or a struct with an `Option<RecordBatch>` and `Option<Error>`?
    *   **`ImportOptions`**: A full struct definition is needed. Fields must be explicitly defined: `has_header: bool`, `delimiter: char`, `quote_char: char`, `escape_char: Option<char>`, `null_values: Vec<String>`, `column_type_overrides: HashMap<String, ArrowDataType>`.
    *   **`PlotConfig`**: This is critical and under-defined. This struct needs to exist for every plot type.
        *   Example for `ScatterPlotConfig`: `x_column: String`, `y_column: String`, `color_column: Option<String>`, `size_column: Option<String>`, `color_map: ColorMapEnum`, `point_size: f32`.
        *   Example for `HistogramConfig`: `column: String`, `bin_count: usize`, `log_scale: bool`.
    *   **`GpuBuffer`**: Define this struct. What does it contain? `vertex_buffer: wgpu::Buffer`, `index_buffer: Option<wgpu::Buffer>`, `instance_buffer: Option<wgpu::Buffer>`, `vertex_count: u32`, `instance_count: u32`, `topology: wgpu::PrimitiveTopology`.

2.  **`specs/events.md`**:
    *   For each `AppEvent` variant, specify every field's data type and purpose.
    *   **Crucially, define the success/failure paths.** For `QueryComplete`, the `result: Result<QueryResult>` is good. Do the same for all other events. `ImportComplete` should be `result: Result<{table_name, schema}>`. `PlotDataReady` must also have a `Result` wrapper.
    *   **Define ownership and data transfer semantics.** For `PlotDataReady { id, buffer: GpuBuffer }`, is `GpuBuffer` sent across the channel? This is impossible as `wgpu::Buffer` is not `Send`. The event must be `PlotDataReady { id, plot_data: Arc<PlotDataBytes> }`. The UI thread then uses these bytes to create the `wgpu::Buffer`. **This is a major architectural correction needed for automations.** The engine thread can't create GPU resources for the UI thread's wgpu device.

3.  **`specs/snapshot_format.md`**:
    *   Specify the archive format (e.g., `.zip` with a `.pikaplot` extension).
    *   Define the file structure inside the archive:
        ```
        - workspace.ron       # Contains WorkspaceSnapshot struct
        - /previews/
            - {node_id}.png   # Cached preview images for nodes
        ```
    *   Provide a complete, documented RON example of a `WorkspaceSnapshot` file for both a Notebook and a Canvas workspace. This is the ground truth the agent will code against.

#### **2. Visual & Interaction Design Specifications (`/designs` directory)**

The agent can write UI code, but it cannot invent design. Mockups need to be converted into precise design tokens and interaction logic.

**Required Documents:**

1.  **`designs/theme.json`**:
    *   A JSON file defining the color palette, fonts, and spacing.
    *   Example: `{"colors": {"bg_primary": "#1A1B26", "accent": "#73DACA", ...}, "fonts": {"body": "Inter", "heading": "Inter Bold"}, "spacing": {"xs": 4, "sm": 8, ...}}`.
    *   This file will be loaded by `pika-app` and used to create an `egui::Style`.

2.  **`designs/component_specs.md`**:
    *   For each major UI component (Notebook Cell, Canvas Node, Inspector Panel), provide a "redline" spec.
    *   **Canvas Node Spec:**
        *   Dimensions: `width: 250px`, `height: variable`.
        *   Header: `height: 32px`, `padding: 8px`, `icon_size: 16px`.
        *   Body: `padding: 12px`.
        *   Ports: `size: 10px`, `hover_size: 14px`, `color: accent`. Define input vs. output port locations (e.g., inputs on left, outputs on right).
    *   **Interaction Logic:** "When a user starts dragging from an output port, a line should be drawn from the port to the mouse cursor. When the mouse hovers over a compatible input port, the port should highlight. On mouse release over a valid port, a `ConnectionEstablished` event is fired."

3.  **`designs/layout_rules.md`**:
    *   Define the default layout for both UI modes.
    *   **Notebook Mode:** Rules for cell reordering, default cell height, and how new cells are added (e.g., always below the currently active cell).
    *   **Canvas Mode:** Define the behavior of the minimap. How does zooming work (e.g., centered on mouse cursor)? What is the pan speed? Are there grid snapping rules?

#### **3. Crate Scaffolding and Entry Points (`/` and `/crates`)**

The agent needs a skeleton to flesh out. You should create the file structure and define the initial entry points.

**Required Setup:**

1.  **Top-Level `Cargo.toml`**: Create the workspace definition as specified in the plan.
2.  **Individual Crate `Cargo.toml` Files**: Create each of the four crates (`pika-core`, `pika-engine`, etc.). In each `Cargo.toml`, add the necessary `dependencies` and specify the `path` to the other workspace crates.
    *   `pika-ui` will need `egui`, `eframe`, `wgpu`.
    *   `pika-engine` will need `duckdb`, `arrow`, `tokio`, `moka`, `dashmap`.
    *   `pika-core` will need `uuid`, `serde`, `thiserror`.
3.  **Create `main.rs` in `pika-app`**:
    ```rust
    // pika-app/src/main.rs
    mod app;
    mod runtime;
    
    use app::PikaApp;
    
    fn main() {
        // TODO: Initialize logging, read settings from disk.
        
        let native_options = eframe::NativeOptions {
            // TODO: Configure window settings from specs.
            ..Default::default()
        };
        
        eframe::run_native(
            "Pika-Plot",
            native_options,
            Box::new(|cc| {
                // TODO: Set up the egui context and custom fonts/style from theme.json.
                // TODO: Spawn the AppRuntime (Engine thread).
                // TODO: Create the initial PikaApp struct, passing the channel senders/receivers.
                Box::new(PikaApp::new(cc))
            }),
        );
    }
    ```
4.  **Create Stub Files for Key Modules**:
    *   `pika-app/src/app.rs`: `pub struct PikaApp { ... } impl eframe::App for PikaApp { ... }`
    *   `pika-app/src/runtime.rs`: `pub struct AppRuntime { ... }`
    *   `pika-engine/src/lib.rs`: `pub struct Engine { ... }`, `pub struct QueryCache { ... }`
    *   `pika-ui/src/lib.rs`: `pub enum WorkspaceMode { ... }`, and stubs for `notebook.rs`, `canvas.rs`, `nodes.rs`.
    *   `pika-core/src/lib.rs`: Define all the core types and events from the specs.

#### **4. Shader Code (`/assets/shaders`)**

GPU rendering is a hard constraint. The agent must be provided with the exact GLSL or WGSL shader code. It is unreasonable to expect an agent to write correct, high-performance shaders from prose.

**Required Files:**

1.  **`assets/shaders/direct.wgsl`**: Shader for rendering a small number of vertices directly.
2.  **`assets/shaders/instanced.wgsl`**: Shader for rendering quads via instancing for medium-sized datasets.
3.  **`assets/shaders/aggregate.wgsl`**: The compute shader for 2D histogram binning of massive datasets. This is the most complex one and requires precise definitions of the input data layout and output texture format.

#### **5. Test Fixtures and Golden Data (`/tests/fixtures`)**

The agent needs data to test against.

**Required Files:**

1.  **`tests/fixtures/simple.csv`**: A small, well-behaved CSV file (e.g., 100 rows).
2.  **`tests/fixtures/medium_with_nulls.csv`**: A larger CSV (e.g., 10,000 rows) that includes empty strings, `NULL`, `N/A` text, and other dirty data to test the import `ImportOptions`.
3.  **`tests/fixtures/golden_snapshot.pikaplot`**: A golden snapshot file (as specified in `specs/snapshot_format.md`) that the deserialization tests can use for verification.

By providing these concrete, unambiguous artifacts, you transform the project from a well-defined plan into a fully scaffolded set of tasks. An advanced automation agent can then pick up each stub file, read the corresponding spec, and execute the implementation with high confidence and minimal deviation.