use egui::{Ui, Color32};
use egui_plot::{Plot, Bar, BarChart, Line, PlotPoints, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, BinStrategy};
use pika_engine::plot::extract_numeric_values;

pub struct HistogramPlot {
    column: String,
    num_bins: usize,
    bin_strategy: BinStrategy,
    show_density: bool,
    show_normal: bool,
    show_legend: bool,
    show_grid: bool,
}

impl HistogramPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::HistogramConfig {
                column,
                num_bins,
                bin_strategy,
                show_density,
                show_normal,
            } => Self {
                column: column.clone(),
                num_bins: *num_bins,
                bin_strategy: *bin_strategy,
                show_density: *show_density,
                show_normal: *show_normal,
                show_legend: true,
                show_grid: true,
            },
            _ => panic!("Invalid config for histogram plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let array = match data.column_by_name(&self.column) {
            Some(arr) => arr,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.column));
                return;
            }
        };
        
        let values_result = extract_numeric_values(array);
        
        match values_result {
            Ok(values) => {
                let valid_values: Vec<f64> = values.into_iter()
                    .filter(|v| !v.is_nan())
                    .collect();
                
                if valid_values.is_empty() {
                    ui.label("No valid numeric values to display");
                    return;
                }
                
                // Calculate statistics
                let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
                let variance = valid_values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / valid_values.len() as f64;
                let std_dev = variance.sqrt();
                let min = valid_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = valid_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                
                // Determine number of bins
                let num_bins = match self.bin_strategy {
                    BinStrategy::Fixed => self.num_bins,
                    BinStrategy::Sturges => (1.0 + (valid_values.len() as f64).log2()).ceil() as usize,
                    BinStrategy::Scott => {
                        let h = 3.5 * std_dev / (valid_values.len() as f64).powf(1.0/3.0);
                        ((max - min) / h).ceil() as usize
                    }
                    _ => self.num_bins,
                }.max(1);
                
                // Create bins
                let bin_width = (max - min) / num_bins as f64;
                let mut bins = vec![0; num_bins];
                
                for &value in &valid_values {
                    let bin_idx = ((value - min) / bin_width).floor() as usize;
                    let bin_idx = bin_idx.min(num_bins - 1);
                    bins[bin_idx] += 1;
                }
                
                // Show statistics
                ui.horizontal(|ui| {
                    ui.label(format!("Count: {}", valid_values.len()));
                    ui.separator();
                    ui.label(format!("Mean: {:.2}", mean));
                    ui.separator();
                    ui.label(format!("Std Dev: {:.2}", std_dev));
                    ui.separator();
                    ui.label(format!("Min: {:.2}", min));
                    ui.separator();
                    ui.label(format!("Max: {:.2}", max));
                });
                
                let plot = Plot::new("histogram_plot")
                    .legend(Legend::default())
                    .show_grid(self.show_grid)
                    .auto_bounds(egui::Vec2b::new(true, true))
                    .x_axis_label(&self.column)
                    .y_axis_label(if self.show_density { "Density" } else { "Count" });
                
                plot.show(ui, |plot_ui| {
                    // Draw histogram bars
                    let mut bars = Vec::new();
                    for (i, &count) in bins.iter().enumerate() {
                        let center = min + (i as f64 + 0.5) * bin_width;
                        let height = if self.show_density {
                            count as f64 / (valid_values.len() as f64 * bin_width)
                        } else {
                            count as f64
                        };
                        
                        bars.push(
                            Bar::new(center, height)
                                .width(bin_width)
                                .fill(Color32::from_rgba_unmultiplied(92, 140, 97, 180))
                        );
                    }
                    
                    plot_ui.bar_chart(
                        BarChart::new(bars)
                            .color(Color32::from_rgb(92, 140, 97))
                            .name("Histogram")
                    );
                    
                    // Draw normal distribution overlay if enabled
                    if self.show_normal {
                        let num_points = 100;
                        let mut normal_curve = Vec::new();
                        
                        for i in 0..=num_points {
                            let x = min + (max - min) * i as f64 / num_points as f64;
                            let z = (x - mean) / std_dev;
                            let y = (-0.5 * z * z).exp() / (std_dev * 2.5066282746310002);
                            normal_curve.push([x, y]);
                        }
                        
                        plot_ui.line(
                            Line::new(PlotPoints::new(normal_curve))
                                .color(Color32::from_rgb(100, 100, 255))
                                .width(2.0)
                                .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                                .name("Normal")
                        );
                    }
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error: {}", e));
            }
        }
    }
} 