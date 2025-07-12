# Technical Questions and Insights - Round 3: Module Delegation

## Overview

This document delegates specific module implementations to leverage each agent's strengths. Each section includes complete context, requirements, and expected deliverables.

## 1. GPU Aggregation Shaders (Priority: CRITICAL)

**Assigned to: Gemini 2.5 Pro & Claude Opus 4**

### Context
We need production-ready WGSL shaders for GPU aggregation. The shaders must handle binning, aggregation, and density calculations for scatter plots and heatmaps.

### Requirements
1. Create complete WGSL shader modules for:
   - 2D binning aggregation (for scatter plots)
   - Density calculation (for heatmaps)
   - Min/max reduction (for auto-scaling)
   - Histogram computation

2. Each shader must:
   - Use 256-thread workgroups (consensus from research)
   - Handle edge cases (empty data, single point)
   - Include proper synchronization
   - Be optimized for discrete GPUs (NVIDIA/AMD)

### Deliverables Needed
```rust
// File: pika-engine/src/gpu/shaders/aggregation.wgsl
// Complete WGSL shader code with:
// - Struct definitions
// - Binding layouts
// - Compute entry points
// - Comments explaining optimization choices

// File: pika-engine/src/gpu/shaders/mod.rs
// Rust module that:
// - Loads shaders
// - Creates pipeline layouts
// - Provides type-safe interfaces
```

### Specific Questions
1. How should we handle dynamic bin counts in the shader?
2. What's the optimal shared memory usage pattern for aggregation?
3. Should we use atomic operations or parallel reduction?
4. How do we efficiently handle sparse data?

## 2. DuckDB Streaming Implementation (Priority: HIGH)

**Assigned to: Gemini 2.5 Pro**

### Context
We need a production implementation of DuckDB streaming with progress reporting, backpressure handling, and memory management.

### Requirements
1. Implement the streaming traits in `pika-engine/src/streaming.rs`
2. Add progress callbacks to DuckDB operations
3. Handle partial results for progressive UI updates
4. Implement query cancellation

### Deliverables Needed
```rust
// File: pika-engine/src/database/streaming.rs
impl DuckDbStream {
    // Complete implementation with:
    // - Batched result streaming
    // - Progress callbacks
    // - Memory-aware batch sizing
    // - Error recovery
}

// File: pika-engine/src/import/streaming_csv.rs
impl CsvStream {
    // CSV streaming that:
    // - Handles large files (50GB+)
    // - Reports progress
    // - Performs type inference per batch
    // - Handles encoding issues
}
```

### Specific Questions
1. How do we implement progress callbacks without blocking DuckDB?
2. What's the best batch size for streaming (rows vs. bytes)?
3. How do we handle schema changes mid-stream?
4. Should we use DuckDB's CSV reader or implement our own?

## 3. Windows File Utilities (Priority: HIGH)

**Assigned to: All Agents**

### Context
We need robust Windows file handling that deals with long paths, file locking, case sensitivity, and other Windows quirks.

### Requirements
1. Path normalization utilities
2. File lock detection and handling
3. Safe file operations with retry logic
4. Integration with `dunce` crate as suggested

### Deliverables Needed
```rust
// File: pika-core/src/utils/windows_fs.rs
pub mod windows_fs {
    // Functions needed:
    // - normalize_path() - handles \\?\ prefix, UNC paths
    // - is_file_locked() - detects if file is open elsewhere
    // - safe_open_file() - with retry and error messages
    // - watch_directory() - for hot reload support
}

// File: pika-cli/src/utils/path_completion.rs
// Windows-aware path completion for CLI
```

### Specific Questions
1. How do we detect which program has a file locked?
2. Should we implement shadow copying for locked files?
3. How do we handle junction points and symbolic links?
4. What's the best way to handle MAX_PATH limitations?

## 4. Testing Infrastructure (Priority: HIGH)

**Assigned to: Claude Opus 4 & GPT-4.5**

### Context
We need comprehensive testing utilities that work in CI without GPU hardware, provide good coverage, and catch regressions.

### Requirements
1. GPU testing framework using software rendering
2. DuckDB test fixtures and helpers
3. Property-based tests for data operations
4. Benchmark harness for performance tracking

