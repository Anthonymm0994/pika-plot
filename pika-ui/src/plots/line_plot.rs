use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Line, Legend, LineStyle};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, LineInterpolation};
use pika_engine::plot::{extract_xy_points, extract_string_values};
use std::collections::BTreeMap;

pub struct LinePlot {
    x_column: String,
    y_column: String,
    color_column: Option<String>,
    line_width: f32,
    show_points: bool,
    interpolation: LineInterpolation,
    show_legend: bool,
    show_grid: bool,
}

impl LinePlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::LineConfig {
                x_column,
                y_column,
                color_column,
                line_width,
                show_points,
                interpolation,
            } => Self {
                x_column: x_column.clone(),
                y_column: y_column.clone(),
                color_column: color_column.clone(),
                line_width: *line_width,
                show_points: *show_points,
                interpolation: *interpolation,
                show_legend: true,
                show_grid: true,
            },
            _ => panic!("Invalid config for line plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let points_result = extract_xy_points(data, &self.x_column, &self.y_column);
        
        match points_result {
            Ok(mut points) => {
                // Sort by x for line plots
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                
                let plot = Plot::new("line_plot")
                    .legend(Legend::default())
                    .show_grid(self.show_grid)
                    .auto_bounds(egui::Vec2b::new(true, true));
                
                plot.show(ui, |plot_ui| {
                    let plot_points: Vec<[f64; 2]> = points.iter()
                        .map(|&(x, y)| [x, y])
                        .collect();
                    
                    let mut line = Line::new(PlotPoints::new(plot_points))
                        .color(Color32::from_rgb(31, 119, 180))
                        .width(self.line_width)
                        .name(&format!("{} vs {}", self.y_column, self.x_column));
                    
                    // Apply interpolation style
                    match self.interpolation {
                        LineInterpolation::Linear => {},
                        LineInterpolation::Step => {
                            line = line.style(LineStyle::Solid);
                        },
                        _ => {},
                    }
                    
                    plot_ui.line(line);
                    
                    // Show points if requested
                    if self.show_points {
                        let points_plot: Vec<[f64; 2]> = points.iter()
                            .map(|&(x, y)| [x, y])
                            .collect();
                        
                        let points = egui_plot::Points::new(PlotPoints::new(points_plot))
                            .color(Color32::from_rgb(31, 119, 180))
                            .radius(3.0);
                        
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