# Pika-Plot Documentation

## Overview
Pika-Plot is a data visualization tool that combines the ease of Excalidraw's canvas with data analysis capabilities.

## Quick Links
- **[Multi-Level Overview](OVERVIEW.md)** - Start here! From executive summary to technical deep-dive
- [Getting Started](#getting-started)
- [Build and Run Instructions](BUILD_AND_RUN.md)
- [Architecture Overview](ARCHITECTURE_SUMMARY.md)
- [Available Plot Types](AVAILABLE_PLOT_TYPES.md)
- [Code Quality Guide](CODE_QUALITY_GUIDE.md)
- [Project Vision](VISION.md) - Design goals and philosophy

## Architecture & Implementation

### Core Documentation
- [Architecture Summary](ARCHITECTURE_SUMMARY.md) - System overview and component relationships
- [Architecture Patterns](ARCHITECTURE_PATTERNS.md) - Design patterns and principles
- [Plot System Summary](PLOT_SYSTEM_SUMMARY.md) - Visualization engine details
- [Canvas Functionality](CANVAS_FUNCTIONALITY_UPDATE.md) - Canvas features and drawing tools
- [UX Implementation](UX_IMPLEMENTATION_SUMMARY.md) - User interface details
- [UI Component Guide](UI_COMPONENT_GUIDE.md) - Visual guide to all UI components
- [Error Handling](ERROR_HANDLING_IMPLEMENTATION_SUMMARY.md) - Error management system
- [Code Quality Guide](CODE_QUALITY_GUIDE.md) - Best practices and conventions

### Development Status
- [Build Verification Report](BUILD_VERIFICATION_REPORT.md) - Latest build and test results
- [Project Organization](PROJECT_ORGANIZATION.md) - Detailed crate architecture
- [Functionality Verification](../FUNCTIONALITY_VERIFICATION.md) - Feature compliance check
- [Final Build Status](FINAL_BUILD_STATUS.md) - Current build configuration
- [Final Success Report](FINAL_SUCCESS_REPORT.md) - Implementation completion summary

## Key Features

### Canvas System
- **Drawing Tools**: Rectangle, Circle, Line, Draw (freehand), Text
- **Node Types**: Table (with preview), Plot, Note, Shape
- **Interactions**: Pan/zoom, right-click menus, node connections
- **Smart Features**: Auto-layout, snap-to-grid, connection routing

### Data Management
- **Import**: CSV files with preview and configuration
- **Configuration**: Header detection, delimiter selection, null handling
- **Column Management**: Type inference, primary keys, constraints
- **Preview**: Live data preview with green header highlighting

### Visualization
- **26 Plot Types**: From basic (line, bar) to advanced (violin, sankey)
- **Customization**: Titles, labels, colors, themes
- **Export**: PNG, SVG formats (temporarily disabled)
- **Themes**: Light and dark modes

## Project Structure

```
pika-plot/
├── pika-app/        # Main GUI application
├── pika-cli/        # Command-line interface
├── pika-core/       # Core types and structures
├── pika-engine/     # Data processing engine
├── pika-ui/         # UI components and widgets
├── docs/            # Documentation
└── test_data/       # Sample data files
```

## Testing
- **45 Total Tests** across UI components
- Comprehensive canvas drawing tests
- Integration tests for workflows
- See [pika-ui/tests/TEST_SUMMARY.md](../pika-ui/tests/TEST_SUMMARY.md) for details

## Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/pika-plot.git
   cd pika-plot
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   ./run.bat  # Windows
   ./run.sh   # Linux/macOS
   ```

## Dependencies
- Rust 1.70+
- egui for UI
- plotters for rendering
- tokio for async operations
- serde for serialization

## Contributing
See our development guides:
- Code follows Rust best practices
- Tests required for new features
- Documentation for public APIs

## License
[License information to be added] 