### Deliverables Needed
```rust
// File: pika-engine/tests/common/mod.rs
pub mod gpu_test_utils {
    // Utilities for:
    // - Creating software rendering context
    // - Comparing GPU results with CPU reference
    // - Generating test datasets
    // - Asserting buffer contents
}

pub mod db_test_utils {
    // Utilities for:
    // - In-memory test databases
    // - Loading test fixtures
    // - Query result assertions
    // - Schema comparison
}

// File: pika-engine/benches/gpu_benchmarks.rs
// Comprehensive GPU benchmarks for:
// - Aggregation performance by data size
// - Memory transfer overhead
// - Pipeline compilation time
```

### Specific Questions
1. How do we test GPU memory pressure scenarios?
2. What's the best way to generate deterministic test data?
3. Should we use property testing for aggregation correctness?
4. How do we benchmark GPU operations consistently?

## 5. UI Components (Priority: MEDIUM)

**Assigned to: All Agents**

### Context
We need polished egui components for the node editor, plot configuration, and data preview.

### Requirements
1. Node editor with smooth pan/zoom
2. Plot configuration panels
3. Data table with virtual scrolling
4. Progress indicators for long operations

### Deliverables Needed
```rust
// File: pika-ui/src/widgets/node_editor.rs
pub struct NodeEditor {
    // Implementation with:
    // - Smooth pan/zoom (GPU accelerated)
    // - Node drag & drop
    // - Edge routing
    // - Selection tools
}

// File: pika-ui/src/widgets/data_table.rs
pub struct DataTable {
    // Virtual scrolling table that:
    // - Handles millions of rows
    // - Supports sorting/filtering
    // - Shows data types
    // - Allows cell selection
}

// File: pika-ui/src/widgets/plot_config.rs
// Configuration UI for each plot type
```

### Specific Questions
1. How do we implement smooth zooming with thousands of nodes?
2. What's the best virtual scrolling strategy for large tables?
3. Should plot previews be GPU rendered or cached images?
4. How do we handle responsive layout for different screen sizes?

## 6. Error Handling Patterns (Priority: MEDIUM)

**Assigned to: Grok 4 & GPT-4.5**

### Context
We need consistent, user-friendly error handling throughout the application with proper recovery strategies.

### Requirements
1. Error messages that guide users to solutions
2. Automatic retry with backoff for transient errors
3. Graceful degradation patterns
4. Error reporting for debugging

### Deliverables Needed
```rust
// File: pika-core/src/error/handlers.rs
pub trait ErrorHandler {
    // Methods for:
    // - Formatting user messages
    // - Determining retry strategy
    // - Logging for debugging
    // - Recovery suggestions
}

// File: pika-ui/src/error/ui_handlers.rs
// UI-specific error handling:
// - Toast notifications
// - Error dialogs
// - Progress failure states
// - Undo/recovery options
```

### Specific Questions
1. How do we detect transient vs. permanent errors?
2. What information should error reports include?
3. How do we handle cascading errors in node graphs?
4. Should we implement error recovery checkpoints?

## 7. Performance Monitoring (Priority: MEDIUM)

**Assigned to: Gemini 2.5 Pro**

### Context
We need built-in performance monitoring to catch regressions and help users optimize their workflows.

### Requirements
1. Frame time tracking
2. Memory usage monitoring
3. GPU utilization tracking
4. Query performance metrics

### Deliverables Needed
```rust
// File: pika-engine/src/monitoring/metrics.rs
pub struct MetricsCollector {
    // Collects:
    // - Frame times
    // - Memory allocations
    // - GPU command timings
    // - Query durations
}

// File: pika-ui/src/debug/perf_overlay.rs
// Debug overlay showing:
// - FPS counter
// - Memory usage graph
// - GPU utilization
// - Active operations
```

### Specific Questions
1. How do we measure GPU utilization on Windows?
2. What's the overhead of performance monitoring?
3. Should metrics be always-on or debug-only?
4. How do we export metrics for analysis?

## Conclusion

Each agent should focus on their assigned modules and provide:
1. **Complete, production-ready code** (not sketches or pseudocode)
2. **Comprehensive tests** for their modules
3. **Documentation** explaining design decisions
4. **Performance considerations** and benchmarks where applicable

Priority order:
1. GPU Aggregation Shaders (blocks everything)
2. DuckDB Streaming (core functionality)
3. Windows File Utilities (robustness)
4. Testing Infrastructure (quality assurance)
5. UI Components (user experience)
6. Error Handling (polish)
7. Performance Monitoring (optimization)

Please provide implementations that are:
- **Windows-specific** where needed (no cross-platform abstractions)
- **Production-ready** (handle edge cases, errors)
- **Well-tested** (include test code)
- **Performant** (include benchmarks)
- **Integrated** (work with existing module structure) 