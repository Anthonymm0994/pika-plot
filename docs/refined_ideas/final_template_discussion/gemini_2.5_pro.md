Excellent. This is a formidable plan that demonstrates deep technical thinking. My role is to be the adversarial reviewer, to stress-test these ideas before a single line of code solidifies them. The goal is not to invalidate the vision but to make it more resilient.

Here is my critique, organized by architectural domain.

---

## `docs/review_notes/architectural_review_01.md`

### 1. UI/UX: Challenging the "Infinite Canvas" Metaphor

The plan is heavily invested in the "infinite canvas" as the primary user interface.

**Hidden Assumption:** An unstructured, infinite canvas is the most intuitive and powerful interface for *exploratory data analysis (EDA)* for *non-technical users*.

**My Challenge:**
This assumption is questionable and potentially counter-productive to the goal of generating "insightful plots."

1.  **The "Blank Canvas" Problem:** For non-technical users, an infinite, empty space is intimidating. It offers no guidance, no "scent" of what to do next. This can lead to choice paralysis, contrasting with the "intuitive" design goal.
2.  **Encourages Disorder:** The primary advantage of tools like notebooks (Jupyter) or dashboards (Tableau) is their inherent structure. They guide a user through a semi-linear or grid-based thought process. An infinite canvas encourages a sprawling, chaotic mess of nodes that becomes impossible to navigate or interpret, defeating the goal of "sharing insights easily." The final workspace will look less like a clean data flow and more like a tangled plate of spaghetti.
3.  **Breaks Established EDA Patterns:** A core pattern in EDA is creating "small multiples"—a grid of similar plots where one variable is changed (e.g., plotting revenue vs. time for each product category). This is trivial to do in a grid-based layout but clumsy and difficult to align on an infinite canvas.

**Proposed Alternative: The "Structured Canvas" or "Flex-Grid"**

Instead of a fully infinite canvas, consider a more structured workspace that combines freedom with guidance:

*   **A "Flex-Grid" Layout:** Imagine a primary grid (like a CSS grid) where users can place nodes. Nodes can span multiple cells. This makes alignment and creating "small multiples" natural. The grid can expand infinitely downwards and sideways, but within a structured paradigm.
*   **Region-based organization:** Allow users to draw named "regions" on the canvas to group related nodes (e.g., "Customer Segmentation Analysis"). This adds a layer of semantics on top of the visual layout.
*   **Storytelling Mode:** A feature that lets a user define a sequence of "views" (camera positions and node visibility) on the canvas to create a linear narrative for export. This directly serves the "share insights" goal better than exporting a messy canvas.

---

### 2. The Rendering Engine: `egui` and `wgpu` — A Painful Marriage

The plan lays out a custom `wgpu` rendering pipeline for plots while using `egui` for the shell UI.

**Hidden Assumption:** One can easily drop a high-performance, custom `wgpu` context inside an `egui` application. `egui` will handle the widgets, and `wgpu` will handle the plots, and they will coexist happily.

**My Challenge:**
This is a significant architectural misstep that underestimates the complexity of integrating two rendering loops.

1.  **Who Owns the Swap Chain?** `eframe` (the `egui` application runner) wants to own the `wgpu::Device`, `wgpu::Queue`, and the render pass via its `Painter`. A separate `GpuPlotRenderer` that also wants to manage its own pipelines, buffers, and render targets will fight `eframe` for control. This leads to extremely complex synchronization, texture sharing, and state management issues.
2.  **Reinventing the Wheel:** The `egui` ecosystem has a canonical way to do this: `egui::PaintCallback`. This allows you to inject custom `wgpu` rendering commands *directly into egui's own render pass*. It handles the boilerplate of setting up the device, queue, and render target for you. The current plan seems to ignore this, opting for a much harder, from-scratch integration.
3.  **Input Handling Hell:** How will mouse clicks from the `egui` window be translated into the coordinate system of your custom `wgpu` viewport, especially when considering pan, zoom, and `egui`'s own UI scaling? `egui::PaintCallback` provides the clipping rectangle and coordinate transformations necessary to solve this. Doing it manually is a minefield of off-by-one errors.

**Proposed Alternative: A Phased, Idiomatic Approach**

1.  **Phase 1 (Immediate):** Start with **`egui_plot`**. It's already highly optimized for many use cases and will handle 10k-100k points interactively. This gets a working product faster and validates the data pipeline. *Do not write a single line of custom rendering code until `egui_plot` proves insufficient.*
2.  **Phase 2 (When Needed):** When you need GPU-accelerated aggregation for 1M+ points, use **`egui::PaintCallback`**. Create your custom `GpuPlotRenderer` that hooks into the `egui` painter. It will receive the `wgpu` device/queue from `egui` to create its compute pipelines and render its aggregated texture onto a quad within the UI. This is the idiomatic, supported, and simplest path.
3.  **The LodManager is Overkill (for now):** The `LodManager` building a full pyramid of downsampled data upfront is a premature optimization. A much simpler model is: on each frame, check the zoom level. If zoomed out, run the GPU aggregation compute shader. If zoomed in, draw a subsample of raw points. This is reactive, not pre-emptive, and far easier to implement.

---

### 3. Caching: A Solution in Search of a Problem?

The proposed `HierarchicalCache` is a beautiful and complex piece of engineering. It might also be the single greatest source of implementation drag and bugs.

**Hidden Assumption:** A multi-level, predictive, lineage-tracking, dependency-aware cache is required to achieve "instant feedback."

**My Challenge:**
The complexity of this cache system far outweighs the likely benefits for this specific problem domain, especially compared to the power of the chosen backend.

