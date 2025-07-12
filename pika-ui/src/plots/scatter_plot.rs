use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Points, Legend, MarkerShape as EguiMarkerShape};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, MarkerShape};
use pika_engine::plot::{extract_xy_points, extract_string_values};
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
        // Extract points
        let points_result = extract_xy_points(data, &self.x_column, &self.y_column);
        
        match points_result {
            Ok(points) => {
                // Extract categories if color column is specified
                let (categories, category_map) = if let Some(color_col) = &self.color_column {
                    if let Some(cat_array) = data.column_by_name(color_col) {
                        if let Ok(cats) = extract_string_values(cat_array) {
                            // Create color map
                            let mut cat_map = BTreeMap::new();
                            let unique_cats: Vec<String> = cats.iter()
                                .cloned()
                                .collect::<std::collections::HashSet<_>>()
                                .into_iter()
                                .collect();
                            
                            for (i, cat) in unique_cats.iter().enumerate() {
                                cat_map.insert(cat.clone(), categorical_color(i));
                            }
                            
                            (Some(cats), Some(cat_map))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };
                
                let plot = Plot::new("scatter_plot")
                    .legend(Legend::default())
                    .data_aspect(1.0)
                    .show_grid(self.show_grid)
                    .auto_bounds(egui::Vec2b::new(true, true));
                
                plot.show(ui, |plot_ui| {
                    if let (Some(cats), Some(cat_map)) = (&categories, &category_map) {
                        // Plot points grouped by category
                        for (category, &color) in cat_map {
                            let category_points: Vec<[f64; 2]> = points.iter()
                                .enumerate()
                                .filter(|(i, _)| cats.get(*i).map(|c| c == category).unwrap_or(false))
                                .map(|(_, &(x, y))| [x, y])
                                .collect();
                            
                            if !category_points.is_empty() {
                                let points = Points::new(PlotPoints::new(category_points))
                                    .color(color)
                                    .radius(self.point_radius)
                                    .shape(convert_marker_shape(self.marker_shape))
                                    .name(category);
                                
                                plot_ui.points(points);
                            }
                        }
                    } else {
                        // No categories, plot all points with same color
                        let plot_points: Vec<[f64; 2]> = points.iter()
                            .map(|&(x, y)| [x, y])
                            .collect();
                        
                        let points = Points::new(PlotPoints::new(plot_points))
                            .color(Color32::from_rgb(31, 119, 180))
                            .radius(self.point_radius)
                            .shape(convert_marker_shape(self.marker_shape))
                            .name(&format!("{} vs {}", self.y_column, self.x_column));
                        
                        plot_ui.points(points);
                    }
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error: {}", e));
            }
        }
    }
}

fn convert_marker_shape(shape: MarkerShape) -> EguiMarkerShape {
    match shape {
        MarkerShape::Circle => EguiMarkerShape::Circle,
        MarkerShape::Square => EguiMarkerShape::Square,
        MarkerShape::Triangle => EguiMarkerShape::Up,
        MarkerShape::Diamond => EguiMarkerShape::Diamond,
        MarkerShape::Cross => EguiMarkerShape::Cross,
        MarkerShape::Plus => EguiMarkerShape::Plus,
    }
}

fn categorical_color(index: usize) -> Color32 {
    const COLORS: &[Color32] = &[
        Color32::from_rgb(31, 119, 180),   // Blue
        Color32::from_rgb(255, 127, 14),   // Orange
        Color32::from_rgb(44, 160, 44),    // Green
        Color32::from_rgb(214, 39, 40),    // Red
        Color32::from_rgb(148, 103, 189),  // Purple
        Color32::from_rgb(140, 86, 75),    // Brown
        Color32::from_rgb(227, 119, 194),  // Pink
        Color32::from_rgb(127, 127, 127),  // Gray
        Color32::from_rgb(188, 189, 34),   // Olive
        Color32::from_rgb(23, 190, 207),   // Cyan
    ];
    
    COLORS[index % COLORS.len()]
} 