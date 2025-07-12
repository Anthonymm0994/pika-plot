use egui::{Ui, Color32};
use egui_plot::{Plot, Bar, BarChart, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, BarOrientation};
use pika_engine::plot::{extract_category_values, aggregate_by_category};

pub struct BarPlot {
    category_column: String,
    value_column: String,
    orientation: BarOrientation,
    bar_width: f32,
    stacked: bool,
    show_legend: bool,
    show_grid: bool,
}

impl BarPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::BarConfig {
                category_column,
                value_column,
                orientation,
                bar_width,
                stacked,
            } => Self {
                category_column: category_column.clone(),
                value_column: value_column.clone(),
                orientation: *orientation,
                bar_width: *bar_width,
                stacked: *stacked,
                show_legend: false,
                show_grid: true,
            },
            _ => panic!("Invalid config for bar plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let pairs_result = extract_category_values(data, &self.category_column, &self.value_column);
        
        match pairs_result {
            Ok(pairs) => {
                let aggregated = aggregate_by_category(pairs);
                
                // Sort categories alphabetically
                let mut sorted_cats: Vec<(String, f64)> = aggregated.into_iter().collect();
                sorted_cats.sort_by(|a, b| a.0.cmp(&b.0));
                
                let plot = Plot::new("bar_plot")
                    .show_grid(self.show_grid)
                    .auto_bounds(egui::Vec2b::new(true, true))
                    .x_axis_label(&self.category_column)
                    .y_axis_label(&self.value_column);
                
                plot.show(ui, |plot_ui| {
                    let mut bars = Vec::new();
                    
                    for (i, (cat, val)) in sorted_cats.iter().enumerate() {
                        let bar = Bar::new(i as f64, *val)
                            .width(self.bar_width as f64)
                            .name(cat)
                            .fill(Color32::from_rgb(92, 140, 97)); // Green
                        bars.push(bar);
                    }
                    
                    let chart = BarChart::new(bars)
                        .color(Color32::from_rgb(92, 140, 97));
                    
                    plot_ui.bar_chart(chart);
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error: {}", e));
            }
        }
    }
} 