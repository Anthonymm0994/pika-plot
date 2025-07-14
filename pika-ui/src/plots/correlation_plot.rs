//! Correlation plot implementation for analyzing relationships between variables

use arrow::array::{Array, Float64Array};
use arrow::record_batch::RecordBatch;
use egui::Ui;
use egui_plot::{Plot, PlotPoints, Polygon, PlotItem};
use pika_core::plots::{PlotConfig, PlotDataConfig, CorrelationMethod, ColorScale};
use std::collections::HashMap;

/// Correlation plot for showing relationships between variables
pub struct CorrelationPlot {
    columns: Vec<String>,
    method: CorrelationMethod,
    show_values: bool,
    color_scale: ColorScale,
}

impl CorrelationPlot {
    /// Create a new correlation plot
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            method: CorrelationMethod::Pearson,
            show_values: true,
            color_scale: ColorScale::Viridis,
        }
    }

    /// Create from plot configuration
    pub fn from_config(config: &PlotConfig) -> Self {
        if let PlotDataConfig::CorrelationConfig {
            columns,
            method,
            show_values,
            color_scale,
        } = &config.specific
        {
            Self {
                columns: columns.clone(),
                method: *method,
                show_values: *show_values,
                color_scale: *color_scale,
            }
        } else {
            Self::new(vec!["x".to_string(), "y".to_string()])
        }
    }

    /// Render the correlation plot
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Calculate correlation matrix
        let correlation_matrix = self.calculate_correlation_matrix(data);
        
        if correlation_matrix.is_empty() {
            ui.label("No data available for correlation analysis");
            return;
        }

        // Render as heatmap
        let plot = Plot::new("correlation_plot")
            .height(400.0)
            .width(400.0)
            .show_axes([true, true])
            .show_grid([false, false])
            .data_aspect(1.0);

        plot.show(ui, |plot_ui| {
            self.render_correlation_heatmap(plot_ui, &correlation_matrix);
        });

        // Show correlation values table if enabled
        if self.show_values {
            self.render_correlation_table(ui, &correlation_matrix);
        }
    }

    fn calculate_correlation_matrix(&self, data: &RecordBatch) -> HashMap<(String, String), f64> {
        let mut correlation_matrix = HashMap::new();
        
        // Extract numeric columns
        let mut column_data: HashMap<String, Vec<f64>> = HashMap::new();
        
        for column_name in &self.columns {
            if let Some(column) = data.column_by_name(column_name) {
                if let Some(float_array) = column.as_any().downcast_ref::<Float64Array>() {
                    let values: Vec<f64> = (0..float_array.len())
                        .filter(|&i| !float_array.is_null(i))
                        .map(|i| float_array.value(i))
                        .collect();
                    
                    if !values.is_empty() {
                        column_data.insert(column_name.clone(), values);
                    }
                }
            }
        }

        // Calculate correlations between all pairs
        for (i, col1) in self.columns.iter().enumerate() {
            for (j, col2) in self.columns.iter().enumerate() {
                if let (Some(data1), Some(data2)) = (column_data.get(col1), column_data.get(col2)) {
                    let correlation = if i == j {
                        1.0 // Perfect correlation with self
                    } else {
                        self.calculate_correlation(data1, data2)
                    };
                    
                    correlation_matrix.insert((col1.clone(), col2.clone()), correlation);
                }
            }
        }

        correlation_matrix
    }

    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        match self.method {
            CorrelationMethod::Pearson => self.pearson_correlation(x, y),
            CorrelationMethod::Spearman => self.spearman_correlation(x, y),
            CorrelationMethod::Kendall => self.kendall_correlation(x, y),
        }
    }

    fn pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut sum_sq_x = 0.0;
        let mut sum_sq_y = 0.0;
        
        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            numerator += dx * dy;
            sum_sq_x += dx * dx;
            sum_sq_y += dy * dy;
        }
        
        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn spearman_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        // Convert to ranks and calculate Pearson correlation of ranks
        let rank_x = self.rank_data(x);
        let rank_y = self.rank_data(y);
        self.pearson_correlation(&rank_x, &rank_y)
    }

    fn kendall_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        // Simplified Kendall's tau implementation
        let n = x.len();
        let mut concordant = 0;
        let mut discordant = 0;
        
        for i in 0..n {
            for j in (i + 1)..n {
                let x_diff = x[i] - x[j];
                let y_diff = y[i] - y[j];
                
                if (x_diff > 0.0 && y_diff > 0.0) || (x_diff < 0.0 && y_diff < 0.0) {
                    concordant += 1;
                } else if (x_diff > 0.0 && y_diff < 0.0) || (x_diff < 0.0 && y_diff > 0.0) {
                    discordant += 1;
                }
            }
        }
        
        let total_pairs = n * (n - 1) / 2;
        if total_pairs == 0 {
            0.0
        } else {
            (concordant as f64 - discordant as f64) / total_pairs as f64
        }
    }

    fn rank_data(&self, data: &[f64]) -> Vec<f64> {
        let mut indexed_data: Vec<(f64, usize)> = data.iter().enumerate().map(|(i, &v)| (v, i)).collect();
        indexed_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        let mut ranks = vec![0.0; data.len()];
        for (rank, &(_, original_index)) in indexed_data.iter().enumerate() {
            ranks[original_index] = rank as f64 + 1.0;
        }
        
        ranks
    }

    fn render_correlation_heatmap(&self, plot_ui: &mut egui_plot::PlotUi, correlation_matrix: &HashMap<(String, String), f64>) {
        for (i, col1) in self.columns.iter().enumerate() {
            for (j, col2) in self.columns.iter().enumerate() {
                if let Some(&correlation) = correlation_matrix.get(&(col1.clone(), col2.clone())) {
                    let color = self.correlation_to_color(correlation);
                    
                    // Draw correlation cell as a rectangle
                    let cell_points = vec![
                        [i as f64 - 0.4, j as f64 - 0.4],
                        [i as f64 + 0.4, j as f64 - 0.4],
                        [i as f64 + 0.4, j as f64 + 0.4],
                        [i as f64 - 0.4, j as f64 + 0.4],
                    ];
                    
                    let polygon = Polygon::new(PlotPoints::new(cell_points))
                        .fill_color(color)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE));
                    plot_ui.polygon(polygon);
                }
            }
        }
    }

    fn correlation_to_color(&self, correlation: f64) -> egui::Color32 {
        // Map correlation (-1 to 1) to color
        let intensity = correlation.abs();
        let red_component = if correlation > 0.0 { 
            (intensity * 255.0) as u8 
        } else { 
            0 
        };
        let blue_component = if correlation < 0.0 { 
            (intensity * 255.0) as u8 
        } else { 
            0 
        };
        
        match self.color_scale {
            ColorScale::Viridis | ColorScale::Blues => {
                egui::Color32::from_rgb(red_component, 0, blue_component)
            }
            ColorScale::Reds => {
                egui::Color32::from_rgb((intensity * 255.0) as u8, 0, 0)
            }
            ColorScale::Diverging => {
                if correlation > 0.0 {
                    egui::Color32::from_rgb((intensity * 255.0) as u8, 0, 0)
                } else {
                    egui::Color32::from_rgb(0, 0, (intensity * 255.0) as u8)
                }
            }
            _ => egui::Color32::from_rgb(red_component, 0, blue_component),
        }
    }

    fn render_correlation_table(&self, ui: &mut Ui, correlation_matrix: &HashMap<(String, String), f64>) {
        ui.separator();
        ui.label("Correlation Matrix:");
        
        egui::Grid::new("correlation_grid")
            .num_columns(self.columns.len() + 1)
            .show(ui, |ui| {
                // Header row
                ui.label(""); // Empty corner
                for col in &self.columns {
                    ui.label(col);
                }
                ui.end_row();
                
                // Data rows
                for col1 in &self.columns {
                    ui.label(col1);
                    for col2 in &self.columns {
                        if let Some(&correlation) = correlation_matrix.get(&(col1.clone(), col2.clone())) {
                            ui.label(format!("{:.3}", correlation));
                        } else {
                            ui.label("N/A");
                        }
                    }
                    ui.end_row();
                }
            });
    }
}

impl Default for CorrelationPlot {
    fn default() -> Self {
        Self::new(vec!["x".to_string(), "y".to_string()])
    }
} 