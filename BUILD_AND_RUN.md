# Build and Run Instructions for Pika-Plot

## Prerequisites

1. **Rust Installation**
   - Install Rust 1.75 or later: https://rustup.rs/
   - Ensure cargo is in your PATH

2. **System Requirements**
   - Windows 10/11, macOS, or Linux
   - 8GB RAM minimum (16GB recommended for large datasets)
   - GPU with WebGPU support (optional but recommended)

## Building the Project

### Quick Build
```bash
# Clone the repository (if not already done)
git clone https://github.com/user/pika-plot.git
cd pika-plot

# Build all components
cargo build --release
```

### Individual Components
```bash
# Build GUI application only
cargo build -p pika-app --release

# Build CLI tool only  
cargo build -p pika-cli --release

# Build with all features
cargo build --all-features --release
```

## Running the Application

### GUI Application (Main Application)
```bash
# Using cargo run
cargo run -p pika-app --release

# Or run the binary directly (Windows)
./target/release/pika-plot.exe

# Or use the provided scripts
./run.bat  # Windows
./run.sh   # Linux/macOS
```

### Command Line Interface
```bash
# Run CLI with help
cargo run -p pika-cli -- --help

# Or run the binary directly
./target/release/pika --help

# Example: Import CSV
./target/release/pika import --file data.csv --table mytable

# Example: Execute query
./target/release/pika query --sql "SELECT * FROM mytable LIMIT 10"
```

## Development Mode

### Running in Development
```bash
# Run with debug info and faster compilation
cargo run -p pika-app

# Run with logging
RUST_LOG=debug cargo run -p pika-app

# Run tests
cargo test

# Check code without building
cargo check
```

### Hot Reload (for UI development)
```bash
# Install cargo-watch if not already installed
cargo install cargo-watch

# Run with auto-reload on file changes
cargo watch -x "run -p pika-app"
```

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   - Ensure you have the latest Rust version: `rustup update`
   - Clean build artifacts: `cargo clean`
   - Try building without optional features first

2. **Performance Issues**
   - Build in release mode: `cargo build --release`
   - Enable GPU acceleration if available
   - Check system resources (RAM, CPU usage)

3. **UI Scaling Issues**
   - The application supports HiDPI displays
   - Adjust scaling in your OS display settings if needed

### Platform-Specific Notes

#### Windows
- Use Git Bash or PowerShell for better terminal experience
- The provided `run.bat` script handles path setup automatically
- Administrator privileges may be needed for some features

#### macOS
- Grant necessary permissions for file access when prompted
- Use `run.sh` script or run from Terminal

#### Linux
- Ensure X11 or Wayland dependencies are installed
- May need to install additional system libraries for graphics

## Features and Modules

### Core Modules
- **pika-core**: Core data structures and types
- **pika-engine**: Data processing and analysis engine
- **pika-ui**: User interface components
- **pika-cli**: Command-line interface
- **pika-app**: Main GUI application

### Optional Features
- GPU acceleration (requires compatible hardware)
- Advanced plot types (see docs/AVAILABLE_PLOT_TYPES.md)
- Real-time collaboration features
- Jupyter notebook integration

## Project Structure
```
pika-plot/
├── pika-app/        # Main GUI application
├── pika-cli/        # Command-line interface
├── pika-core/       # Core types and structures
├── pika-engine/     # Data processing engine
├── pika-ui/         # UI components and widgets
├── docs/            # Documentation
├── test_data/       # Sample data files
├── run.bat          # Windows run script
└── run.sh           # Unix run script
```

## Next Steps

1. Run the application: `cargo run -p pika-app --release`
2. Try importing a CSV file using the Data Sources panel
3. Create visualizations by right-clicking on the canvas
4. Explore the various plot types and features
5. Check docs/ for more detailed documentation

For more information, see:
- [Available Plot Types](docs/AVAILABLE_PLOT_TYPES.md)
- [Architecture Overview](docs/ARCHITECTURE_SUMMARY.md)
- [Vision Document](docs/VISION.md) 