# Design Document

## Overview

The comprehensive plotting system will transform the current placeholder plot implementations in the fresh project into fully functional, interactive visualizations. The system will leverage DataFusion's columnar processing capabilities and Arrow format for efficient data handling while providing rich interactivity, proper legends, and axis labeling.

The design builds upon the existing plot architecture in fresh, which already has a solid foundation with the `Plot` trait, `PlotType` enum, and `PlotWindow` structure. We will enhance each plot implementation to provide full functionality while maintaining consistency across all visualization types.

## Architecture

### Core Components

#### 1. Enhanced Plot Trait System
The existing `Plot` trait will be extended to support advanced features:

```rust
pub trait Plot {
    fn name(&self) -> &'static str;
    fn required_x_types(&self) -> Option<Vec<DataType>>;
    fn required_y_types(&self) -> Vec<DataType>;
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)>;
    
    // New methods for enhanced functionality
    fn supports_multiple_series(&self) -> bool { false }
    fn supports_color_mapping(&self) -> bool { false }
    fn supports_size_mapping(&self) -> bool { false }
    fn supports_interactive_selection(&self) -> bool { true }
    fn get_default_config(&self) -> PlotConfiguration;
    
    fn validate_columns(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<(), String>;
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String>;
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration);
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration);
    fn handle_interaction(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) -> Option<PlotInteraction>;
}
```

#### 2. Enhanced Data Structures

**PlotConfiguration**: Extended configuration for all plot types
```rust
#[derive(Debug, Clone)]
pub struct PlotConfiguration {
    pub title: String,
    pub x_column: String,
    pub y_column: String,
    pub color_column: Option<String>,
    pub size_column: Option<String>,
    pub group_column: Option<String>,
    
    // Visual settings
    pub show_legend: bool,
    pub show_grid: bool,
    pub show_axes_labels: bool,
    pub color_scheme: ColorScheme,
    pub marker_size: f32,
    pub line_width: f32,
    
    // Interaction settings
    pub allow_zoom: bool,
    pub allow_pan: bool,
    pub allow_selection: bool,
    pub show_tooltips: bool,
    
    // Plot-specific settings
    pub plot_specific: PlotSpecificConfig,
}
```

**Enhanced PlotData**: Support for rich data representation
```rust
#[derive(Debug, Clone)]
pub struct PlotData {
    pub points: Vec<PlotPoint>,
    pub series: Vec<DataSeries>,
    pub metadata: PlotMetadata,
    pub statistics: Option<DataStatistics>,
}

#[derive(Debug, Clone)]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>, // For 3D plots
    pub label: Option<String>,
    pub color: Option<Color32>,
    pub size: Option<f32>,
    pub series_id: Option<String>,
    pub tooltip_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DataSeries {
    pub id: String,
    pub name: String,
    pub points: Vec<PlotPoint>,
    pub color: Color32,
    pub visible: bool,
    pub style: SeriesStyle,
}
```

#### 3. DataFusion Integration Layer

**DataProcessor**: Efficient data processing using DataFusion
```rust
pub struct DataProcessor {
    context: SessionContext,
}

impl DataProcessor {
    pub async fn aggregate_for_bar_chart(&self, data: &QueryResult, x_col: &str, y_col: &str) -> Result<Vec<(String, f64)>, String>;
    pub async fn compute_histogram_bins(&self, data: &QueryResult, column: &str, bins: usize) -> Result<Vec<(f64, f64, usize)>, String>;
    pub async fn compute_correlation_matrix(&self, data: &QueryResult, columns: &[String]) -> Result<Vec<Vec<f64>>, String>;
    pub async fn detect_anomalies(&self, data: &QueryResult, column: &str, method: AnomalyMethod) -> Result<Vec<bool>, String>;
    pub async fn compute_box_plot_stats(&self, data: &QueryResult, column: &str, group_by: Option<&str>) -> Result<Vec<BoxPlotStats>, String>;
}
```

