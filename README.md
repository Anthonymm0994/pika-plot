# Pika-Plot

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Status](https://img.shields.io/badge/status-beta-yellow.svg?style=for-the-badge)
![Tests](https://img.shields.io/badge/tests-59%20passing-brightgreen.svg?style=for-the-badge)

**A modern data visualization canvas application built with Rust**

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Documentation](#documentation)

</div>

## Overview

Pika-Plot combines the intuitive canvas interaction of tools like Excalidraw with powerful data visualization capabilities. Import your data, create stunning visualizations, and build interactive data workflows—all on an infinite canvas.

## ✨ Features

### 🎨 Canvas-First Design
- **Infinite Canvas**: Pan, zoom, and organize your data workflow spatially
- **Drawing Tools**: Rectangle, circle, line, freehand drawing, and text annotations
- **Node System**: Everything is a node—tables, plots, shapes, and text
- **Smart Connections**: Bezier curve connections with type-aware coloring

### 📊 Comprehensive Visualizations
26 plot types across multiple categories:
- **Basic**: Scatter, Line, Bar, Histogram, Area
- **Statistical**: Box Plot, Violin, Heatmap, Correlation Matrix
- **Time Series**: Time Series, Candlestick, Stream, Calendar
- **3D**: Scatter3D, Surface3D, Contour
- **Specialized**: Network, Radar, Parallel Coordinates, Geographic

### 📁 Data Management
- **CSV Import**: Smart type inference with preview
- **Column Configuration**: Set data types, primary keys, and constraints
- **Live Preview**: See your data before importing
- **Multiple Tables**: Work with multiple data sources simultaneously

### 🎯 Professional UI/UX
- **Dark Theme**: Easy on the eyes for extended use
- **Context Menus**: Right-click for quick actions
- **Keyboard Shortcuts**: Efficient workflow with standard shortcuts
- **Responsive Panels**: Resizable panels that adapt to your workflow

## 🚀 Getting Started

### Prerequisites
- Rust 1.75.0 or higher
- 8GB RAM recommended
- Windows/Linux/macOS

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/pika-plot.git
cd pika-plot

# Build the project
cargo build --release

# Run the application
cargo run -p pika-app --release
```

For platform-specific instructions, see [BUILD_AND_RUN.md](BUILD_AND_RUN.md).

## 📖 Usage

### Quick Start

1. **Import Data**
   - Click "➕ Import CSV..." in the Data Sources panel
   - Select your file and configure import settings
   - Preview and adjust column types
   - Click "Create Database"

2. **Create Visualizations**
   - Click the green "+" button next to your table to add it to canvas
   - Right-click the table node and select "Create Plot"
   - Choose your plot type and configure properties
   - See your visualization appear instantly

3. **Build Workflows**
   - Connect nodes by dragging from output to input ports
   - Add annotations with drawing tools
   - Organize your canvas with shapes and text
   - Save your workspace for later

### Canvas Controls
- **Pan**: Middle mouse button or Space + drag
- **Zoom**: Ctrl + Mouse wheel
- **Select**: Left click or drag to select multiple
- **Delete**: Select and press Delete key

## 🏗️ Architecture

Pika-Plot uses a modular architecture with clear separation of concerns:

```
pika-plot/
├── pika-core/      # Core types, events, and data structures
├── pika-engine/    # Data processing and visualization engine
├── pika-ui/        # User interface components
├── pika-app/       # Main GUI application
├── pika-cli/       # Command-line interface
└── pika-traits/    # Shared trait definitions
```

Key architectural patterns:
- **Event-Driven**: Loose coupling via broadcast channels
- **Node-Based**: Everything on canvas is a node
- **Type-Safe**: Leveraging Rust's type system
- **Async I/O**: Non-blocking file operations

## 📚 Documentation

- [Architecture Patterns](docs/ARCHITECTURE_PATTERNS.md) - Design patterns and principles
- [UI Component Guide](docs/UI_COMPONENT_GUIDE.md) - Visual guide to all UI components
- [Code Quality Guide](docs/CODE_QUALITY_GUIDE.md) - Best practices and conventions
- [Available Plot Types](docs/AVAILABLE_PLOT_TYPES.md) - All 26 visualization types
- [Project Organization](docs/PROJECT_ORGANIZATION.md) - Crate structure and modules

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p pika-ui

# Run with output
cargo test -- --nocapture
```

Current test status: **59 tests passing** ✅

## 🤝 Contributing

We welcome contributions! Please see our [Code Quality Guide](docs/CODE_QUALITY_GUIDE.md) for coding standards and best practices.

### Development Setup

```bash
# Install development dependencies
rustup component add clippy rustfmt

# Format code
cargo fmt

# Run lints
cargo clippy

# Run benchmarks
cargo bench
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built with these excellent Rust crates:
- [egui](https://github.com/emilk/egui) - Immediate mode GUI
- [tokio](https://tokio.rs/) - Async runtime
- [plotters](https://github.com/plotters-rs/plotters) - Plotting library
- [serde](https://serde.rs/) - Serialization framework

---

<div align="center">
Made with ❤️ and Rust
</div> 