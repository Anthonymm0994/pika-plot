# Pika-Plot

A GPU-accelerated data visualization tool with a notebook-style interface, built in Rust.

## Project Status

**Current State**: Core functionality implemented and tested ✅
- ✅ **pika-core**: Complete with comprehensive tests (13 tests passing)
- ✅ **pika-engine**: Complete with comprehensive tests (14 tests passing) 
- ✅ **Integration tests**: 10 comprehensive integration tests covering full workflow
- 🚧 **pika-ui**: Partial implementation (compilation errors need fixing)
- 🚧 **pika-app**: Not yet implemented
- 🚧 **pika-cli**: Basic structure in place

## Architecture

The project is organized as a Rust workspace with multiple crates:

### Core Libraries (✅ Working)

- **`pika-core`**: Core types, error handling, events, and data structures
- **`pika-engine`**: Data processing engine with DuckDB integration, GPU acceleration, and plot rendering

### User Interface (🚧 In Progress)

- **`pika-ui`**: egui-based user interface components
- **`pika-app`**: Main application binary
- **`pika-cli`**: Command-line interface

## Features Implemented

### Data Processing
- ✅ DuckDB integration for SQL queries
- ✅ CSV import with configurable options
- ✅ Arrow-based data handling
- ✅ Memory management and monitoring
- ✅ Concurrent query execution
- ✅ Query validation and error handling

### Plot System
- ✅ Comprehensive plot type definitions (25+ plot types)
- ✅ Plot configuration system
- ✅ GPU-accelerated rendering infrastructure
- ✅ Data extraction from Arrow arrays
- ✅ Plot bounds calculation and rendering modes

### Workspace Management
- ✅ Workspace snapshots for save/load
- ✅ Node-based canvas system
- ✅ Event system for UI-Engine communication
- ✅ Memory coordination and limits

### Testing
- ✅ Unit tests for all core modules
- ✅ Integration tests covering complete workflows
- ✅ Database operations testing
- ✅ Concurrent operations testing
- ✅ Error handling validation

## Quick Start

### Prerequisites

- Rust 1.88+ (stable toolchain)
- Git

### Building

```bash
# Clone the repository
git clone <repository-url>
cd pika-plot

# Run tests for core functionality
cargo test --package pika-core
cargo test --package pika-engine

# Run integration tests
cargo test --package pika-engine --test integration_tests

# Build core libraries (working)
cargo build --package pika-core
cargo build --package pika-engine
```

### Running Tests

```bash
# All core tests
cargo test --package pika-core --package pika-engine

# Specific test suites
cargo test --package pika-core --lib
cargo test --package pika-engine --lib
cargo test --package pika-engine --test integration_tests
```

## Example Usage

```rust
use pika_engine::{Database, QueryEngine};
use pika_core::plots::PlotConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database and query engine
    let db = Arc::new(Mutex::new(Database::new().await?));
    let query_engine = QueryEngine::new(db.clone());
    
    // Import CSV data
    let import_sql = "CREATE TABLE data AS SELECT * FROM read_csv_auto('data.csv')";
    {
        let database = db.lock().await;
        database.execute(import_sql).await?;
    }
    
    // Execute queries
    let result = query_engine.execute("SELECT * FROM data WHERE value > 100").await?;
    println!("Found {} rows", result.row_count);
    
    // Create plot configuration
    let plot_config = PlotConfig::scatter("x".to_string(), "y".to_string());
    println!("Plot type: {:?}", plot_config.plot_type);
    
    Ok(())
}
```

## Development Status

### Completed Components

1. **Core Data Types** - All fundamental types for nodes, events, plots, and workspace management
2. **Database Integration** - Full DuckDB integration with async support
3. **Query Engine** - SQL execution with validation and error handling  
4. **Plot System** - Comprehensive plot type definitions and configuration
5. **Memory Management** - Memory monitoring, limits, and coordination
6. **Event System** - Publisher-subscriber pattern for UI-Engine communication
7. **Workspace Snapshots** - Save/load functionality for workspace state
8. **GPU Infrastructure** - WGPU-based rendering pipelines (ready for integration)

### Remaining Work

1. **UI Implementation** - Fix compilation errors in egui-based interface
2. **Application Integration** - Connect UI to engine
3. **CLI Implementation** - Command-line interface for batch operations
4. **Performance Optimization** - GPU rendering integration
5. **Documentation** - API documentation and user guides

## Technical Details

### Dependencies

- **Database**: DuckDB 1.3.2 with Arrow integration
- **GPU**: wgpu 0.20 for GPU acceleration
- **UI**: egui 0.28 for immediate mode GUI
- **Async**: tokio 1.46 for async runtime
- **Serialization**: serde for JSON/RON support

### Memory Management

The system includes comprehensive memory management:
- Memory coordinators track allocation
- Configurable memory limits
- Memory warnings and cleanup
- Guard-based memory protection

### Performance Features

- GPU-accelerated rendering with multiple pipelines
- Streaming data processing for large datasets
- Concurrent query execution
- Memory-efficient Arrow data handling

## Contributing

1. Focus on fixing UI compilation errors in `pika-ui/`
2. Implement missing application logic in `pika-app/`
3. Add more comprehensive CLI features
4. Improve error handling and user experience
5. Add more plot types and visualization options

## License

[License information to be added]

---

**Note**: This project demonstrates a working data processing and visualization engine. The core functionality (data import, SQL queries, plot configuration, memory management) is fully implemented and tested. The remaining work focuses on user interface implementation and application integration. 