1.  **DuckDB is Already a Cache:** DuckDB is an in-memory-first analytical database. It is exceptionally fast. Queries over multi-million row datasets that fit in RAM often execute in milliseconds. The plan seems to treat DuckDB as a dumb, slow block store, which it is not. The latency bottleneck will likely be UI rendering or GPU data transfer, not a simple `GROUP BY` query.
2.  **Cache Invalidation is Hard:** The document describes a "CacheCoordinator" that performs "smart eviction" based on a "dependency graph." This is one of the hardest problems in computer science. Implementing this correctly is a massive project in itself. A bug here will lead to stale data being shown to the user, a critical failure for a data analysis tool.
3.  **Predictive Prefetching is Speculative Fiction:** The idea that the app can accurately predict a user's next exploratory step with >70% confidence is highly optimistic. This will likely lead to wasted computation and memory, actively harming performance, not helping it.

**Proposed Alternative: Brutal Simplicity**

1.  **Level 1 Cache (The Only One You Need for V1):** `(Normalized SQL Hash) -> Arc<RecordBatch>`. Use a simple `DashMap` or `moka` (a high-performance concurrent cache crate). This is your *query cache*. It prevents re-running the same SQL. This will provide 90% of the perceived performance benefit.
2.  **Level 2 Cache (Plot-Specific):** `(Query Cache Fingerprint + Plot Settings) -> GPU-Ready Buffer`. This caches the result of *transforming* query data into something a plot can render (e.g., the aggregated heatmap texture).
3.  **Delegate to DuckDB:** Don't build a complex engine to derive results from cached supersets. If the user adds a `WHERE` clause, *just run the new query*. DuckDB will be shockingly fast. The time spent building the derivation logic will be orders of magnitude greater than the time saved over the application's lifetime.

---

### 4. Portability & State: The Snapshot Is a Trap

The snapshot design bundles data chunks (Parquet) and workspace state into a single archive file.

**Hidden Assumption:** A self-contained snapshot including all data is the best way to achieve portability and reproducibility.

**My Challenge:**
This model has critical flaws when dealing with large data and resource-constrained machines.

1.  **The 50GB Snapshot:** What happens when the user has analyzed 50GB of CSVs? The `.pikaplot` file is now 50GB. This is not portable. It can't be emailed, sent on Slack, or easily stored.
2.  **Violates "Offline-First" Principle:** The core idea is that data lives locally. If a user tries to open this 50GB snapshot on a laptop with 16GB of RAM, the application will either fail to load or thrash the system into oblivion as it tries to unpack and cache the data. The snapshot's resource requirements are implicitly tied to the machine that created it.
3.  **Source of Truth Ambiguity:** What happens if the source CSV is updated? The snapshot now contains stale, embedded data. The user has no easy way to "refresh" the analysis with the new data.

**Proposed Alternative: Recipe-Based Snapshots**

A snapshot should be a lightweight **recipe**, not a heavy archive.

*   **Core Content:** The `.pikaplot` file should contain the canvas state (node positions, connections, annotations) and the **queries**, but **not the full data**.
*   **Data Linking:**
    *   It stores *references* to the source data files (e.g., `~/data/sales_2024.csv`).
    *   On loading, Pika-Plot checks for these files. If found, it re-runs the import and query pipeline (which will be fast due to the now-warmed-up query cache).
    *   If not found, it prompts the user to locate the missing files.
*   **Optional Data Embedding:** For true portability (e.g., sharing with a colleague), provide an "Export as Packaged Snapshot" option that *does* bundle the data. This should be an explicit user action, and the UI should warn about the potential file size.

---

### 5. From-Scratch Rethink: The Pragmatic Pika-Plot v0.1

If I were to architect the initial version from scratch, I would drastically reduce the scope to prioritize delivering core user value with maximum stability.

**Guiding Principle:** "Make it work, make it right, *then* make it fast (if necessary)."

1.  **Simplified Crate Structure (3 Crates):**
    *   `pika-plot`: The main binary application crate. Contains the `eframe::App` implementation, all UI code, and async orchestration.
    *   `pika-core`: Contains the backend logic: DuckDB connection, query execution, caching, data structures. No traits, just concrete structs initially.
    *   `pika-importer`: The polished CSV import logic, largely isolated.

2.  **UI/Rendering Stack:**
    *   Use `eframe` for the application shell.
    *   Use `egui_plot` for ALL plotting. No `wgpu` code.
    *   UI is a simple, resizable grid layout, not an infinite canvas.

3.  **Backend & Caching:**
    *   A single `ComputeManager` struct that holds the `duckdb::Connection` and is shared behind an `Arc<Mutex>`.
    *   A single `moka` cache: `Cache<NormalizedSqlString, Arc<RecordBatch>>`.
    *   All queries are sent to a `tokio` thread pool via an MPSC channel. The `ComputeManager` runs the query (checking the cache first) and sends the `Arc<RecordBatch>` back to the UI thread.

4.  **Data Flow:**
    1.  User imports CSV -> Importer uses `duckdb::Connection` to run `read_csv_auto`.
    2.  UI adds a "Data Source" card to the grid.
    3.  User adds a "Plot" card. The plot UI has dropdowns to select the data source, columns, and plot type.
    4.  The UI constructs a SQL query based on the dropdowns.
    5.  The SQL is sent to the `ComputeManager`.
    6.  The result `RecordBatch` is sent back and fed *directly* into `egui_plot` (sampling the first 10,000 rows if necessary).

This architecture is less "masterfully engineered" on paper, but it is **shippable, testable, and robust**. It delivers 80% of the user value with 20% of the proposed complexity. It solves the user's core problem—easily plotting large CSVs offline—before getting bogged down in technically "interesting" but ultimately secondary problems like predictive caching and force-directed layouts. From this strong, simple foundation, the more advanced features from the original plan can be added *incrementally* as performance bottlenecks are *proven* to exist.