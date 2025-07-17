//! Plot implementations module
//! 
//! This module contains all the plot implementations ported from frog-viz,
//! with proper column type validation using DataFusion's type system.

use datafusion::arrow::datatypes::DataType;
use egui::{Ui, Color32, RichText};
use crate::core::QueryResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Import all plot modules
pub mod bar;
pub mod line;
pub mod scatter;
pub mod histogram;
pub mod box_plot;
pub mod heatmap;
pub mod violin;
pub mod anomaly;
pub mod correlation;
pub mod distribution;
pub mod scatter3d;
pub mod surface3d;
pub mod contour;
pub mod parallel_coordinates;
pub mod radar;
pub mod sankey;
pub mod treemap;
pub mod sunburst;
pub mod network;
pub mod geo;
pub mod time_analysis;
pub mod candlestick;
pub mod stream;
pub mod polar;

// DataFusion integration layer
pub mod data_processor;

// Enhanced utilities based on frog-viz patterns
pub mod utils;
pub mod enhanced_config;

// Re-export enhanced utilities
pub use utils::{categorical_color, viridis_color, plasma_color, diverging_color};
pub use utils::{calculate_quartiles, detect_outliers_iqr, zscore_outliers, calculate_statistics};
pub use utils::{extract_numeric_values, extract_string_values, extract_temporal_values};
pub use enhanced_config::{EnhancedPlotConfig, ColorScheme};

// Enhanced Plot trait with advanced functionality
pub trait Plot {
    /// Get the name of the plot type
    fn name(&self) -> &'static str;
    
    /// Get required column types for X axis (None means no X column required)
    fn required_x_types(&self) -> Option<Vec<DataType>>;
    
    /// Get required column types for Y axis
    fn required_y_types(&self) -> Vec<DataType>;
    
    /// Get optional column types for additional dimensions (e.g., size, color)
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)> {
        vec![]
    }
    
    /// Check if this plot supports multiple data series
    fn supports_multiple_series(&self) -> bool { false }
    
    /// Check if this plot supports color mapping
    fn supports_color_mapping(&self) -> bool { false }
    
    /// Check if this plot supports size mapping
    fn supports_size_mapping(&self) -> bool { false }
    
    /// Check if this plot supports interactive selection
    fn supports_interactive_selection(&self) -> bool { true }
    
    /// Get default configuration for this plot type
    fn get_default_config(&self) -> PlotConfiguration {
        PlotConfiguration {
            title: String::new(),
            x_column: String::new(),
            y_column: String::new(),
            color_column: None,
            size_column: None,
            group_column: None,
            show_legend: true,
            show_grid: true,
            show_axes_labels: true,
            color_scheme: ColorScheme::Viridis,
            marker_size: 4.0,
            line_width: 2.0,
            allow_zoom: true,
            allow_pan: true,
            allow_selection: true,
            show_tooltips: true,
            plot_specific: PlotSpecificConfig::None,
        }
    }
    
    /// Validate if the selected columns are appropriate for this plot type
    fn validate_columns(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<(), String> {
        // Enhanced validation logic
        if let Some(_required_x) = self.required_x_types() {
            if config.x_column.is_empty() {
                return Err("X column is required for this plot type".to_string());
            }
            // TODO: Check actual column type against required types
        }
        
        if config.y_column.is_empty() {
            return Err("Y column is required for this plot type".to_string());
        }
        
        // Validate optional columns
        if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() && !query_result.columns.contains(color_col) {
                return Err(format!("Color column '{}' not found in data", color_col));
            }
        }
        
        if let Some(size_col) = &config.size_column {
            if !size_col.is_empty() && !query_result.columns.contains(size_col) {
                return Err(format!("Size column '{}' not found in data", size_col));
            }
        }
        
        Ok(())
    }
    
    /// Prepare data from QueryResult into PlotData structure
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        // Default implementation - can be overridden by specific plot types
        let points = extract_plot_points(query_result, config)?;
        
        Ok(PlotData {
            points,
            series: vec![],
            metadata: PlotMetadata {
                title: config.title.clone(),
                x_label: config.x_column.clone(),
                y_label: config.y_column.clone(),
                show_legend: config.show_legend,
                show_grid: config.show_grid,
                color_scheme: config.color_scheme.clone(),
            },
            statistics: None,
        })
    }
    
    /// Render the plot
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration);
    
    /// Render legend for the plot
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if !data.series.is_empty() && config.show_legend {
            ui.group(|ui| {
                ui.label(RichText::new("Series:").strong());
                ui.separator();
                
                for series in &data.series {
                    let mut is_visible = series.visible;
                    if ui.checkbox(&mut is_visible, &series.name).changed() {
                        // Note: This would require mutable access to data, which we don't have here
                        // The actual visibility toggle should be handled in handle_interaction
                    }
                    
                    // Show series style indicator
                    ui.horizontal(|ui| {
                        match series.style {
                            SeriesStyle::Line { width: _, dashed } => {
                                let style_text = if dashed { "---" } else { "———" };
                                ui.colored_label(series.color, style_text);
                            },
                            SeriesStyle::Points { size: _, shape } => {
                                let shape_text = match shape {
                                    MarkerShape::Circle => "●",
                                    MarkerShape::Square => "■",
                                    MarkerShape::Diamond => "◆",
                                    MarkerShape::Triangle => "▲",
                                    MarkerShape::Cross => "✚",
                                    MarkerShape::Plus => "➕",
                                };
                                ui.colored_label(series.color, shape_text);
                            },
                            SeriesStyle::Bars { width: _ } => {
                                ui.colored_label(series.color, "■");
                            },
                            SeriesStyle::Area { alpha: _ } => {
                                ui.colored_label(series.color, "▬");
                            },
                        }
                        
                        if !is_visible {
                            ui.label(RichText::new(&series.name).strikethrough());
                        } else {
                            ui.label(&series.name);
                        }
                    });
                }
                
                // Show statistics if available
                if let Some(stats) = &data.statistics {
                    ui.separator();
                    ui.label(RichText::new("Statistics:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.label(format!("{}", stats.count));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mean Y:");
                        ui.label(format!("{:.3}", stats.mean_y));
                    });
                    if let Some(corr) = stats.correlation {
                        ui.horizontal(|ui| {
                            ui.label("Correlation:");
                            ui.label(format!("{:.3}", corr));
                        });
                    }
                }
            });
        }
    }
    
    /// Handle plot interactions
    fn handle_interaction(&self, _ui: &mut Ui, _data: &PlotData, _config: &PlotConfiguration) -> Option<PlotInteraction> {
        None
    }
}

