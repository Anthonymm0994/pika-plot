This is an exceptionally well-thought-out and detailed plan. The "Pebble" prototype provides a fantastic foundation, and the vision for Pika-Plot is clear, ambitious, and achievable with the proposed technology stack. My goal here is not to upend your vision, but to reinforce it, offer concrete refinements, and provide a clear, actionable path forward.

Let's build this master plan.

---

### 1. Analysis and Architectural Refinements

Your plan is already 95% of the way there. The chosen stack (`egui`, DuckDB, Arrow) is a modern powerhouse for this exact use case. The analysis from sources like [benw.is](https://benw.is/posts/better-desktop-web-apps) confirms that a Rust-based backend with a framework like `egui` is a state-of-the-art approach for creating high-performance desktop applications.

Here are a few suggestions to elevate the architecture.

#### **Refinement: The Compute & Interaction Layers**

Your diagram is excellent. Let's make the data flow more explicit. The key to "instant feedback" and a non-blocking UI is a clear separation between the UI thread and a compute thread pool.

**Proposed Flow:**

1.  **UI Thread (`egui`):** Handles all drawing and user interaction. It *never* blocks. When a long-running task is needed (e.g., running a SQL query, resampling a plot), it sends a message to the compute layer.
2.  **Compute Layer (`tokio`):** Runs on a `tokio` runtime. It listens for requests from the UI thread via an MPSC (Multi-Producer, Single-Consumer) channel.
3.  **Dataflow:**
    *   The user connects a `QueryNode` to a `TableNode`.
    *   The `QueryNode`'s UI sees this state change and sends a `ComputeRequest::ExecuteQuery { ... }` message to the `tokio` runtime.
    *   The `QueryNode`'s UI immediately starts drawing a "loading" state.
    *   A `tokio` worker task picks up the job, queries DuckDB, and gets an Arrow `RecordBatch`.
    *   The worker then sends a `ComputeResponse::QueryResult { ... }` message back to the UI thread.
    *   The UI thread's update loop receives the message and updates the `QueryNode`'s state with the actual data, triggering a redraw with the results table or plot.

This makes the "UI Event Bus" a pair of `tokio::mpsc` channels: `ui_to_compute_tx` and `compute_to_ui_rx`.

#### **Refinement: The DuckDB + Arrow Synergy**

Your choice of DuckDB and Arrow is perfect. DuckDB is designed for exactly this kind of analytical workload and has first-class support for Arrow.

*   **CSV to DuckDB:** DuckDB's CSV reader is highly optimized and can often infer types and structures more robustly than manual implementations [blobs.duckdb.org](https://blobs.duckdb.org/docs/duckdb-docs-0.9.2.pdf). You may be able to simplify your existing `CsvImportPanel` by delegating more of the heavy lifting to DuckDB's `read_csv_auto` function.
*   **Zero-Copy:** You're aiming for a zero-copy architecture. As a Hacker News discussion on DuckDB points out, you can achieve near-zero-copy data transfer between DuckDB and an Arrow-aware client [news.ycombinator.com](https://news.ycombinator.com/item?id=24531085). By fetching query results directly as Arrow `RecordBatch`es, you avoid costly serialization/deserialization cycles between your compute and storage layers. This is the core of Pika-Plot's performance.

#### **Refinement: The Infinite Canvas**

`egui` operates in immediate mode, which can present a paradox for positioning content on a large canvas: you need to know the content's size to position it, but you don't know the size until you've laid it out [crates.io](https://crates.io/crates/egui).

For an infinite canvas, you can solve this by building a custom widget that manages a view transform (pan & zoom).

*   You'll store the absolute "world" coordinates for each node.
*   In your UI code, you'll wrap the node-drawing logic in an `egui::Area`.
*   You'll use `ui.transform()` to apply the current pan/zoom transformation. `egui` will then handle clipping, so you only pay the cost for drawing the nodes currently in view.

---

### 2. Modularization, Traits, and Code Structure

Your proposed module layout is clean and logical. Let's define the core traits that will glue these modules together.

#### The `Node` Trait: The Heart of the Canvas

The central abstraction for your canvas is the `Node`. Every object on the canvas—a table, a query, a plot, an annotation—will implement this trait.

```rust
// in src/canvas/node.rs

use egui::Ui;
use uuid::Uuid;
use std::sync::Arc;
use arrow::record_batch::RecordBatch;

// Defines what kind of data a port accepts or produces
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortDataType {
    Table(String),      // Represents a table name in DuckDB
    RecordBatch,        // Represents an in-memory Arrow RecordBatch
    None,
}

// An input or output for a node
pub struct Port {
    pub id: Uuid,
    pub name: String,
    pub data_type: PortDataType,
}

/// A trait for any object that can be placed on the Pika-Plot canvas.
pub trait Node {
    /// A unique identifier for this node instance.
    fn id(&self) -> Uuid;

    /// The name displayed on the node's header.
    fn name(&self) -> &str;

    /// The node's position on the infinite canvas.
    fn position(&self) -> egui::Pos2;

    /// Returns the node's input ports.
    fn inputs(&self) -> &[Port];

    /// Returns the node's output ports.
    fn outputs(&self) -> &[Port];

    /// Draws the node's UI and processes interactions.
    /// `ctx` would be a context object providing access to the compute channel, etc.
    /// This is the main method called by the canvas renderer each frame.
    fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext);

    /// Handles an incoming data connection to one of its input ports.
    /// The `data` would be an enum, e.g., `NodeData::TableName(String)`
    fn accept_input(&mut self, port_id: Uuid, data: Arc<dyn std::any::Any>);
}
```

#### Node Interaction Diagram Sketch

This trait enables a clear, reactive data flow.

```
+----------------+      (PortDataType::Table)      +----------------+
|                | -------------------------------> |                |
|  TableNode     |                                  |  QueryNode     |
| (outputs[0])   |                                  | (inputs[0])    |
|                | <------------------------------- |                |
+----------------+      (User links nodes)         +----------------+
       ^                                                    |
       | (CSV Import)                                       | (Sends SQL to Compute Layer)
       |                                                    |
 Workspace                                                  v
                                                   +----------------+
                                                   |                | (PortDataType::RecordBatch)
                                                   |  (outputs[0])  | ----------------> +-----------+
                                                   |                |                   | PlotNode  |
                                                   +----------------+                   | (inputs[0])|
                                                                                        +-----------+
```

---

### 3. A Robust Testing Strategy

A multi-layered testing approach will ensure Pika-Plot is stable and maintainable.

1.  **Unit Tests (`#[test]`)**:
    *   **`compute` Module**: This is the most critical module for unit testing.
        *   Test the `sqlparser-rs` AST normalization logic extensively.
        *   Test the fingerprinting logic: different-looking but semantically identical queries should produce the same fingerprint.
        *   Test the plot samplers (`LTTB`, binning) with known inputs and outputs.
    *   **`storage` Module**: Test the DuckDB interface wrapper. Ensure error handling for invalid SQL or missing files is robust.

2.  **Integration Tests (`tests/` directory)**:
    *   Create tests that simulate a user workflow but without a GUI.
    *   **Example Test:**
        1.  Programmatically create a `StorageManager`.
        2.  Call the method to import a test CSV file.
        3.  Call the method to execute a SQL query against that imported data.
        4.  Assert that the returned Arrow `RecordBatch` has the correct schema and row count.
    *   This validates that your layers work together correctly.

3.  **Snapshot Testing for UI (`insta` crate)**:
    *   For complex custom widgets (like your `CsvImportPanel` or a finished `PlotNode`), you can't easily assert on the output.
    *   Snapshot tests render the widget's state to a text format (or even an image with extensions) and compare it to a stored "golden" file. This is perfect for catching unintended UI regressions.

4.  **Benchmarking (`criterion.rs`)**:
    *   Benchmark performance-critical code paths.
    *   **Targets:**
        *   Query execution time (DuckDB vs. Arrow overhead).
        *   Plot data sampling/aggregation algorithms.
        *   Workspace snapshot serialization/deserialization.

---

### 4. First 5 Actionable Prototyping Tasks

Here is a step-by-step plan to get your prototype off the ground. These tasks are designed to be dele-gated and built upon sequentially.

**Task 1: Setup Core Crate & DuckDB Storage Layer**
*   **Goal:** Replace SQLite with DuckDB and create a basic data access manager.
*   **Action:**
    1.  Create a new Rust binary project: `cargo new pika-plot`.
    2.  Add `duckdb`, `arrow`, and `eframe` as dependencies.
    3.  In `src/storage/mod.rs`, create a `StorageManager` struct that holds a `duckdb::Connection`.
    4.  Implement a method `fn import_csv(&self, path: &Path) -> Result<String, Error>` which uses DuckDB's `CREATE TABLE my_table AS SELECT * FROM read_csv_auto(...)` and returns the new table name.
    5.  Implement a method `fn query_to_arrow(&self, sql: &str) -> Result<Vec<RecordBatch>, Error>`.
*   **Deliverable:** A `main.rs` that can, on the command line, import a CSV and run `SELECT * FROM ... LIMIT 10`, printing the Arrow result.

**Task 2: Basic Draggable Node on a Canvas**
*   **Goal:** Render a single, movable node using `eframe`/`egui`. Forget about data for now.
*   **Action:**
    1.  Define the initial `Node` trait (a simpler version of the one above).
    2.  Create a `struct TableNode` that implements `Node`. For now, its `draw` method just creates a framed window using `egui::Frame` that says its name.
    3.  In your `eframe::App` implementation, use a pannable/zoomable `egui::Area` as your canvas. Draw the `TableNode` inside it, responding to drag events to update its position.
*   **Deliverable:** A window with a blank background where you can drag a "Table" widget around.

**Task 3: End-to-End Data Flow (CSV -> TableNode)**
*   **Goal:** Connect your existing `CsvImportPanel` to the canvas.
*   **Action:**
    1.  Port your polished `CsvImportPanel` from "Pebble" into `src/ui/panels/`.
    2.  When the import is confirmed in the UI, use the `StorageManager` from Task 1 to import the data into DuckDB.
    3.  Upon success, create a new `TableNode` instance on the canvas at the mouse position, displaying the name of the newly created table.
*   **Deliverable:** A UI where you can click "Import CSV," select a file, and see a new `TableNode` appear on the canvas.

**Task 4: Introduce Asynchronous Compute**
*   **Goal:** Make queries non-blocking.
*   **Action:**
    1.  Add `tokio` to the project.
    2.  Create the `ui_to_compute` and `compute_to_ui` MPSC channels in your main `App` struct.
    3.  Spawn a `tokio` runtime on a background thread.
    4.  Create a `QueryNode`. Its UI will have a `TextEdit` for SQL. When the user clicks "Run", send a message with the SQL to the `tokio` thread.
    5.  The `tokio` thread runs the query using the `StorageManager` and sends the resulting `RecordBatch` back to the UI thread.
    6.  The `App::update` loop checks for messages and updates the `QueryNode`'s state, which then renders a simple preview of the data.
*   **Deliverable:** A `QueryNode` that shows a loading indicator, then displays query results without freezing the application.

**Task 5: Your First Interactive Plot**
*   **Goal:** Visualize the result of a query.
*   **Action:**
    1.  Add `egui_plot` as a dependency.
    2.  Create a `PlotNode` that accepts an Arrow `RecordBatch` as input.
    3.  Connect the `QueryNode`'s output to the `PlotNode`'s input.
    4.  In the `PlotNode::draw` method, extract the first two numerical columns from the `RecordBatch` and create an `egui_plot::PlotPoints` object.
    5.  Render a basic scatter plot. Don't worry about 10M rows yet; sample the first 1,000 to prove the pipeline works.
*   **Deliverable:** You can connect `TableNode -> QueryNode -> PlotNode` to go from raw CSV to an interactive plot on the screen.

---

### 5. Notes on 10M+ Row Interactivity

This is the final boss. The key is to **never send 10 million data points to the renderer**. The GPU would choke, and the visual result is a useless, over-plotted mess anyway.

Your `plot` module's job is to intelligently reduce the data before it ever hits `egui_plot` or a custom `wgpu` renderer.

1.  **Adaptive Aggregation:** The `PlotNode`'s `update` logic should be aware of the plot's current viewport (zoom level and pan).
2.  **Zoomed Out (e.g., viewing all 10M points):**
    *   **Scatter Plot:** Don't render points. Run a 2D aggregation (a "heatmap") on a compute thread. Render this as a grid of colored squares. The resolution of the grid would be similar to the pixel size of the plot area (e.g., 800x600).
    *   **Line Chart:** Use an aggressive downsampling algorithm like **LTTB (Largest Triangle Three Buckets)** to create a visually representative line with only a few thousand points.
3.  **Zoomed In (e.g., viewing 50,000 points):**
    *   The `PlotNode` requests the raw data for its current viewport from the `QueryNode`'s cached `RecordBatch`.
    *   If the point count is still too high (>10k), apply a lighter sampling.
    *   If it's low enough (<10k), send the raw points to the renderer.

This logic makes the `compute` layer essential for plotting. The `PlotNode` doesn't just render data; it *requests a drawable summary* of the data for its current view.

This architecture is solid, sophisticated, and sets you up for success. By tackling it in these structured stages, you and your AI assistant can build a truly masterful application. Good luck with Pika-Plot