# Cross-Agent Technical Research for Pika-Plot

## Overview

This document synthesizes insights from multiple AI agents (Gemini 2.5 Pro, Claude Opus 4, GPT 4.5, and Grok 4) regarding Pika-Plot implementation challenges. It highlights areas of consensus, disagreement, and provides focused technical questions for future cross-agent collaboration.

## Areas of Consensus

### 1. GPU Buffer Alignment (✅ Full Agreement)

All agents agree on enforcing 256-byte alignment for GPU buffers:
- **Required** for uniform/storage buffers (not vertex/index buffers)
- Prevents runtime validation errors and performance penalties
- Older GPUs benefit most from explicit alignment
- Implementation: Query device limits and align dynamically

```rust
// Consensus implementation pattern
const ALIGNMENT: u64 = 256;
let aligned_size = ((size + ALIGNMENT - 1) / ALIGNMENT) * ALIGNMENT;
```

### 2. Dedicated Test Machine (✅ Full Agreement)

Universal agreement on setting up a Windows 10 machine with GTX 960:
- Essential for catching driver quirks early
- Should be integrated into CI/CD pipeline
- Helps establish performance baselines
- Tools: RenderDoc for GPU captures, automated smoke tests

### 3. Spatial Indexing Benefits (✅ Agreement with variance)

All agents agree spatial indexing becomes beneficial, but thresholds vary:
- **GPT 4.5**: ~200 nodes
- **Gemini 2.5 Pro**: 200-500 nodes (break-even point)
- **Grok 4**: ~500+ nodes
- **Claude Opus 4**: Implement from start for future-proofing

**Consensus**: Implement when reaching 200-300 nodes, using `rstar` crate.

## Areas of Disagreement

### 1. DuckDB Progress Monitoring

Different approaches suggested:

| Agent | Approach | Pros | Cons |
|-------|----------|------|------|
| **Gemini 2.5 Pro** | Native callbacks via `register_progress_callback` | Clean, low-overhead | Requires FFI wrapper |
| **GPT 4.5** | Polling with timeout | Simple to implement | Higher overhead |
| **Grok 4** | FFI with C callbacks | Direct integration | Complex setup |
| **Claude Opus 4** | Not specifically addressed | - | - |

**Recommendation**: Try Gemini's callback approach first, fall back to polling if unstable.

### 2. Memory-Mapped Arrow Files

Strong disagreement on safety:

| Agent | Position | Rationale |
|-------|----------|-----------|
| **Gemini 2.5 Pro** | **Avoid it** | Risk of UB, minimal performance benefit |
| **Claude Opus 4** | Use with care | Common in production, needs encapsulation |
| **GPT 4.5** | Careful use OK | Must audit with MIRI |
| **Grok 4** | **Avoid it** | Windows file locking issues |

**Recommendation**: Start without mmap, benchmark actual I/O bottlenecks first.

## Focused Technical Questions for Future Agents

### 1. GPU Compute Optimization

**Context**: We're using wgpu for GPU aggregation on discrete GPUs only.

**Questions**:
- What's the optimal workgroup size for aggregation kernels on modern discrete GPUs?
- Should we use subgroup operations despite limited wgpu support?
- How do we efficiently handle variable-length data in compute shaders?
- What's the best strategy for GPU-side sorting (bitonic vs radix)?

### 2. DuckDB-Arrow Zero-Copy Integration

**Context**: We need efficient data transfer between DuckDB and GPU memory.

**Questions**:
- Can we use DuckDB's Arrow interface to directly map buffers to GPU memory?
- What's the overhead of DuckDB's `query_arrow` vs custom result handling?
- How do we handle schema evolution without full re-queries?
- Is there a way to stream partial results from DuckDB to GPU progressively?

### 3. Canvas Rendering Architecture

**Context**: Egui canvas with potentially thousands of nodes and edges.

**Questions**:
- Should we use immediate mode or retained mode for node rendering?
- How do we efficiently batch draw calls for edges with different styles?
- What's the best approach for GPU-accelerated edge routing (curved vs straight)?
- How do we handle level-of-detail (LOD) for zoomed-out views?

### 4. Memory Pressure Handling

**Context**: Windows-only, 4-16GB typical user RAM, large datasets.

**Questions**:
- How do we accurately measure GPU memory usage via wgpu?
- What's the best strategy for cache eviction under memory pressure?
- Should we implement a custom allocator for better memory tracking?
- How do we coordinate memory limits between DuckDB and GPU buffers?

### 5. Async Architecture Patterns

**Context**: Tokio-based async with UI responsiveness requirements.

**Questions**:
- Should we use separate runtimes for compute vs I/O tasks?
- How do we prevent priority inversion between UI and background tasks?
- What's the optimal channel buffer size for event passing?
- Should we implement custom executors for GPU command submission?

## Implementation Priorities

Based on agent consensus, prioritize in this order:

1. **GPU Resource Manager** with 256-byte alignment
2. **DuckDB async wrapper** with spawn_blocking
3. **Basic caching** with moka
4. **Spatial indexing** preparation (implement at ~250 nodes)
5. **Test infrastructure** with GTX 960 machine

## Performance Benchmarking Questions

Ask future agents to provide specific benchmarks for:

1. **Data Loading**: CSV vs Parquet vs Arrow throughput on 1GB files
2. **GPU Transfer**: Optimal batch sizes for CPU->GPU transfers
3. **Aggregation**: Compare GPU vs DuckDB for common operations
4. **Rendering**: Frame time targets for various node counts
5. **Memory**: Peak usage patterns during typical workflows

## Architecture Decision Points

Future agents should weigh in on:

1. **Plugin Architecture**: Should we support custom node types via WASM?
2. **Scripting**: Python bindings vs embedded Lua/Rhai?
3. **Networking**: Future multi-user support considerations?
4. **File Format**: Custom binary format vs pure DuckDB storage?
5. **Undo/Redo**: Command pattern vs snapshot-based?

## Testing Strategy Questions

1. **GPU Testing**: How to mock wgpu for unit tests?
2. **Integration Tests**: Best practices for DuckDB fixtures?
3. **Performance Tests**: Automated regression detection?
4. **UI Testing**: Headless egui testing strategies?
5. **Fuzz Testing**: Which components benefit most?

## Conclusion

This document should be shared with future AI agents to gather diverse perspectives on these technical challenges. Focus on concrete, implementation-ready solutions rather than theoretical discussions. Each agent should pick 2-3 questions from their area of expertise and provide detailed, code-level guidance.

**Key Principle**: Gather multiple viewpoints, synthesize the best approaches, and make decisions based on practical constraints rather than theoretical optimality. 