/// Enhanced plot data structure with rich metadata and series support
#[derive(Debug, Clone)]
pub struct PlotData {
    pub points: Vec<PlotPoint>,
    pub series: Vec<DataSeries>,
    pub metadata: PlotMetadata,
    pub statistics: Option<DataStatistics>,
}

/// Enhanced plot point with multi-dimensional support and tooltip data
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

/// Data series for multi-series plots
#[derive(Debug, Clone)]
pub struct DataSeries {
    pub id: String,
    pub name: String,
    pub points: Vec<PlotPoint>,
    pub color: Color32,
    pub visible: bool,
    pub style: SeriesStyle,
}

/// Plot metadata containing display information
#[derive(Debug, Clone)]
pub struct PlotMetadata {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub show_legend: bool,
    pub show_grid: bool,
    pub color_scheme: ColorScheme,
}

/// Statistical information about the data
#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub mean_x: f64,
    pub mean_y: f64,
    pub std_x: f64,
    pub std_y: f64,
    pub correlation: Option<f64>,
    pub count: usize,
}

/// Series styling options
#[derive(Debug, Clone)]
pub enum SeriesStyle {
    Line { width: f32, dashed: bool },
    Points { size: f32, shape: MarkerShape },
    Bars { width: f32 },
    Area { alpha: f32 },
}

/// Marker shapes for point plots
#[derive(Debug, Clone, Copy)]
pub enum MarkerShape {
    Circle,
    Square,
    Diamond,
    Triangle,
    Cross,
    Plus,
}

/// Comprehensive plot configuration
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

impl Default for PlotConfiguration {
    fn default() -> Self {
        Self {
            title: String::new(),
            x_column: String::new(),
            y_column: String::new(),
            color_column: None,
            size_column: None,
            group_column: None,
            show_legend: true,
            show_grid: true,
            show_axes_labels: true,
            color_scheme: ColorScheme::Viridis,
            marker_size: 4.0,
            line_width: 2.0,
            allow_zoom: true,
            allow_pan: true,
            allow_selection: true,
            show_tooltips: true,
            plot_specific: PlotSpecificConfig::None,
        }
    }
}