### Plot Type Implementations

#### 1. Basic 2D Plots

**Bar Chart**
- Supports categorical and numeric X-axis
- Automatic data aggregation using DataFusion
- Grouped and stacked bar options
- Interactive legend with series toggle
- Hover tooltips with aggregated values

**Line Chart**
- Multiple series support with automatic color assignment
- Temporal data optimization using DataFusion date functions
- Interactive markers with data point details
- Zoom and pan functionality
- Missing data handling with gap visualization

**Scatter Plot**
- Color mapping by categorical or continuous variables
- Size mapping for bubble chart functionality
- Interactive selection with lasso and rectangle tools
- Correlation statistics display
- Density overlays for large datasets

**Histogram**
- Automatic bin size calculation using Freedman-Diaconis rule
- Overlay multiple distributions
- Kernel density estimation curves
- Statistical annotations (mean, median, std dev)
- Interactive bin adjustment

**Box Plot**
- Grouped box plots by categorical variables
- Outlier detection and highlighting
- Violin plot overlay option
- Statistical significance testing between groups
- Interactive outlier inspection

#### 2. Statistical Plots

**Correlation Matrix**
- Heatmap visualization with color-coded correlation coefficients
- Interactive cell selection showing scatter plots
- Hierarchical clustering of variables
- Statistical significance indicators
- Export correlation table functionality

**Distribution Plot**
- Multiple distribution overlays (normal, log-normal, etc.)
- Q-Q plots for distribution comparison
- Probability density and cumulative distribution functions
- Goodness-of-fit statistics
- Interactive parameter adjustment

**Anomaly Detection**
- Multiple detection algorithms (IQR, Z-score, Isolation Forest)
- Interactive threshold adjustment
- Anomaly scoring and ranking
- Time series anomaly detection for temporal data
- Export anomaly reports

#### 3. Advanced Visualizations

**3D Scatter Plot**
- Three-dimensional point clouds with rotation controls
- Color and size mapping for fourth and fifth dimensions
- Interactive 3D navigation (rotate, zoom, pan)
- Projection views (XY, XZ, YZ planes)
- 3D selection tools

**Parallel Coordinates**
- Multi-dimensional data visualization
- Interactive axis reordering
- Brushing and linking for data filtering
- Categorical and continuous axis support
- Pattern highlighting and clustering

**Sankey Diagram**
- Flow visualization between categorical variables
- Automatic layout optimization
- Interactive node and link selection
- Flow value tooltips
- Hierarchical flow representation

### Interactive Features

#### 1. Legend System
- Collapsible legend panels
- Series visibility toggle
- Color scheme selection
- Legend positioning options
- Export legend as separate image

#### 2. Tooltip System
- Context-aware tooltips with relevant data
- Multi-line tooltips for complex data
- Custom tooltip formatting
- Tooltip positioning optimization
- Persistent tooltip pinning

#### 3. Selection and Brushing
- Rectangle selection tool
- Lasso selection for irregular shapes
- Brush-and-link across multiple plots
- Selection statistics display
- Export selected data functionality

#### 4. Zoom and Pan
- Mouse wheel zoom with center-point focus
- Pan with mouse drag
- Zoom to selection rectangle
- Reset view functionality
- Synchronized zoom across linked plots

### Performance Optimizations

#### 1. DataFusion Utilization
- Push aggregations down to DataFusion query engine
- Use Arrow format for zero-copy data access
- Leverage columnar processing for statistical computations
- Implement lazy evaluation for expensive operations
- Cache computed results for repeated operations

#### 2. Rendering Optimizations
- Level-of-detail rendering for large datasets
- Viewport culling for off-screen elements
- Efficient color mapping with pre-computed palettes
- Batch rendering for similar elements
- Progressive rendering for complex visualizations

#### 3. Memory Management
- Streaming data processing for large datasets
- Efficient data structures for plot elements
- Memory pooling for frequently allocated objects
- Garbage collection optimization
- Data compression for cached results

