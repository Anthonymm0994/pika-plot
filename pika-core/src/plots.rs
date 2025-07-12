//! Plot type definitions and configurations.

use serde::{Deserialize, Serialize};

/// Plot types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotType {
    // Basic plots
    Scatter,
    Line,
    Bar,
    Histogram,
    
    // Statistical plots
    BoxPlot,
    Violin,
    Heatmap,
    Correlation,
    
    // Advanced plots
    Scatter3D,
    Surface3D,
    Contour,
    
    // Time series
    TimeSeries,
    Candlestick,
    Stream,
    
    // Hierarchical
    Treemap,
    Sunburst,
    Sankey,
    
    // Network
    Network,
    
    // Specialized
    Radar,
    Polar,
    ParallelCoordinates,
    Geo,
    
    // Analysis
    Anomaly,
    Distribution,
}

/// Main plot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotConfig {
    pub plot_type: PlotType,
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub width: u32,
    pub height: u32,
    pub specific: PlotDataConfig,
}

/// Plot-specific data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlotDataConfig {
    ScatterConfig {
        x_column: String,
        y_column: String,
        size_column: Option<String>,
        color_column: Option<String>,
        point_radius: f32,
        marker_shape: MarkerShape,
    },
    
    LineConfig {
        x_column: String,
        y_column: String,
        color_column: Option<String>,
        line_width: f32,
        show_points: bool,
        interpolation: LineInterpolation,
    },
    
    BarConfig {
        category_column: String,
        value_column: String,
        orientation: BarOrientation,
        bar_width: f32,
        stacked: bool,
    },
    
    HistogramConfig {
        column: String,
        num_bins: usize,
        bin_strategy: BinStrategy,
        show_density: bool,
        show_normal: bool,
    },
    
    BoxPlotConfig {
        category_column: Option<String>,
        value_column: String,
        show_outliers: bool,
        box_width: f32,
    },
    
    ViolinConfig {
        category_column: Option<String>,
        value_column: String,
        show_box: bool,
        bandwidth: f32,
    },
    
    HeatmapConfig {
        x_column: String,
        y_column: String,
        value_column: String,
        color_scale: ColorScale,
        interpolation: bool,
    },
    
    CorrelationConfig {
        columns: Vec<String>,
        method: CorrelationMethod,
        show_values: bool,
        color_scale: ColorScale,
    },
    
    Scatter3DConfig {
        x_column: String,
        y_column: String,
        z_column: String,
        color_column: Option<String>,
        size_column: Option<String>,
    },
    
    Surface3DConfig {
        x_column: String,
        y_column: String,
        z_column: String,
        color_scale: ColorScale,
        wireframe: bool,
    },
    
    TimeSeriesConfig {
        time_column: String,
        value_columns: Vec<String>,
        aggregation: TimeAggregation,
        show_range_selector: bool,
    },
    
    CandlestickConfig {
        time_column: String,
        open_column: String,
        high_column: String,
        low_column: String,
        close_column: String,
        volume_column: Option<String>,
    },
    
    TreemapConfig {
        hierarchy_columns: Vec<String>,
        value_column: String,
        color_column: Option<String>,
        tiling_method: TilingMethod,
    },
    
    SunburstConfig {
        hierarchy_columns: Vec<String>,
        value_column: String,
        color_column: Option<String>,
    },
    
    SankeyConfig {
        source_column: String,
        target_column: String,
        value_column: String,
        node_padding: f32,
    },
    
    NetworkConfig {
        node_id_column: String,
        edge_source_column: String,
        edge_target_column: String,
        node_size_column: Option<String>,
        edge_weight_column: Option<String>,
        layout: NetworkLayout,
    },
    
    RadarConfig {
        category_column: String,
        value_columns: Vec<String>,
        fill_alpha: f32,
        show_grid: bool,
    },
    
    ParallelCoordinatesConfig {
        columns: Vec<String>,
        color_column: Option<String>,
        alpha: f32,
        show_ticks: bool,
    },
    
    GeoConfig {
        location_column: String,
        value_column: Option<String>,
        location_type: LocationType,
        map_style: MapStyle,
    },
}

// Enums for configuration options

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerShape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineInterpolation {
    Linear,
    Smooth,
    Step,
    StepAfter,
    StepBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinStrategy {
    Fixed,
    Sturges,
    Scott,
    FreedmanDiaconis,
    SquareRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScale {
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Turbo,
    Rainbow,
    Blues,
    Reds,
    Greens,
    Greys,
    Diverging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationMethod {
    Pearson,
    Spearman,
    Kendall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAggregation {
    None,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TilingMethod {
    Squarify,
    Binary,
    Slice,
    Dice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkLayout {
    Force,
    Circular,
    Grid,
    Hierarchical,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    LatLon,
    Country,
    State,
    City,
    ZipCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapStyle {
    Streets,
    Satellite,
    Terrain,
    Dark,
    Light,
}

impl PlotConfig {
    /// Create a scatter plot configuration
    pub fn scatter(x: String, y: String) -> Self {
        Self {
            plot_type: PlotType::Scatter,
            title: None,
            x_label: Some(x.clone()),
            y_label: Some(y.clone()),
            width: 800,
            height: 600,
            specific: PlotDataConfig::ScatterConfig {
                x_column: x,
                y_column: y,
                size_column: None,
                color_column: None,
                point_radius: 3.0,
                marker_shape: MarkerShape::Circle,
            },
        }
    }
    
    /// Create a line plot configuration
    pub fn line(x: String, y: String) -> Self {
        Self {
            plot_type: PlotType::Line,
            title: None,
            x_label: Some(x.clone()),
            y_label: Some(y.clone()),
            width: 800,
            height: 600,
            specific: PlotDataConfig::LineConfig {
                x_column: x,
                y_column: y,
                color_column: None,
                line_width: 2.0,
                show_points: false,
                interpolation: LineInterpolation::Linear,
            },
        }
    }
    
    /// Create a bar plot configuration
    pub fn bar(category: String, value: String) -> Self {
        Self {
            plot_type: PlotType::Bar,
            title: None,
            x_label: Some(category.clone()),
            y_label: Some(value.clone()),
            width: 800,
            height: 600,
            specific: PlotDataConfig::BarConfig {
                category_column: category,
                value_column: value,
                orientation: BarOrientation::Vertical,
                bar_width: 0.7,
                stacked: false,
            },
        }
    }
    
    /// Create a histogram configuration
    pub fn histogram(column: String) -> Self {
        Self {
            plot_type: PlotType::Histogram,
            title: None,
            x_label: Some(column.clone()),
            y_label: Some("Count".to_string()),
            width: 800,
            height: 600,
            specific: PlotDataConfig::HistogramConfig {
                column,
                num_bins: 30,
                bin_strategy: BinStrategy::Fixed,
                show_density: false,
                show_normal: false,
            },
        }
    }
} 