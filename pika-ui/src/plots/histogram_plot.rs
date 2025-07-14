use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Polygon, Legend, PlotItem};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig};
use pika_engine::plot::extract_numeric_values;
use crate::theme::{PlotTheme, get_theme_mode};

pub struct HistogramPlot {
    column: String,
    bins: usize,
    bin_width: Option<f64>,
    show_density: bool,
    show_legend: bool,
}

impl HistogramPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::HistogramConfig {
                column,
                num_bins,
                bin_strategy: _,
                show_density,
                show_normal: _,
            } => Self {
                column: column.clone(),
                bins: *num_bins,
                bin_width: None,
                show_density: *show_density,
                show_legend: true,
            },
            _ => panic!("Invalid config for histogram plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        // Get the column array
        let column_array = match data.column_by_name(&self.column) {
            Some(array) => array,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.column));
                return;
            }
        };
        
        match extract_numeric_values(column_array) {
            Ok(values) => {
                if values.is_empty() {
                    ui.colored_label(Color32::YELLOW, "No data to display");
                    return;
                }
                
                // Calculate histogram
                let (min_val, max_val) = values.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &val| {
                    (min.min(val), max.max(val))
                });
                
                let bin_width = self.bin_width.unwrap_or((max_val - min_val) / self.bins as f64);
                let mut bins = vec![0; self.bins];
                
                for &value in &values {
                    let bin_index = ((value - min_val) / bin_width).floor() as usize;
                    let bin_index = bin_index.min(self.bins - 1);
                    bins[bin_index] += 1;
                }
                
                // Convert to density if requested
                let total_area = values.len() as f64 * bin_width;
                let scale = if self.show_density { 1.0 / total_area } else { 1.0 };
                
                // Create plot
                let mut plot = Plot::new("histogram_plot")
                    .legend(Legend::default())
                    .show_grid(true)
                    .show_axes([true, true]);
                
                // Apply theme colors to plot
                if theme_mode == crate::theme::ThemeMode::Dark {
                    plot = plot.show_background(false);
                }
                
                plot.show(ui, |plot_ui| {
                    // Create histogram bars as polygons
                    let color = plot_theme.categorical_color(0);
                    let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 128);
                    
                    for (i, &count) in bins.iter().enumerate() {
                        let x_left = min_val + i as f64 * bin_width;
                        let x_right = x_left + bin_width;
                        let height = count as f64 * scale;
                        
                        if height > 0.0 {
                            let points = vec![
                                [x_left, 0.0],
                                [x_right, 0.0],
                                [x_right, height],
                                [x_left, height],
                            ];
                            
                            let polygon = Polygon::new(PlotPoints::new(points))
                                .fill_color(fill_color)
                                .stroke(egui::Stroke::new(1.0, color))
                                .width(1.0);
                            
                            plot_ui.polygon(polygon);
                        }
                    }
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error rendering histogram: {}", e));
            }
        }
    }
} 