## Components and Interfaces

### PlotWindow Enhancement
The existing `PlotWindow` will be enhanced with:
- Advanced configuration panels
- Real-time preview during configuration
- Plot export functionality
- Multiple view synchronization
- Undo/redo for configuration changes

### Configuration UI
- Tabbed configuration interface
- Visual column type indicators
- Real-time validation feedback
- Configuration templates and presets
- Import/export configuration files

### Data Validation System
- Column type compatibility checking
- Data quality assessment
- Missing data handling strategies
- Outlier detection and reporting
- Data transformation suggestions

## Data Models

### Plot-Specific Configurations
Each plot type will have specific configuration options:

```rust
#[derive(Debug, Clone)]
pub enum PlotSpecificConfig {
    BarChart(BarChartConfig),
    LineChart(LineChartConfig),
    ScatterPlot(ScatterPlotConfig),
    Histogram(HistogramConfig),
    // ... other plot types
}

#[derive(Debug, Clone)]
pub struct BarChartConfig {
    pub bar_width: f32,
    pub group_spacing: f32,
    pub stacking_mode: StackingMode,
    pub sort_order: SortOrder,
}
```

### Color Schemes
Predefined and custom color schemes:
```rust
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    Categorical(Vec<Color32>),
    Sequential(SequentialScheme),
    Diverging(DivergingScheme),
    Custom(HashMap<String, Color32>),
}
```

## Error Handling

### Validation Errors
- Clear error messages for incompatible column types
- Suggestions for alternative plot types
- Data quality warnings
- Configuration validation feedback

### Runtime Errors
- Graceful degradation for rendering failures
- Fallback visualizations for unsupported data
- Error recovery mechanisms
- User-friendly error reporting

### Data Processing Errors
- DataFusion query error handling
- Memory limit exceeded handling
- Timeout handling for long operations
- Progress indicators for lengthy computations

## Testing Strategy

### Unit Tests
- Individual plot implementation testing
- Data processing function validation
- Configuration validation testing
- Error handling verification

### Integration Tests
- End-to-end plot creation workflows
- DataFusion integration testing
- UI interaction testing
- Performance benchmarking

### Visual Regression Tests
- Plot rendering consistency verification
- Cross-platform rendering validation
- Theme and styling consistency
- Export functionality testing

### Performance Tests
- Large dataset handling benchmarks
- Memory usage profiling
- Rendering performance measurement
- DataFusion query optimization validation

### GPU-Accelerated Rendering Architecture

#### 1. GPU Rendering Layer

**GpuRenderer**: Core GPU rendering abstraction inspired by rerun and frog-viz
```rust
pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    
    // Shader pipelines
    line_pipeline: wgpu::RenderPipeline,
    point_pipeline: wgpu::RenderPipeline,
    shape_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    
    // Resource management
    vertex_buffers: HashMap<BufferId, wgpu::Buffer>,
    index_buffers: HashMap<BufferId, wgpu::Buffer>,
    uniform_buffers: HashMap<BufferId, wgpu::Buffer>,
    
    // Rendering state
    current_frame: Option<wgpu::SurfaceTexture>,
    depth_texture: wgpu::Texture,
    msaa_texture: wgpu::Texture,
}

impl GpuRenderer {
    pub fn new(window: &winit::Window) -> Result<Self, RendererError>;
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn render(&mut self, commands: &[RenderCommand]) -> Result<(), RendererError>;
    pub fn create_line_batch(&mut self, vertices: &[LineVertex]) -> BufferId;
    pub fn create_point_batch(&mut self, vertices: &[PointVertex]) -> BufferId;
    pub fn create_shape_batch(&mut self, vertices: &[ShapeVertex]) -> BufferId;
}
```

