//! Radar plot implementation for multi-dimensional data visualization

use arrow::array::{Array, Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use egui::Ui;
use egui_plot::{Plot, PlotPoints, Line, Polygon, PlotItem};
use pika_core::plots::{PlotConfig, PlotDataConfig};
use std::collections::HashMap;

/// Radar plot for showing multi-dimensional data
pub struct RadarPlot {
    category_column: String,
    value_columns: Vec<String>,
    fill_alpha: f32,
    show_grid: bool,
}

impl RadarPlot {
    /// Create a new radar plot
    pub fn new(category_column: String, value_columns: Vec<String>) -> Self {
        Self {
            category_column,
            value_columns,
            fill_alpha: 0.3,
            show_grid: true,
        }
    }

    /// Create from plot configuration
    pub fn from_config(config: &PlotConfig) -> Self {
        if let PlotDataConfig::RadarConfig {
            category_column,
            value_columns,
            fill_alpha,
            show_grid,
        } = &config.specific
        {
            Self {
                category_column: category_column.clone(),
                value_columns: value_columns.clone(),
                fill_alpha: *fill_alpha,
                show_grid: *show_grid,
            }
        } else {
            Self::new("category".to_string(), vec!["value".to_string()])
        }
    }

    /// Render the radar plot
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let plot = Plot::new("radar_plot")
            .height(400.0)
            .width(400.0)
            .show_axes([false, false])
            .show_grid([false, false])
            .data_aspect(1.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            if self.show_grid {
                self.render_radar_grid(plot_ui);
            }
            self.render_radar_data(plot_ui, data);
        });
    }

    fn render_radar_grid(&self, plot_ui: &mut egui_plot::PlotUi) {
        let num_axes = self.value_columns.len();
        if num_axes == 0 {
            return;
        }

        let center = [0.0, 0.0];
        let max_radius = 1.0;
        
        // Draw concentric circles
        for i in 1..=5 {
            let radius = (i as f64 / 5.0) * max_radius;
            let circle_points = self.generate_circle_points(center, radius, 64);
            
            let line = Line::new(PlotPoints::new(circle_points))
                .color(egui::Color32::from_gray(200))
                .width(1.0);
            plot_ui.line(line);
        }

        // Draw axis lines
        for i in 0..num_axes {
            let angle = (i as f64 / num_axes as f64) * 2.0 * std::f64::consts::PI;
            let end_point = [
                center[0] + max_radius * angle.cos(),
                center[1] + max_radius * angle.sin(),
            ];
            
            let axis_line = Line::new(PlotPoints::new(vec![center, end_point]))
                .color(egui::Color32::from_gray(150))
                .width(1.0);
            plot_ui.line(axis_line);
        }

        // Draw axis labels (simplified - just note positions)
        for (i, _column) in self.value_columns.iter().enumerate() {
            let angle = (i as f64 / num_axes as f64) * 2.0 * std::f64::consts::PI;
            let label_radius = max_radius * 1.1;
            let _label_pos = [
                center[0] + label_radius * angle.cos(),
                center[1] + label_radius * angle.sin(),
            ];
            
            // Note: egui_plot doesn't have built-in text rendering, so we'd need to handle this differently
            // For now, we'll skip the labels or implement them as part of the main UI
        }
    }

    fn render_radar_data(&self, plot_ui: &mut egui_plot::PlotUi, data: &RecordBatch) {
        // Group data by category
        let grouped_data = self.group_data_by_category(data);
        
        // Render each category as a separate radar shape
        for (i, (category, values)) in grouped_data.iter().enumerate() {
            let radar_points = self.calculate_radar_points(values);
            
            if !radar_points.is_empty() {
                let color = self.get_category_color(i);
                
                // Draw filled polygon
                if self.fill_alpha > 0.0 {
                    let fill_color = egui::Color32::from_rgba_unmultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        (self.fill_alpha * 255.0) as u8,
                    );
                    
                    let polygon = Polygon::new(PlotPoints::new(radar_points.clone()))
                        .fill_color(fill_color)
                        .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT));
                    plot_ui.polygon(polygon);
                }
                
                // Draw outline
                let line = Line::new(PlotPoints::new(radar_points))
                    .color(color)
                    .width(2.0)
                    .name(category);
                plot_ui.line(line);
            }
        }
    }

    fn group_data_by_category(&self, data: &RecordBatch) -> HashMap<String, Vec<f64>> {
        let mut grouped_data = HashMap::new();
        
        // Get category column
        let category_array = data
            .column_by_name(&self.category_column)
            .and_then(|col| col.as_any().downcast_ref::<StringArray>());
        
        if let Some(categories) = category_array {
            // For each row, collect values for all value columns
            for row in 0..categories.len() {
                if !categories.is_null(row) {
                    let category = categories.value(row);
                    let mut row_values = Vec::new();
                    
                    // Collect values for all value columns for this row
                    for value_column in &self.value_columns {
                        if let Some(value) = self.get_value_at_row(data, value_column, row) {
                            row_values.push(value);
                        } else {
                            row_values.push(0.0); // Default value for missing data
                        }
                    }
                    
                    // Average values for categories that appear multiple times
                    let entry = grouped_data.entry(category.to_string()).or_insert_with(|| vec![0.0; self.value_columns.len()]);
                    for (i, &value) in row_values.iter().enumerate() {
                        if i < entry.len() {
                            entry[i] = (entry[i] + value) / 2.0; // Simple averaging
                        }
                    }
                }
            }
        }
        
        grouped_data
    }

    fn get_value_at_row(&self, data: &RecordBatch, column_name: &str, row: usize) -> Option<f64> {
        data.column_by_name(column_name)
            .and_then(|col| col.as_any().downcast_ref::<Float64Array>())
            .and_then(|array| {
                if !array.is_null(row) {
                    Some(array.value(row))
                } else {
                    None
                }
            })
    }

    fn calculate_radar_points(&self, values: &[f64]) -> Vec<[f64; 2]> {
        if values.is_empty() {
            return Vec::new();
        }
        
        let center = [0.0, 0.0];
        let max_radius = 1.0;
        let num_axes = values.len();
        
        // Normalize values to 0-1 range
        let max_value = values.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let min_value = values.iter().fold(f64::INFINITY, |acc, &x| acc.min(x));
        let range = max_value - min_value;
        
        let mut points = Vec::new();
        
        for (i, &value) in values.iter().enumerate() {
            let angle = (i as f64 / num_axes as f64) * 2.0 * std::f64::consts::PI;
            let normalized_value = if range > 0.0 {
                (value - min_value) / range
            } else {
                0.5 // Default to middle if all values are the same
            };
            
            let radius = normalized_value * max_radius;
            let point = [
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ];
            
            points.push(point);
        }
        
        // Close the polygon by adding the first point at the end
        if !points.is_empty() {
            points.push(points[0]);
        }
        
        points
    }

    fn generate_circle_points(&self, center: [f64; 2], radius: f64, num_points: usize) -> Vec<[f64; 2]> {
        let mut points = Vec::new();
        
        for i in 0..=num_points {
            let angle = (i as f64 / num_points as f64) * 2.0 * std::f64::consts::PI;
            let point = [
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ];
            points.push(point);
        }
        
        points
    }

    fn get_category_color(&self, index: usize) -> egui::Color32 {
        let colors = [
            egui::Color32::BLUE,
            egui::Color32::RED,
            egui::Color32::GREEN,
            egui::Color32::YELLOW,
            egui::Color32::LIGHT_BLUE,
            egui::Color32::LIGHT_RED,
            egui::Color32::LIGHT_GREEN,
            egui::Color32::GOLD,
        ];
        
        colors[index % colors.len()]
    }
}

impl Default for RadarPlot {
    fn default() -> Self {
        Self::new("category".to_string(), vec!["value".to_string()])
    }
} 