//! Violin plot implementation for statistical data visualization

use arrow::array::{Array, Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use egui::Ui;
use egui_plot::{Plot, PlotPoints, Polygon, PlotItem};
use pika_core::plots::{PlotConfig, PlotDataConfig};
use std::collections::HashMap;

/// Violin plot for showing distribution shape and statistics
pub struct ViolinPlot {
    category_column: Option<String>,
    value_column: String,
    show_box: bool,
    bandwidth: f32,
}

impl ViolinPlot {
    /// Create a new violin plot
    pub fn new(category_column: Option<String>, value_column: String) -> Self {
        Self {
            category_column,
            value_column,
            show_box: true,
            bandwidth: 1.0,
        }
    }

    /// Create from plot configuration
    pub fn from_config(config: &PlotConfig) -> Self {
        if let PlotDataConfig::ViolinConfig {
            category_column,
            value_column,
            show_box,
            bandwidth,
        } = &config.specific
        {
            Self {
                category_column: category_column.clone(),
                value_column: value_column.clone(),
                show_box: *show_box,
                bandwidth: *bandwidth,
            }
        } else {
            Self::new(None, "value".to_string())
        }
    }

    /// Render the violin plot
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let plot = Plot::new("violin_plot")
            .height(400.0)
            .show_axes([true, true])
            .show_grid([true, true]);

        plot.show(ui, |plot_ui| {
            if let Some(category_col) = &self.category_column {
                self.render_grouped_violins(plot_ui, data, category_col);
            } else {
                self.render_single_violin(plot_ui, data, 0.0);
            }
        });
    }

    fn render_grouped_violins(&self, plot_ui: &mut egui_plot::PlotUi, data: &RecordBatch, category_col: &str) {
        // Get category and value columns
        let category_array = data
            .column_by_name(category_col)
            .and_then(|col| col.as_any().downcast_ref::<StringArray>());
        
        let value_array = data
            .column_by_name(&self.value_column)
            .and_then(|col| col.as_any().downcast_ref::<Float64Array>());

        if let (Some(categories), Some(values)) = (category_array, value_array) {
            // Group values by category
            let mut grouped_data: HashMap<String, Vec<f64>> = HashMap::new();
            
            for i in 0..categories.len() {
                if !categories.is_null(i) && !values.is_null(i) {
                    let category = categories.value(i);
                    let value = values.value(i);
                    grouped_data.entry(category.to_string()).or_default().push(value);
                }
            }

            // Render violin for each category
            for (i, (category, values)) in grouped_data.iter().enumerate() {
                let x_pos = i as f64;
                self.render_violin_shape(plot_ui, values, x_pos, category);
            }
        }
    }

    fn render_single_violin(&self, plot_ui: &mut egui_plot::PlotUi, data: &RecordBatch, x_pos: f64) {
        // Get value column
        let value_array = data
            .column_by_name(&self.value_column)
            .and_then(|col| col.as_any().downcast_ref::<Float64Array>());

        if let Some(values) = value_array {
            let values_vec: Vec<f64> = (0..values.len())
                .filter(|&i| !values.is_null(i))
                .map(|i| values.value(i))
                .collect();
            
            self.render_violin_shape(plot_ui, &values_vec, x_pos, "");
        }
    }

    fn render_violin_shape(&self, plot_ui: &mut egui_plot::PlotUi, values: &[f64], x_pos: f64, _label: &str) {
        if values.is_empty() {
            return;
        }

        // Calculate basic statistics
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min_val = sorted_values[0];
        let max_val = sorted_values[sorted_values.len() - 1];
        let median = sorted_values[sorted_values.len() / 2];
        
        // Calculate quartiles
        let q1 = sorted_values[sorted_values.len() / 4];
        let q3 = sorted_values[3 * sorted_values.len() / 4];

        // Create simplified violin shape using histogram-like approach
        let num_bins = 20;
        let bin_width = (max_val - min_val) / num_bins as f64;
        let mut bins = vec![0; num_bins];
        
        // Count values in each bin
        for &value in values {
            let bin_idx = ((value - min_val) / bin_width).floor() as usize;
            let bin_idx = bin_idx.min(num_bins - 1);
            bins[bin_idx] += 1;
        }
        
        // Find max bin count for normalization
        let max_count = *bins.iter().max().unwrap_or(&1) as f64;
        
        // Create violin shape points
        let mut left_points = Vec::new();
        let mut right_points = Vec::new();
        
        for (i, &count) in bins.iter().enumerate() {
            let y = min_val + (i as f64 + 0.5) * bin_width;
            let width = (count as f64 / max_count) * 0.4; // Scale width
            
            left_points.push([x_pos - width, y]);
            right_points.push([x_pos + width, y]);
        }
        
        // Reverse right points to create closed polygon
        right_points.reverse();
        
        // Combine points
        let mut polygon_points = left_points;
        polygon_points.extend(right_points);
        
        // Draw violin shape
        if !polygon_points.is_empty() {
            let polygon = Polygon::new(PlotPoints::new(polygon_points))
                .fill_color(egui::Color32::from_rgba_unmultiplied(100, 150, 200, 100))
                .stroke(egui::Stroke::new(1.0, egui::Color32::BLUE));
            plot_ui.polygon(polygon);
        }
        
        // Draw box plot overlay if enabled
        if self.show_box {
            let box_width = 0.1;
            
            // Draw median line
            let median_points = PlotPoints::new(vec![
                [x_pos - box_width, median],
                [x_pos + box_width, median],
            ]);
            plot_ui.line(egui_plot::Line::new(median_points).color(egui::Color32::RED).width(2.0));
            
            // Draw quartile box
            let box_points = vec![
                [x_pos - box_width, q1],
                [x_pos + box_width, q1],
                [x_pos + box_width, q3],
                [x_pos - box_width, q3],
            ];
            let box_polygon = Polygon::new(PlotPoints::new(box_points))
                .fill_color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150))
                .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK));
            plot_ui.polygon(box_polygon);
        }
    }
}

impl Default for ViolinPlot {
    fn default() -> Self {
        Self::new(None, "value".to_string())
    }
} 