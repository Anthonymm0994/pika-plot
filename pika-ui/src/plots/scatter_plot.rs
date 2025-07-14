use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Points, Legend, MarkerShape as EguiMarkerShape};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, MarkerShape};
use pika_engine::plot::{extract_xy_points, extract_string_values};
use crate::theme::{PlotTheme, get_theme_mode};
use std::collections::BTreeMap;
use pika_core::query::QueryResult;
use polars::prelude::DataFrame;
use polars::prelude::col;
use polars::prelude::to_ndarray;

pub struct ScatterPlot {
    x_column: String,
    y_column: String,
    color_column: Option<String>,
    size_column: Option<String>,
    point_radius: f32,
    marker_shape: MarkerShape,
    show_legend: bool,
    show_grid: bool,
}

impl ScatterPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::ScatterConfig {
                x_column,
                y_column,
                color_column,
                size_column,
                point_radius,
                marker_shape,
            } => Self {
                x_column: x_column.clone(),
                y_column: y_column.clone(),
                color_column: color_column.clone(),
                size_column: size_column.clone(),
                point_radius: *point_radius,
                marker_shape: *marker_shape,
                show_legend: true,
                show_grid: true,
            },
            _ => panic!("Invalid config for scatter plot"),
        }
    }
    
    // Ported from frog-viz scatter
    pub fn render(&self, ui: &mut Ui, df: DataFrame) {
        let points = df.select([col("x"), col("y")])?.to_ndarray()?;
        // From frog-viz: categories, colors
        let colors = /* extract */;
        Plot::new("scatter").show(ui, |p| {
            p.points(Points::new(points).color(colors));
        });
    }
}

fn convert_marker_shape(shape: MarkerShape) -> EguiMarkerShape {
    match shape {
        MarkerShape::Circle => EguiMarkerShape::Circle,
        MarkerShape::Square => EguiMarkerShape::Square,
        MarkerShape::Diamond => EguiMarkerShape::Diamond,
        MarkerShape::Triangle => EguiMarkerShape::Up,
        MarkerShape::Cross => EguiMarkerShape::Cross,
        MarkerShape::Plus => EguiMarkerShape::Plus,
    }
} 