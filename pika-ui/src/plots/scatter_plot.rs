use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Points, Legend, MarkerShape as EguiMarkerShape};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, MarkerShape};
use pika_engine::plot::{extract_xy_points, extract_string_values};
use crate::theme::{PlotTheme, get_theme_mode};
use std::collections::BTreeMap;

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
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        // Extract points
        let points_result = extract_xy_points(data, &self.x_column, &self.y_column);
        
        match points_result {
            Ok(points) => {
                // Convert to the format expected by PlotPoints
                let plot_points: Vec<[f64; 2]> = points.iter().map(|(x, y)| [*x, *y]).collect();
                
                // Extract categories if color column is specified
                let (categories, category_map) = if let Some(color_col) = &self.color_column {
                    if let Some(color_array) = data.column_by_name(color_col) {
                        match extract_string_values(color_array) {
                            Ok(values) => {
                                let mut unique_categories = BTreeMap::new();
                                let mut category_index = 0;
                                
                                for value in &values {
                                    if !unique_categories.contains_key(value) {
                                        unique_categories.insert(value.clone(), category_index);
                                        category_index += 1;
                                    }
                                }
                                
                                (Some(values), unique_categories)
                            }
                            Err(_) => (None, BTreeMap::new()),
                        }
                    } else {
                        (None, BTreeMap::new())
                    }
                } else {
                    (None, BTreeMap::new())
                };
                
                // Create plot
                let mut plot = Plot::new("scatter_plot")
                    .legend(Legend::default())
                    .show_grid(self.show_grid)
                    .show_axes([true, true]);
                
                // Apply theme colors to plot
                if theme_mode == crate::theme::ThemeMode::Dark {
                    plot = plot.show_background(false);
                }
                
                plot.show(ui, |plot_ui| {
                    if let Some(categories) = categories {
                        // Group points by category
                        let mut category_points: BTreeMap<String, Vec<[f64; 2]>> = BTreeMap::new();
                        
                        for (i, point) in plot_points.iter().enumerate() {
                            if let Some(category) = categories.get(i) {
                                category_points.entry(category.clone())
                                    .or_insert_with(Vec::new)
                                    .push(*point);
                            }
                        }
                        
                        // Plot each category with its own color
                        for (category, cat_points) in category_points {
                            if let Some(&color_index) = category_map.get(&category) {
                                let color = plot_theme.categorical_color(color_index);
                                let points_obj = PlotPoints::new(cat_points);
                                let points = Points::new(points_obj)
                                    .radius(self.point_radius)
                                    .color(color)
                                    .shape(convert_marker_shape(self.marker_shape))
                                    .name(category);
                                plot_ui.points(points);
                            }
                        }
                    } else {
                        // Single series with theme-appropriate color
                        let color = plot_theme.categorical_color(0);
                        let points_obj = PlotPoints::new(plot_points);
                        let points = Points::new(points_obj)
                            .radius(self.point_radius)
                            .color(color)
                            .shape(convert_marker_shape(self.marker_shape))
                            .name(format!("{} vs {}", self.x_column, self.y_column));
                        plot_ui.points(points);
                    }
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error rendering scatter plot: {}", e));
            }
        }
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