**RenderCommand**: GPU rendering commands for plot primitives
```rust
pub enum RenderCommand {
    Clear { color: [f32; 4] },
    DrawLines { 
        buffer_id: BufferId,
        count: u32,
        color: [f32; 4],
        width: f32,
        style: LineStyle,
    },
    DrawPoints {
        buffer_id: BufferId,
        count: u32,
        color: [f32; 4],
        size: f32,
        shape: PointShape,
    },
    DrawShapes {
        buffer_id: BufferId,
        count: u32,
        color: [f32; 4],
        filled: bool,
    },
    DrawText {
        text: String,
        position: [f32; 2],
        color: [f32; 4],
        size: f32,
    },
}
```

#### 2. GPU-Accelerated Plot Primitives

**Line Rendering**: Inspired by rerun's LineDrawableBuilder
```rust
pub struct GpuLineRenderer {
    renderer: GpuRenderer,
    vertex_buffer: wgpu::Buffer,
    strip_buffer: wgpu::Buffer,
    batch_info: Vec<LineBatchInfo>,
}

impl GpuLineRenderer {
    pub fn add_line_strip(&mut self, points: &[glam::Vec2], color: Color32, width: f32);
    pub fn add_line_segments(&mut self, segments: &[(glam::Vec2, glam::Vec2)], color: Color32, width: f32);
    pub fn set_line_style(&mut self, style: LineStyle);
    pub fn render(&mut self, commands: &mut Vec<RenderCommand>);
}
```

**Point Rendering**: GPU-accelerated point clouds
```rust
pub struct GpuPointRenderer {
    renderer: GpuRenderer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl GpuPointRenderer {
    pub fn add_points(&mut self, positions: &[glam::Vec2], colors: &[Color32], sizes: &[f32]);
    pub fn add_point_cloud(&mut self, points: &[PointVertex]);
    pub fn set_point_shape(&mut self, shape: PointShape);
    pub fn render(&mut self, commands: &mut Vec<RenderCommand>);
}
```

**Shape Rendering**: GPU-accelerated geometric primitives
```rust
pub struct GpuShapeRenderer {
    renderer: GpuRenderer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl GpuShapeRenderer {
    pub fn add_rectangle(&mut self, min: glam::Vec2, max: glam::Vec2, color: Color32, filled: bool);
    pub fn add_circle(&mut self, center: glam::Vec2, radius: f32, color: Color32, filled: bool);
    pub fn add_polygon(&mut self, vertices: &[glam::Vec2], color: Color32, filled: bool);
    pub fn render(&mut self, commands: &mut Vec<RenderCommand>);
}
```

#### 3. Integration with Existing Plot System

**GpuPlotRenderer**: GPU-accelerated plot rendering wrapper
```rust
pub struct GpuPlotRenderer {
    gpu_renderer: GpuRenderer,
    line_renderer: GpuLineRenderer,
    point_renderer: GpuPointRenderer,
    shape_renderer: GpuShapeRenderer,
    fallback_renderer: CpuPlotRenderer,
    
    // Capability detection
    gpu_available: bool,
    gpu_capabilities: GpuCapabilities,
}

impl GpuPlotRenderer {
    pub fn new() -> Result<Self, RendererError>;
    pub fn render_line_chart(&mut self, data: &PlotData, config: &PlotConfiguration);
    pub fn render_scatter_plot(&mut self, data: &PlotData, config: &PlotConfiguration);
    pub fn render_bar_chart(&mut self, data: &PlotData, config: &PlotConfiguration);
    pub fn render_histogram(&mut self, data: &PlotData, config: &PlotConfiguration);
    pub fn fallback_to_cpu(&mut self) -> &mut CpuPlotRenderer;
}
```

**Enhanced Plot Trait**: GPU acceleration support
```rust
pub trait Plot {
    // ... existing methods ...
    
    // New GPU acceleration methods
    fn supports_gpu_acceleration(&self) -> bool { true }
    fn render_gpu(&self, renderer: &mut GpuPlotRenderer, data: &PlotData, config: &PlotConfiguration);
    fn get_gpu_requirements(&self) -> GpuRequirements;
}

#[derive(Debug, Clone)]
pub struct GpuRequirements {
    pub max_vertices: usize,
    pub max_instances: usize,
    pub required_features: Vec<wgpu::Features>,
    pub required_limits: wgpu::Limits,
}
```

