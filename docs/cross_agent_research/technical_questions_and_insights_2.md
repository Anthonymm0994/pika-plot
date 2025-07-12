# Technical Questions and Insights - Round 2

## Overview

This document contains focused technical questions for the next phase of Pika-Plot implementation, emphasizing practical concerns around testing, CLI ergonomics, and production readiness.

## 1. Testing Strategy Questions

### GPU Testing Without Hardware

**Context**: We need robust GPU tests that can run in CI without actual GPU hardware.

**Questions**:
1. What's the best approach for mocking wgpu Device and Queue for unit tests?
2. Should we use `wgpu::Instance::new(wgpu::Backends::PRIMARY)` with software rendering, or create full mock implementations?
3. How do we test GPU memory pressure scenarios without actual VRAM limits?
4. What's the recommended way to test shader compilation errors and validation?
5. Should we maintain a separate test harness that can optionally run on real GPUs when available?

**Specific Code Example Needed**:
```rust
// How to structure this for testability?
pub trait GpuDevice: Send + Sync {
    fn create_buffer(&self, desc: &BufferDescriptor) -> Buffer;
    fn create_shader_module(&self, desc: &ShaderModuleDescriptor) -> ShaderModule;
}
```

### DuckDB Integration Testing

**Context**: Testing data operations without creating/destroying databases repeatedly.

**Questions**:
1. Should we use in-memory databases for all tests, or maintain test fixtures on disk?
2. What's the best pattern for testing long-running queries without making tests slow?
3. How do we mock progress callbacks for testing UI updates?
4. Should we create a trait abstraction over DuckDB Connection for better testability?
5. What's the recommended approach for testing concurrent read/write patterns?

## 2. CLI User Experience

### Progress Indication and Interactivity

**Context**: The CLI needs to provide good feedback for long operations while remaining scriptable.

**Questions**:
1. For progress bars, should we use `indicatif` with custom themes or build our own?
2. How do we handle progress for multi-stage operations (load → process → export)?
3. Should we support `--json` output mode that emits structured progress events?
4. What's the best way to handle Ctrl+C gracefully during long operations?
5. Should the CLI support interactive mode vs batch mode detection (TTY vs pipe)?

**Specific Example Needed**:
```bash
# How should this behave?
pika import large_file.csv | pika query "SELECT * WHERE value > 100" | pika plot scatter -x time -y value
```

### Configuration Management

**Context**: Users need to configure database paths, memory limits, GPU preferences.

**Questions**:
1. Should we use `~/.config/pika/config.toml` or embed config in project directories?
2. How do we handle config precedence (CLI args > env vars > config file > defaults)?
3. Should GPU selection be automatic with override, or always explicit?
4. What format for memory limits is most intuitive (`--memory 4GB` vs `--memory 4294967296`)?
5. How do we handle config migration between versions?

## 3. Error Handling and Recovery

### Graceful Degradation Patterns

**Context**: The app should remain useful even when optimal paths fail.

**Questions**:
1. If GPU aggregation fails, how do we transparently fall back to CPU without user intervention?
2. Should we automatically retry failed operations with lower memory limits?
3. How do we communicate performance degradation to users (GPU → CPU fallback)?
4. What's the best pattern for partial results when operations are interrupted?
5. Should we implement checkpointing for very long operations?

**Specific Scenario**:
```rust
// User imports 50GB CSV, system has 16GB RAM
// How do we handle this gracefully?
```

## 4. Performance Benchmarking

### Automated Performance Testing

**Context**: We need to catch performance regressions and establish baselines.

**Questions**:
1. Should we use `criterion` for micro-benchmarks or build custom benchmark harness?
2. How do we benchmark GPU operations consistently across different hardware?
3. What's the best way to generate representative synthetic datasets for benchmarks?
4. Should benchmarks be part of CI, or run separately on dedicated hardware?
5. How do we track performance over time and detect regressions?

**Key Metrics to Track**:
- CSV import throughput (MB/s)
- Query execution time vs DuckDB baseline
- GPU aggregation speedup factor
- Memory usage per million points
- Frame render time by node count

## 5. Data Streaming Architecture

### Handling Larger-than-Memory Datasets

**Context**: Users will have datasets that don't fit in RAM or VRAM.