/// Plot-specific configuration options
#[derive(Debug, Clone)]
pub enum PlotSpecificConfig {
    None,
    BarChart(BarChartConfig),
    LineChart(LineChartConfig),
    ScatterPlot(ScatterPlotConfig),
    Histogram(HistogramConfig),
    BoxPlot(BoxPlotConfig),
    Violin(ViolinPlotConfig),
}

impl PlotSpecificConfig {
    pub fn as_bar_chart(&self) -> &BarChartConfig {
        match self {
            PlotSpecificConfig::BarChart(config) => config,
            _ => panic!("Expected BarChartConfig"),
        }
    }
    
    pub fn as_line_chart(&self) -> &LineChartConfig {
        match self {
            PlotSpecificConfig::LineChart(config) => config,
            _ => panic!("Expected LineChartConfig"),
        }
    }
    
    pub fn as_scatter_plot(&self) -> &ScatterPlotConfig {
        match self {
            PlotSpecificConfig::ScatterPlot(config) => config,
            _ => panic!("Expected ScatterPlotConfig"),
        }
    }
    
    pub fn as_histogram(&self) -> &HistogramConfig {
        match self {
            PlotSpecificConfig::Histogram(config) => config,
            _ => panic!("Expected HistogramConfig"),
        }
    }
    
    pub fn as_box_plot(&self) -> &BoxPlotConfig {
        match self {
            PlotSpecificConfig::BoxPlot(config) => config,
            _ => panic!("Expected BoxPlotConfig"),
        }
    }
    
    pub fn as_violin(&self) -> &ViolinPlotConfig {
        match self {
            PlotSpecificConfig::Violin(config) => config,
            _ => panic!("Expected ViolinPlotConfig"),
        }
    }
}

/// Bar chart specific configuration
#[derive(Debug, Clone)]
pub struct BarChartConfig {
    pub bar_width: f32,
    pub group_spacing: f32,
    pub stacking_mode: StackingMode,
    pub sort_order: SortOrder,
}

impl Default for BarChartConfig {
    fn default() -> Self {
        Self {
            bar_width: 0.8,
            group_spacing: 0.2,
            stacking_mode: StackingMode::None,
            sort_order: SortOrder::None,
        }
    }
}

/// Line chart specific configuration
#[derive(Debug, Clone)]
pub struct LineChartConfig {
    pub line_style: LineStyle,
    pub show_points: bool,
    pub smooth_lines: bool,
    pub fill_area: bool,
}

impl Default for LineChartConfig {
    fn default() -> Self {
        Self {
            line_style: LineStyle::Solid,
            show_points: true,
            smooth_lines: false,
            fill_area: false,
        }
    }
}

/// Scatter plot specific configuration
#[derive(Debug, Clone)]
pub struct ScatterPlotConfig {
    pub point_shape: MarkerShape,
    pub show_trend_line: bool,
    pub show_density: bool,
    pub jitter_amount: f32,
}

impl Default for ScatterPlotConfig {
    fn default() -> Self {
        Self {
            point_shape: MarkerShape::Circle,
            show_trend_line: false,
            show_density: false,
            jitter_amount: 0.0,
        }
    }
}

/// Histogram specific configuration
#[derive(Debug, Clone)]
pub struct HistogramConfig {
    pub bin_count: Option<usize>,
    pub bin_width: Option<f64>,
    pub show_density: bool,
    pub show_normal_curve: bool,
}

impl Default for HistogramConfig {
    fn default() -> Self {
        Self {
            bin_count: None,
            bin_width: None,
            show_density: false,
            show_normal_curve: false,
        }
    }
}

/// Box plot specific configuration
#[derive(Debug, Clone)]
pub struct BoxPlotConfig {
    pub show_outliers: bool,
    pub show_mean: bool,
    pub notched: bool,
    pub violin_overlay: bool,
}

impl Default for BoxPlotConfig {
    fn default() -> Self {
        Self {
            show_outliers: true,
            show_mean: false,
            notched: false,
            violin_overlay: false,
        }
    }
}

/// Stacking modes for bar charts
#[derive(Debug, Clone)]
pub enum StackingMode {
    None,
    Stacked,
    Percent,
}

/// Sort orders for categorical data
#[derive(Debug, Clone)]
pub enum SortOrder {
    None,
    Ascending,
    Descending,
    ByValue,
}