#### 4. Performance Optimization Features

**Level-of-Detail Rendering**
```rust
pub struct LodRenderer {
    levels: Vec<LodLevel>,
    current_level: usize,
    zoom_factor: f32,
}

impl LodRenderer {
    pub fn update_lod(&mut self, zoom_factor: f32, data_size: usize);
    pub fn get_visible_elements(&self, bounds: &PlotBounds) -> Vec<usize>;
    pub fn render_optimized(&mut self, renderer: &mut GpuPlotRenderer, data: &PlotData);
}
```

**Frustum Culling**
```rust
pub struct FrustumCuller {
    frustum: Frustum,
    transform: glam::Mat4,
}

impl FrustumCuller {
    pub fn update_frustum(&mut self, camera: &Camera);
    pub fn is_visible(&self, bounds: &Aabb) -> bool;
    pub fn cull_elements(&self, elements: &[PlotElement]) -> Vec<usize>;
}
```

**Async GPU Operations**
```rust
pub struct AsyncGpuRenderer {
    render_queue: Vec<RenderCommand>,
    completion_queue: Vec<RenderResult>,
    worker_thread: Option<JoinHandle<()>>,
}

impl AsyncGpuRenderer {
    pub fn submit_render(&mut self, commands: Vec<RenderCommand>);
    pub fn poll_completions(&mut self) -> Vec<RenderResult>;
    pub fn wait_for_completion(&mut self, timeout: Duration) -> Result<(), TimeoutError>;
}
```

#### 5. Fallback Strategy

**Capability Detection**
```rust
pub struct GpuCapabilityDetector {
    adapter_info: wgpu::AdapterInfo,
    features: wgpu::Features,
    limits: wgpu::Limits,
}

impl GpuCapabilityDetector {
    pub fn detect_capabilities() -> Result<GpuCapabilities, DetectionError>;
    pub fn is_supported(&self, requirements: &GpuRequirements) -> bool;
    pub fn get_fallback_options(&self) -> Vec<FallbackOption>;
}
```

**Hybrid Rendering**
```rust
pub struct HybridRenderer {
    gpu_renderer: Option<GpuPlotRenderer>,
    cpu_renderer: CpuPlotRenderer,
    current_mode: RenderMode,
}

impl HybridRenderer {
    pub fn render(&mut self, plot: &dyn Plot, data: &PlotData, config: &PlotConfiguration);
    pub fn switch_to_gpu(&mut self) -> Result<(), RendererError>;
    pub fn switch_to_cpu(&mut self);
    pub fn auto_select_mode(&mut self, data_size: usize, complexity: RenderComplexity);
}
```

#### 6. Shader Pipeline

**Line Shader**: GPU-accelerated line rendering
```glsl
// vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) strip_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    // Line strip vertex processing
    // Anti-aliasing and width calculation
}

// fragment shader
@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Anti-aliased line rendering
    // Proper caps and joins
}
```

**Point Shader**: GPU-accelerated point rendering
```glsl
// vertex shader with instancing
struct InstanceData {
    position: vec2<f32>,
    color: vec4<f32>,
    size: f32,
};

@vertex
fn vertex_main(
    @location(0) position: vec2<f32>,
    @location(1) instance_data: InstanceData,
) -> VertexOutput {
    // Instanced point rendering
    // Size and shape calculation
}
```

This GPU acceleration architecture provides:
- **High Performance**: GPU-accelerated rendering for large datasets
- **Compatibility**: Graceful fallback to CPU rendering
- **Flexibility**: Support for all existing plot types
- **Scalability**: Level-of-detail and frustum culling
- **Reliability**: Async operations and error handling