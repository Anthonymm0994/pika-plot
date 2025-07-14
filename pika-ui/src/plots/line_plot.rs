use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Line, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig};
use pika_engine::plot::extract_xy_points;
use crate::theme::{PlotTheme, get_theme_mode};

pub struct LinePlot {
    x_column: String,
    y_column: String,
    color_column: Option<String>,
    line_width: f32,
    show_points: bool,
    show_legend: bool,
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
                interpolation: _,
            } => Self {
                x_column: x_column.clone(),
                y_column: y_column.clone(),
                color_column: color_column.clone(),
                line_width: *line_width,
                show_points: *show_points,
                show_legend: true,
            },
            _ => panic!("Invalid config for line plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        match extract_xy_points(data, &self.x_column, &self.y_column) {
            Ok(mut points) => {
                // Sort points by x-coordinate for proper line drawing
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                
                // Convert to the format expected by PlotPoints
                let plot_points: Vec<[f64; 2]> = points.into_iter().map(|(x, y)| [x, y]).collect();
                
                // Create plot
                let mut plot = Plot::new("line_plot")
                    .legend(Legend::default())
                    .show_grid(true)
                    .show_axes([true, true]);
                
                // Apply theme colors to plot
                if theme_mode == crate::theme::ThemeMode::Dark {
                    plot = plot.show_background(false);
                }
                
                plot.show(ui, |plot_ui| {
                    let color = plot_theme.categorical_color(0);
                    let points_obj = PlotPoints::new(plot_points);
                    
                    let mut line = Line::new(points_obj)
                        .color(color)
                        .width(self.line_width)
                        .name(format!("{} vs {}", self.x_column, self.y_column));
                    
                    if self.show_points {
                        line = line.style(egui_plot::LineStyle::Solid);
                    }
                    
                    plot_ui.line(line);
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error rendering line plot: {}", e));
            }
        }
    }
} 