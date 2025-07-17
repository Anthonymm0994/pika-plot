use egui::{Ui, Color32};
use egui_plot::{Bar, BarChart, Plot, Legend};
use datafusion::arrow::datatypes::DataType;

use super::{Plot as PlotTrait, PlotData};

pub struct HistogramPlot;

impl PlotTrait for HistogramPlot {
    fn name(&self) -> &'static str {
        "Histogram"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Histograms don't require an X column - they compute bins from Y values
        None
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        // Y axis must be numeric for histogram calculation
        vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float16, DataType::Float32, DataType::Float64,
        ]
    }
    
    fn validate_columns(&self, query_result: &crate::core::QueryResult, x_col: &str, y_col: &str) -> Result<(), String> {
        // Histogram only needs Y column
        if y_col.is_empty() {
            return Err("Y column is required for histogram".to_string());
        }
        Ok(())
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
            });
            return;
        }
        
        // Calculate histogram bins
        let values: Vec<f64> = data.points.iter().map(|p| p.y).collect();
        let bins = calculate_histogram_bins(&values, 20); // 20 bins by default
        
        if bins.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data to display");
            });
            return;
        }
        
        let bin_width = if bins.len() > 1 {
            bins[1].0 - bins[0].0
        } else {
            1.0 // Default width for single bin
        };
        
        let bars: Vec<Bar> = bins.iter()
            .map(|(bin_center, count)| {
                Bar::new(*bin_center, *count as f64)
                    .width(bin_width)
            })
            .collect();
        
        let chart = BarChart::new(bars)
            .name(&data.title)
            .color(Color32::from_rgb(150, 150, 250));
        
        let plot = Plot::new("histogram")
            .x_axis_label("Value")
            .y_axis_label("Frequency")
            .show_grid(data.show_grid);
        
        let plot = if data.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }
}

fn calculate_histogram_bins(values: &[f64], num_bins: usize) -> Vec<(f64, usize)> {
    if values.is_empty() {
        return vec![];
    }
    
    let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if min_val == max_val {
        return vec![(min_val, values.len())];
    }
    
    let bin_width = (max_val - min_val) / num_bins as f64;
    let mut bins = vec![0; num_bins];
    
    for &value in values {
        let bin_idx = ((value - min_val) / bin_width).floor() as usize;
        let bin_idx = bin_idx.min(num_bins - 1); // Handle edge case where value == max_val
        bins[bin_idx] += 1;
    }
    
    bins.into_iter()
        .enumerate()
        .map(|(i, count)| {
            let bin_center = min_val + (i as f64 + 0.5) * bin_width;
            (bin_center, count)
        })
        .collect()
} 