**Questions**:
1. Should we implement custom streaming iterators or rely on DuckDB's spilling?
2. How do we coordinate memory limits between DuckDB and GPU buffers?
3. What's the best pattern for progressive loading in the UI?
4. Should we support tiled rendering for plots with billions of points?
5. How do we handle backpressure when GPU can't keep up with data rate?

**Example Architecture Needed**:
```rust
trait DataStream {
    async fn next_batch(&mut self) -> Option<RecordBatch>;
    fn estimated_total_size(&self) -> Option<u64>;
    fn can_seek(&self) -> bool;
}
```

## 6. Windows-Specific Concerns

### File System and Path Handling

**Context**: Windows has unique challenges with paths, file locking, and permissions.

**Questions**:
1. How do we handle UNC paths and long path support (>260 chars)?
2. Should we use `\\?\` prefix automatically for long paths?
3. How do we deal with file locking when users try to import open Excel files?
4. What's the best approach for handling case-insensitive file systems?
5. Should we support Windows-style path completion in the CLI?

### GPU Driver Quirks

**Context**: Windows GPU drivers have different behavior than Linux/Mac.

**Questions**:
1. How do we handle WDDM timeout detection and recovery (TDR)?
2. Should we automatically reduce batch sizes on older drivers?
3. What's the best way to detect and work around driver bugs?
4. How do we handle laptops with switchable graphics (Intel + NVIDIA)?
5. Should we provide GPU driver version warnings/recommendations?

## 7. Plugin Architecture Considerations

### Extensibility Without Complexity

**Context**: Power users may want custom node types or visualizations.

**Questions**:
1. If we add plugins later, should they be WASM, dynamic libraries, or Lua/Rhai scripts?
2. How do we sandbox plugins for security while maintaining performance?
3. What's the minimal plugin API surface that would be useful?
4. Should plugins be able to add new plot types or just modify existing ones?
5. How do we handle plugin versioning and compatibility?

## 8. Debugging and Diagnostics

### Production Debugging Tools

**Context**: Users need to diagnose issues without developer tools.

**Questions**:
1. Should we embed a diagnostics mode that captures detailed traces?
2. What information should be in crash reports (with privacy considerations)?
3. How do we implement `--debug` mode without affecting release performance?
4. Should we support remote debugging or log streaming?
5. What's the best way to capture GPU debugging information?

**Example Diagnostic Output Needed**:
```
pika diagnose --verbose
System: Windows 11 (Build 22000)
GPU: NVIDIA RTX 3080 (Driver 531.41)
Memory: 16GB (8GB available)
DuckDB: v0.10.0
Dataset: 1.2M rows, 14 columns
Status: GPU aggregation failed, fell back to CPU
Suggestion: Update GPU driver to 535.xx or later
```

## 9. Real-World Dataset Handling

### Common Data Quality Issues

**Context**: Real data is messy. How do we handle common problems gracefully?

**Questions**:
1. How should we handle CSV files with inconsistent column counts?
2. What's the best UX for data type inference failures?
3. Should we auto-detect and handle common encoding issues (UTF-8 with BOM, Latin-1)?
4. How do we communicate data quality issues without overwhelming users?
5. Should we provide data cleaning suggestions or just report issues?

## 10. Crate Ecosystem Questions

### Dependencies and Maintenance

**Context**: Choosing crates that will be maintained and performant.

**Questions**:
1. For date/time handling, should we use `chrono` or `time` crate?
2. Is `color-eyre` worth it for better error reports, or stick with `anyhow`?
3. Should we use `tracing` with multiple subscribers or simpler `log` crate?
4. For CLI tables, `comfy-table` vs `tabled` vs `prettytable-rs`?
5. Should we vendor critical dependencies to ensure reproducible builds?

## Conclusion

These questions focus on the practical challenges of building a production-ready tool. The answers will help guide implementation decisions and ensure we build something that works well in real-world scenarios.

**Priority Areas for Next Sprint**:
1. Testing infrastructure (mocking GPU, DuckDB fixtures)
2. CLI polish (progress, configuration, error messages)  
3. Memory management (streaming, graceful degradation)
4. Windows-specific robustness
5. Performance benchmarking framework

Please have the other agents focus on 2-3 questions from their areas of expertise, providing concrete code examples where possible. 