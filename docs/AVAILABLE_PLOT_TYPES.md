# Available Plot Types in Pika-Plot

## Currently Implemented in Core (pika-core/src/plots.rs)

The following plot types are defined in the core architecture:

### Basic Plots
- **Scatter**: Basic 2D scatter plots with customizable markers
- **Line**: Line charts with multiple interpolation options (Linear, Smooth, Step)
- **Bar**: Vertical/horizontal bar charts with stacking support
- **Histogram**: Distribution plots with various binning strategies

### Statistical Plots
- **BoxPlot**: Box and whisker plots for statistical analysis
- **Violin**: Violin plots combining box plots with kernel density
- **Heatmap**: 2D density/value visualization with color scales
- **Correlation**: Correlation matrix visualization

### 3D Plots
- **Scatter3D**: 3D scatter plots with rotation and zoom
- **Surface3D**: 3D surface plots with wireframe options
- **Contour**: Contour plots for 2D/3D data

### Time Series
- **TimeSeries**: Time-based line charts with aggregation
- **Candlestick**: Financial OHLC charts
- **Stream**: Stream graphs for temporal data

### Hierarchical
- **Treemap**: Hierarchical data visualization
- **Sunburst**: Radial hierarchical visualization
- **Sankey**: Flow diagrams

### Network/Graph
- **Network**: Node-link network visualizations

### Specialized
- **Radar**: Radar/spider charts for multivariate data
- **Polar**: Polar coordinate plots
- **ParallelCoordinates**: Multi-dimensional data visualization
- **Geo**: Geographic/map-based visualizations

### Analysis
- **Anomaly**: Anomaly detection visualizations
- **Distribution**: Distribution analysis plots

## Available in Frog-viz Reference (frog-viz/crates/dv-views/src/plots/)

Frog-viz provides a comprehensive implementation of all the above plot types with additional features:

### Implementation Status
All plot types listed above have full implementations in frog-viz with:
- Configuration structures
- View implementations
- Interaction handlers
- Export capabilities

### Additional Features in Frog-viz
- **Utils**: Common plotting utilities and helpers
- **Time Analysis**: Advanced time series analysis beyond basic plots
- **Enhanced interactivity**: Brush selection, zoom, pan
- **Professional styling**: Multiple color schemes and themes

## UI Implementation Status

Currently, the pika-ui plot modules are temporarily disabled due to dependency conflicts. The plot types are defined in the core but need lightweight implementations without heavy dependencies like polars/arrow.

### Next Steps for Plot Implementation
1. Create lightweight plot renderers using egui primitives
2. Implement GPU-accelerated rendering for large datasets
3. Add interactive features (zoom, pan, selection)
4. Support real-time data updates
5. Enable export to various formats (PNG, SVG, PDF)

## Plot Configuration

Each plot type supports extensive configuration through the `PlotDataConfig` enum:

```rust
pub enum PlotDataConfig {
    ScatterConfig { /* fields */ },
    LineConfig { /* fields */ },
    BarConfig { /* fields */ },
    // ... etc
}
```

Common configuration options include:
- Data column mappings
- Visual styling (colors, sizes, shapes)
- Interaction settings
- Aggregation methods
- Animation options 