/// Line styles
#[derive(Debug, Clone)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

/// Plot interaction events
#[derive(Debug, Clone)]
pub enum PlotInteraction {
    PointSelected(Vec<usize>),
    AreaSelected(f64, f64, f64, f64), // x1, y1, x2, y2
    SeriesToggled(String),
    ZoomChanged(f64, f64, f64, f64), // x1, y1, x2, y2
}

/// All available plot types
#[derive(Debug, Clone, PartialEq)]
pub enum PlotType {
    // Basic 2D plots
    BarChart,
    LineChart,
    ScatterPlot,
    Histogram,
    BoxPlot,
    HeatMap,
    ViolinPlot,
    
    // Statistical plots
    AnomalyDetection,
    CorrelationMatrix,
    DistributionPlot,
    
    // 3D plots
    Scatter3D,
    Surface3D,
    ContourPlot,
    
    // Multi-dimensional plots
    ParallelCoordinates,
    RadarChart,
    
    // Hierarchical and flow plots
    SankeyDiagram,
    Treemap,
    SunburstChart,
    NetworkGraph,
    
    // Geographic plots
    GeoPlot,
    
    // Time series analysis
    TimeAnalysis,
    
    // Financial plots
    CandlestickChart,
    
    // Utility plots
    StreamGraph,
    PolarPlot,
}

