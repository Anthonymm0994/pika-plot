use egui::{Ui, Color32};
use egui_plot::{Plot, BoxElem, BoxPlot as EguiBoxPlot, BoxSpread, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig};
use pika_engine::plot::extract_numeric_values;
use std::collections::HashMap;

pub struct BoxPlot {
    category_column: Option<String>,
    value_column: String,
    show_outliers: bool,
    box_width: f32,
    show_legend: bool,
    show_grid: bool,
}

impl BoxPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::BoxPlotConfig {
                category_column,
                value_column,
                show_outliers,
                box_width,
            } => Self {
                category_column: category_column.clone(),
                value_column: value_column.clone(),
                show_outliers: *show_outliers,
                box_width: *box_width,
                show_legend: true,
                show_grid: true,
            },
            _ => panic!("Invalid config for box plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let value_array = match data.column_by_name(&self.value_column) {
            Some(arr) => arr,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.value_column));
                return;
            }
        };
        
        let values = match extract_numeric_values(value_array) {
            Ok(v) => v,
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error: {}", e));
                return;
            }
        };
        
        // Filter out NaN values
        let mut valid_values: Vec<f64> = values.into_iter()
            .filter(|v| !v.is_nan())
            .collect();
        
        if valid_values.is_empty() {
            ui.label("No valid numeric values to display");
            return;
        }
        
        // Sort values for percentile calculation
        valid_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let plot = Plot::new("box_plot")
            .legend(Legend::default())
            .show_grid(self.show_grid)
            .auto_bounds(egui::Vec2b::new(true, true));
        
        plot.show(ui, |plot_ui| {
            // Calculate box plot statistics
            let n = valid_values.len();
            let q1_idx = n / 4;
            let median_idx = n / 2;
            let q3_idx = 3 * n / 4;
            
            let min = valid_values[0];
            let q1 = valid_values[q1_idx];
            let median = valid_values[median_idx];
            let q3 = valid_values[q3_idx];
            let max = valid_values[n - 1];
            
            // Create box element
            let box_elem = BoxElem::new(0.0, BoxSpread::new(min, q1, median, q3, max))
                .box_width(self.box_width as f64)
                .name(&self.value_column);
            
            plot_ui.box_plot(EguiBoxPlot::new(vec![box_elem])
                .color(Color32::from_rgb(92, 140, 97)));
        });
    }
} 