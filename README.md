# Pika-Plot

⚡ GPU-accelerated data visualization canvas for exploring gigabytes of data with intelligent caching.

## Overview

Pika-Plot is a high-performance data visualization tool that combines the flexibility of a node-based canvas with the power of GPU acceleration. It's designed to handle large datasets efficiently while providing an intuitive interface for data exploration and visualization.

### Key Features

- **GPU-Accelerated Rendering**: Leverages WebGPU for fast plot rendering of millions of points
- **Node-Based Canvas**: Visual programming interface for building data pipelines
- **Dual-Mode Interface**: Switch between Canvas mode (node-based) and Notebook mode (linear)
- **DuckDB Integration**: Fast in-process SQL analytics engine
- **Smart Caching**: Intelligent query and GPU resource caching
- **Real-time Streaming**: Support for streaming data processing and aggregation
- **Export Capabilities**: Export plots as PNG/SVG, data as CSV/JSON, or save entire workspaces

## Architecture

The project is organized as a Rust workspace with the following crates:

- **pika-core**: Core types, traits, and shared functionality
- **pika-engine**: Data processing engine with DuckDB integration and GPU rendering
- **pika-ui**: User interface components built with egui
- **pika-app**: Main desktop application
- **pika-cli**: Command-line interface for batch operations

## Getting Started

### Prerequisites

- Rust 1.70 or later
- A GPU with WebGPU support (for GPU acceleration)
- Windows, macOS, or Linux

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/pika-plot.git
cd pika-plot

# Build the project
cargo build --release

# Run the desktop application
cargo run --bin pika-app

# Or use the CLI
cargo run --bin pika-cli -- --help
```

### Quick Start

1. **Import Data**: Click "Import Data" or drag a CSV file onto the canvas
2. **Create a Query**: Add a Query node and connect it to your data source
3. **Visualize**: Add a Plot node and configure your visualization
4. **Export**: Save your plot as an image or export the processed data

## Usage Examples

### Desktop Application

The main application provides a visual interface for data exploration:

```bash
cargo run --bin pika-app
```

### Command Line Interface

Use the CLI for batch processing and automation:

```bash
# Import a CSV file
pika import -f data.csv -t sales_data

# Run a query
pika query -s "SELECT category, SUM(amount) FROM sales_data GROUP BY category" -f table

# Generate a plot
pika plot -q "SELECT x, y FROM data" -t scatter -x x -y y -o output.png

# Export data
pika export -s "SELECT * FROM processed_data" -o results.csv
```

## Node Types

### Data Nodes
- **Table Node**: Represents imported data tables
- **Query Node**: Execute SQL queries on connected data
- **Plot Node**: Create visualizations (scatter, line, bar, histogram)

### Connections
- Nodes connect via typed ports (Table, RecordBatch, PlotConfig)
- Visual bezier curves show data flow
- Color-coded by connection type

## Performance

Pika-Plot is designed for performance with large datasets:

- **Direct Rendering**: Up to 10,000 points rendered directly
- **Instanced Rendering**: 10,000 - 100,000 points using GPU instancing
- **Aggregated Rendering**: 100,000+ points using GPU compute shaders

## Development

### Running Tests

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p pika-core
cargo test -p pika-engine
cargo test -p pika-ui
```

### Project Structure

```
pika-plot/
├── pika-core/       # Core types and traits
├── pika-engine/     # Data processing and GPU rendering
├── pika-ui/         # User interface components
├── pika-app/        # Desktop application
├── pika-cli/        # Command-line interface
├── test_data/       # Sample data files
└── docs/            # Documentation
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) for the user interface
- Powered by [DuckDB](https://duckdb.org/) for data processing
- GPU acceleration via [wgpu](https://wgpu.rs/)
- Inspired by node-based tools like Blender's Geometry Nodes and TouchDesigner 