impl PlotType {
    pub fn name(&self) -> &'static str {
        match self {
            PlotType::BarChart => "Bar Chart",
            PlotType::LineChart => "Line Chart",
            PlotType::ScatterPlot => "Scatter Plot",
            PlotType::Histogram => "Histogram",
            PlotType::BoxPlot => "Box Plot",
            PlotType::HeatMap => "Heat Map",
            PlotType::ViolinPlot => "Violin Plot",
            PlotType::AnomalyDetection => "Anomaly Detection",
            PlotType::CorrelationMatrix => "Correlation Matrix",
            PlotType::DistributionPlot => "Distribution Plot",
            PlotType::Scatter3D => "3D Scatter Plot",
            PlotType::Surface3D => "3D Surface Plot",
            PlotType::ContourPlot => "Contour Plot",
            PlotType::ParallelCoordinates => "Parallel Coordinates",
            PlotType::RadarChart => "Radar Chart",
            PlotType::SankeyDiagram => "Sankey Diagram",
            PlotType::Treemap => "Treemap",
            PlotType::SunburstChart => "Sunburst Chart",
            PlotType::NetworkGraph => "Network Graph",
            PlotType::GeoPlot => "Geographic Plot",
            PlotType::TimeAnalysis => "Time Series Analysis",
            PlotType::CandlestickChart => "Candlestick Chart",
            PlotType::StreamGraph => "Stream Graph",
            PlotType::PolarPlot => "Polar Plot",
        }
    }
    
    pub fn all_types() -> Vec<Self> {
        vec![
            // Basic 2D plots
            PlotType::BarChart,
            PlotType::LineChart,
            PlotType::ScatterPlot,
            PlotType::Histogram,
            PlotType::BoxPlot,
            PlotType::HeatMap,
            PlotType::ViolinPlot,
            
            // Statistical plots
            PlotType::AnomalyDetection,
            PlotType::CorrelationMatrix,
            PlotType::DistributionPlot,
            
            // 3D plots
            PlotType::Scatter3D,
            PlotType::Surface3D,
            PlotType::ContourPlot,
            
            // Multi-dimensional plots
            PlotType::ParallelCoordinates,
            PlotType::RadarChart,
            
            // Hierarchical and flow plots
            PlotType::SankeyDiagram,
            PlotType::Treemap,
            PlotType::SunburstChart,
            PlotType::NetworkGraph,
            
            // Geographic plots
            PlotType::GeoPlot,
            
            // Time series analysis
            PlotType::TimeAnalysis,
            
            // Financial plots
            PlotType::CandlestickChart,
            
            // Utility plots
            PlotType::StreamGraph,
            PlotType::PolarPlot,
        ]
    }
    
    /// Get the categories for grouping plots in the UI
    pub fn categories() -> Vec<(&'static str, Vec<PlotType>)> {
        vec![
            ("Basic 2D", vec![
                PlotType::BarChart,
                PlotType::LineChart,
                PlotType::ScatterPlot,
                PlotType::Histogram,
                PlotType::BoxPlot,
                PlotType::HeatMap,
                PlotType::ViolinPlot,
            ]),
            ("Statistical", vec![
                PlotType::AnomalyDetection,
                PlotType::CorrelationMatrix,
                PlotType::DistributionPlot,
            ]),
            ("3D Plots", vec![
                PlotType::Scatter3D,
                PlotType::Surface3D,
                PlotType::ContourPlot,
            ]),
            ("Multi-dimensional", vec![
                PlotType::ParallelCoordinates,
                PlotType::RadarChart,
            ]),
            ("Hierarchical & Flow", vec![
                PlotType::SankeyDiagram,
                PlotType::Treemap,
                PlotType::SunburstChart,
                PlotType::NetworkGraph,
            ]),
            ("Geographic", vec![
                PlotType::GeoPlot,
            ]),
            ("Time Series", vec![
                PlotType::TimeAnalysis,
                PlotType::CandlestickChart,
            ]),
            ("Specialized", vec![
                PlotType::StreamGraph,
                PlotType::PolarPlot,
            ]),
        ]
    }
    
    /// Get required column types for X axis (None means no X column required)
    pub fn required_x_types(&self) -> Option<Vec<DataType>> {
        match self {
            // Plots that require X column
            PlotType::BarChart | PlotType::LineChart | PlotType::ScatterPlot | 
            PlotType::Scatter3D | PlotType::Surface3D | PlotType::ContourPlot |
            PlotType::TimeAnalysis | PlotType::CandlestickChart => {
                Some(vec![DataType::Utf8, DataType::Int64, DataType::Float64])
            }
            
            // Plots that don't require X column
            PlotType::Histogram | PlotType::BoxPlot | PlotType::ViolinPlot |
            PlotType::HeatMap | PlotType::CorrelationMatrix | PlotType::DistributionPlot |
            PlotType::AnomalyDetection => None,
            
            // Default: require X column
            _ => Some(vec![DataType::Utf8, DataType::Int64, DataType::Float64]),
        }
    }
    
    /// Get required column types for Y axis
    pub fn required_y_types(&self) -> Vec<DataType> {
        match self {
            // Plots that require numeric Y
            PlotType::BarChart | PlotType::LineChart | PlotType::ScatterPlot |
            PlotType::Histogram | PlotType::BoxPlot | PlotType::ViolinPlot |
            PlotType::HeatMap | PlotType::CorrelationMatrix | PlotType::DistributionPlot |
            PlotType::AnomalyDetection | PlotType::Scatter3D | PlotType::Surface3D |
            PlotType::ContourPlot => {
                vec![DataType::Int64, DataType::Float64]
            }
            
            // Default: allow any type
            _ => vec![DataType::Utf8, DataType::Int64, DataType::Float64],
        }
    }
    
    /// Check if this plot type supports the given column types
    pub fn supports_column_types(&self, x_type: Option<&DataType>, y_type: &DataType) -> bool {
        match self {
            // Numeric plots require numeric data
            PlotType::ScatterPlot | PlotType::LineChart | PlotType::Scatter3D | 
            PlotType::Surface3D | PlotType::ContourPlot => {
                is_numeric_type(y_type) && x_type.map_or(true, is_numeric_type)
            }
            
            // Bar charts can have categorical X axis
            PlotType::BarChart => {
                is_numeric_type(y_type) && x_type.map_or(true, |t| is_categorical_type(t) || is_numeric_type(t))
            }
            
            // Histograms only need numeric Y
            PlotType::Histogram | PlotType::BoxPlot | PlotType::ViolinPlot => {
                is_numeric_type(y_type)
            }
            
            // Heat maps need numeric values
            PlotType::HeatMap => is_numeric_type(y_type),
            
            // Time series need temporal X axis
            PlotType::TimeAnalysis | PlotType::CandlestickChart => {
                is_numeric_type(y_type) && x_type.map_or(false, is_temporal_type)
            }
            
            // Statistical plots have specific requirements
            PlotType::CorrelationMatrix => is_numeric_type(y_type),
            PlotType::DistributionPlot => is_numeric_type(y_type),
            PlotType::AnomalyDetection => is_numeric_type(y_type),
            
            // Default: allow any types
            _ => true,
        }
    }
}

/// Helper functions for type checking
pub fn is_numeric_type(dtype: &DataType) -> bool {
    matches!(dtype,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
        DataType::Float16 | DataType::Float32 | DataType::Float64 |
        DataType::Decimal128(_, _) | DataType::Decimal256(_, _)
    )
}

