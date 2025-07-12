This is an excellent synthesis. The clear breakdown into areas of consensus and disagreement, along with a highly focused set of technical questions, is precisely what's needed to move forward efficiently.

As Gemini 2.5 Pro, my goal is to provide pragmatic, systems-level answers that are immediately actionable for implementation. I will focus on the three areas where my expertise is most relevant: **GPU Compute**, **DuckDB-Arrow Integration**, and **Memory Pressure Handling**.

---

### **1. Technical Response: GPU Compute Optimization**

**Context:** Leveraging discrete GPUs with `wgpu` for data aggregation.

#### **1.1. Optimal Workgroup Size**

For modern NVIDIA (32 threads/warp) and AMD (64 threads/wavefront) discrete GPUs, a workgroup size of **64 or 128** is typically optimal.

*   **Recommendation:** Start with a 1D workgroup size of **`@workgroup_size(64)`**.
*   **Rationale:**
    1.  **Occupancy:** This size allows the GPU scheduler to efficiently hide memory latency by keeping multiple workgroups (or warps/wavefronts) resident on a single Streaming Multiprocessor (SM). A size of 64 cleanly maps to two NVIDIA warps or one AMD wavefront.
    2.  **Resource Usage:** It's a sweet spot that doesn't exhaust shared memory or register files too quickly, allowing for good parallelism.
    3.  **Simplicity:** Using a fixed, power-of-two size simplifies dispatch logic: `(num_elements + 63) / 64`.

#### **1.2. Use of Subgroup Operations**

**Absolutely, yes.** Subgroup operations are a significant performance lever, and `wgpu`'s support, while requiring an extension, is mature enough for production use.

*   **Benefit:** They allow threads within a subgroup (warp/wavefront) to exchange data using ultra-fast, dedicated hardware paths instead of slower, shared-memory atomics or barriers. This is perfect for parallel reductions (e.g., summing bins).
*   **Implementation Strategy:**
    1.  **Enable the Extension:** Check for adapter support for `wgpu::Features::SUBGROUP_OPERATIONS` at startup. If it's not available (which is unlikely on your target hardware), you can fall back to a simpler shader.
    2.  **Use in WGSL:** Add `#requires subgroups;` to your shader. Use operations like `subgroupBroadcast` and `subgroupAdd`.

    ```wgsl
    // Simplified reduction example using subgroups
    #requires subgroups;

    var<workgroup> shared_sums: array<atomic<u32>, 64>;

    @compute @workgroup_size(64)
    fn main(@builtin(local_invocation_id) local_id: vec3<u32>, @builtin(subgroup_invocation_id) subgroup_id: u32) {
        let my_value = ...;
        
        // Each thread in the subgroup computes its partial sum
        let partial_sum = subgroupAdd(my_value);

        // Only one thread in the subgroup writes the result to shared memory
        if (subgroup_id == 0) {
            atomicAdd(&shared_sums[local_id.x / 32], partial_sum);
        }
    }
    ```

#### **1.3. Handling Variable-Length Data in Compute**

This is a classic GPU problem. The most robust pattern is a **two-pass approach using a prefix sum (scan) algorithm.**

*   **Pass 1: Calculate Offsets.** Run a compute shader that takes your variable-length items as input. Each thread determines the length of its item and participates in a parallel prefix sum algorithm. The output is a buffer of offsets where each item's data should be written in the final output buffer.
*   **Pass 2: Gather/Scatter.** Run the main processing shader. Each thread reads its input item and its calculated offset from the offset buffer, then writes its results to the final output buffer at that precise location.

#### **1.4. GPU-side Sorting (Bitonic vs. Radix)**

*   **Bitonic Sort:** Simpler to implement in a compute shader. It's a comparison-based sort that works well for **small to medium-sized datasets** (up to a few million elements) and is more general-purpose. This is the **recommended starting point.**
*   **Radix Sort:** More complex to implement but significantly faster for **large datasets of integers or floats**. It requires multiple passes (one for each radix chunk, e.g., 4 bits at a time).

**Recommendation:** Implement a **Bitonic Sort** first. It will be sufficient for many use cases. Only investigate a Radix Sort if profiling shows that GPU sorting is a major bottleneck on your largest datasets. There are many well-documented WebGPU bitonic sort examples available online to adapt.

---

### **2. Technical Response: DuckDB-Arrow Zero-Copy Integration**

**Context:** Transferring data from DuckDB's analytical engine to GPU memory.

#### **2.1. Can we directly map DuckDB's Arrow buffers to GPU?**

**No.** This is a common misconception about "zero-copy." System RAM (where DuckDB/Arrow live) and GPU VRAM are physically separate memory spaces. A data transfer (copy) over the PCIe bus is unavoidable.

The goal is **efficient copy**, not zero-copy. This means:
1.  Avoiding intermediate CPU-side copies and format conversions.
2.  Packing data into the ideal layout for the GPU before the transfer.

The `AoS` (Array of Structs) conversion using `bytemuck` discussed previously is the most efficient pattern.

#### **2.2. Overhead of `query_arrow` vs. Custom Result Handling**

The overhead of DuckDB's `query_arrow` is **negligible**. DuckDB is internally columnar and Arrow-compatible. When you request an Arrow result, DuckDB is essentially giving you direct, read-only pointers to its internal data structures via the Arrow C Data Interface. The `duckdb-rs` crate then safely wraps these pointers. This is vastly more efficient than any custom row-by-row result handling you could write.

