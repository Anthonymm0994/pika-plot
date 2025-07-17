//! Plot implementations module
//! 
//! This module contains all the plot implementations ported from frog-viz,
//! with proper column type validation using DataFusion's type system.

use datafusion::arrow::datatypes::DataType;
use egui::Ui;
use crate::core::QueryResult;

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

// Plot trait that all plots must implement
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
    
    /// Validate if the selected columns are appropriate for this plot type
    fn validate_columns(&self, query_result: &QueryResult, x_col: &str, y_col: &str) -> Result<(), String> {
        // Default validation logic
        if let Some(required_x) = self.required_x_types() {
            if x_col.is_empty() {
                return Err("X column is required for this plot type".to_string());
            }
            // TODO: Check actual column type against required types
        }
        
        if y_col.is_empty() {
            return Err("Y column is required for this plot type".to_string());
        }
        
        Ok(())
    }
    
    /// Render the plot
    fn render(&self, ui: &mut Ui, data: &PlotData);
}

/// Common plot data structure
#[derive(Debug, Clone)]
pub struct PlotData {
    pub points: Vec<PlotPoint>,
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub show_legend: bool,
    pub show_grid: bool,
}

#[derive(Debug, Clone)]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
    pub color: Option<egui::Color32>,
    pub size: Option<f32>,
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

/// Re-export commonly used items
pub use self::bar::BarChartPlot;
pub use self::line::LineChartPlot;
pub use self::scatter::ScatterPlotImpl;
pub use self::histogram::HistogramPlot;
pub use self::box_plot::BoxPlotImpl; 