pub fn is_categorical_type(dtype: &DataType) -> bool {
    matches!(dtype, DataType::Utf8 | DataType::LargeUtf8)
}

pub fn is_temporal_type(dtype: &DataType) -> bool {
    matches!(dtype,
        DataType::Date32 | DataType::Date64 |
        DataType::Time32(_) | DataType::Time64(_) |
        DataType::Timestamp(_, _) | DataType::Duration(_) |
        DataType::Interval(_)
    )
}

/// Helper function to extract plot points from QueryResult
pub fn extract_plot_points(query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<PlotPoint>, String> {
    if config.y_column.is_empty() {
        return Err("Y column not selected".to_string());
    }

    let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
        .ok_or("Y column not found")?;
    
    let x_idx = if !config.x_column.is_empty() {
        query_result.columns.iter().position(|c| c == &config.x_column)
    } else {
        None
    };

    let color_idx = if let Some(color_col) = &config.color_column {
        if !color_col.is_empty() {
            query_result.columns.iter().position(|c| c == color_col)
        } else {
            None
        }
    } else {
        None
    };

    let size_idx = if let Some(size_col) = &config.size_column {
        if !size_col.is_empty() {
            query_result.columns.iter().position(|c| c == size_col)
        } else {
            None
        }
    } else {
        None
    };

    let mut points = Vec::new();
    let colors = get_categorical_colors(&config.color_scheme);
    let mut color_map: HashMap<String, Color32> = HashMap::new();
    let mut color_index = 0;
    
    for (row_idx, row) in query_result.rows.iter().enumerate() {
        if row.len() > y_idx {
            let y_val = row[y_idx].parse::<f64>()
                .map_err(|_| format!("Failed to parse Y value '{}' as number", row[y_idx]))?;
            
            let x_val = if let Some(x_idx) = x_idx {
                if row.len() > x_idx {
                    row[x_idx].parse::<f64>().unwrap_or(row_idx as f64)
                } else {
                    row_idx as f64
                }
            } else {
                row_idx as f64
            };

            // Handle color mapping
            let point_color = if let Some(color_idx) = color_idx {
                if row.len() > color_idx {
                    let color_value = &row[color_idx];
                    if let Some(&existing_color) = color_map.get(color_value) {
                        Some(existing_color)
                    } else {
                        let new_color = colors[color_index % colors.len()];
                        color_map.insert(color_value.clone(), new_color);
                        color_index += 1;
                        Some(new_color)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Handle size mapping
            let point_size = if let Some(size_idx) = size_idx {
                if row.len() > size_idx {
                    row[size_idx].parse::<f32>().ok()
                } else {
                    None
                }
            } else {
                None
            };

            // Create tooltip data
            let mut tooltip_data = HashMap::new();
            tooltip_data.insert("X".to_string(), x_val.to_string());
            tooltip_data.insert("Y".to_string(), y_val.to_string());
            
            if let Some(x_idx) = x_idx {
                if row.len() > x_idx {
                    tooltip_data.insert(config.x_column.clone(), row[x_idx].clone());
                }
            }
            tooltip_data.insert(config.y_column.clone(), row[y_idx].clone());
            
            if let Some(color_idx) = color_idx {
                if row.len() > color_idx {
                    tooltip_data.insert(config.color_column.as_ref().unwrap().clone(), row[color_idx].clone());
                }
            }

            points.push(PlotPoint {
                x: x_val,
                y: y_val,
                z: None,
                label: None,
                color: point_color,
                size: point_size,
                series_id: None,
                tooltip_data,
            });
        }
    }
    
    Ok(points)
}

/// Get categorical colors from color scheme
pub fn get_categorical_colors(scheme: &ColorScheme) -> Vec<Color32> {
    scheme.get_colors(10)
}

/// Re-export commonly used items
pub use self::bar::BarChartPlot;
pub use self::line::LineChartPlot;
pub use self::scatter::ScatterPlotImpl;
pub use self::histogram::HistogramPlot;
pub use self::box_plot::BoxPlotImpl;
pub use self::data_processor::{DataProcessor, AnomalyMethod, BoxPlotStats};
pub use self::heatmap::HeatmapPlot;
pub use self::violin::{ViolinPlot, ViolinPlotConfig};