**Recommendation:** Always use the `query_arrow` interface. It is the most performant path.

#### **2.3. Handling Schema Evolution Without Full Re-Queries**

The only safe way to handle this is to treat schemas as immutable. If a schema changes, any data associated with the old schema is considered stale.

**Implementation Pattern:**

1.  When a query completes, hash its resulting `arrow::datatypes::Schema`.
2.  Store this `schema_hash` alongside the `RecordBatch` in your query cache.
3.  When a `PlotNode` consumes a `QueryResult`, it also stores the `schema_hash`.
4.  If the upstream `QueryNode` re-executes and produces a result with a *different* `schema_hash`, the `PlotNode` must be marked as "dirty" and its GPU buffers must be fully re-generated. It cannot reuse any old data, as the column layout may have changed entirely.

#### **2.4. Streaming Partial Results from DuckDB**

**Yes, this is possible and highly recommended** for improving perceived responsiveness. You can use the `ArrowArrayStreamReader` to fetch one `RecordBatch` at a time.

```rust
// pika-engine/src/query.rs
use duckdb::arrow::record_batch::RecordBatch;
use duckdb::Arrow;

pub async fn stream_query_results(
    conn: Arc<Mutex<duckdb::Connection>>,
    sql: String,
    // Channel to send batches to the UI as they arrive
    batch_tx: mpsc::Sender<RecordBatch>
) -> Result<(), PikaError> {

    tokio::task::spawn_blocking(move || {
        let mut conn_guard = conn.lock().unwrap();
        let mut arrow: Arrow = conn_guard.query_arrow(&sql, &[])?;
        let mut stream_reader = arrow.get_stream(None)?;

        while let Some(batch_result) = stream_reader.next() {
            let batch = batch_result?;
            
            // Send the batch to the UI thread.
            // Using a blocking send here as the stream is controlled by this loop.
            batch_tx.blocking_send(batch).map_err(|e| ...)?;
        }

        Ok(())
    }).await?
}
```
**UI Pattern:** The UI can receive the first batch and immediately render a low-fidelity preview of the plot, then progressively update it as more batches arrive. This makes the application feel much faster.

---

### **3. Technical Response: Memory Pressure Handling**

**Context:** Coordinating memory usage in a Windows-only app with large datasets and GPU buffers.

#### **3.1. Accurately Measuring GPU Memory Usage**

`wgpu` intentionally abstracts this. The most reliable method is to **track your own allocations.**

**Implementation Pattern:** Create a simple wrapper around `wgpu::Device`.

```rust
// pika-ui/src/gpu/tracked_device.rs
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TrackedDevice {
    device: wgpu::Device,
    allocated_bytes: Arc<AtomicUsize>,
}

impl TrackedDevice {
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(desc);
        self.allocated_bytes.fetch_add(desc.size as usize, Ordering::Relaxed);
        // Wrap the buffer in a custom type with a Drop impl to auto-decrement
        // or handle this manually.
        buffer 
    }
    
    // ... similar wrappers for create_texture ...
    
    pub fn used_vram(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }
}
```
This gives you application-level knowledge of VRAM usage. For deeper debugging, **use RenderDoc**, but for runtime decisions, your own tracking is required.

#### **3.2. Cache Eviction Strategy Under Pressure**

The best strategy is a **cost-based, tiered eviction.**

1.  **Define a Cost:** Assign a "cost" to each cached item.
    *   `RecordBatch` in query cache: `cost = size_in_bytes`.
    *   `GpuBuffer` in plot cache: `cost = size_in_bytes * 2.0` (VRAM is more precious).
    *   Plot data that is *not currently visible*: `cost = cost * 0.5` (lower priority to keep).
2.  **Central Coordinator:** Have a `MemoryCoordinator` that holds a priority queue (min-heap) of all cached items, ordered by their cost.
3.  **On Pressure:** When total memory exceeds a threshold (e.g., 80% RAM), the coordinator sends `Evict(item_id)` events for items with the lowest cost until the pressure is relieved.

#### **3.3. Custom Allocator for Memory Tracking**

**Avoid writing your own low-level allocator.** The complexity is immense. Instead, use a dedicated library. The consensus points towards **`gpu-allocator`**. It's designed for exactly this use case: it handles alignment, sub-allocation from larger blocks (reducing `vkAllocateMemory` calls), and provides memory usage statistics out of the box. It will integrate perfectly with the `TrackedDevice` wrapper pattern.

#### **3.4. Coordinating DuckDB and GPU Memory Limits**

This is the key role of the `MemoryCoordinator`.

**Implementation Pattern:**

1.  **Global Budget:** The application starts and determines a total memory budget (e.g., 75% of available system RAM).
2.  **Central Ledger:** The `MemoryCoordinator` (living in `pika-app` or `pika-engine`) is the single source of truth.
3.  **Registration:**
    *   When `pika-engine`'s `QueryCache` caches a `RecordBatch`, it registers it with the coordinator: `coordinator.register_ram_usage(id, size)`.
    *   When `pika-ui`'s `GpuResourceManager` allocates a buffer, it registers it: `coordinator.register_vram_usage(id, size)`.
4.  **Decision Making:**
    *   Before a large operation (query, import), the engine asks the coordinator: `coordinator.can_allocate(estimated_bytes)`.
    *   The coordinator checks the total budget. If insufficient, it triggers the eviction process described above, starting with the cheapest resources, until enough memory is freed.

This centralized, event-driven approach prevents the UI and engine from fighting over resources and provides a robust, predictable system for managing memory pressure.