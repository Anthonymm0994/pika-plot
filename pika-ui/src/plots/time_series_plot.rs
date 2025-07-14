//! Time series plot implementation for temporal data visualization

use arrow::array::{Array, Float64Array, StringArray, TimestampMillisecondArray};
use arrow::record_batch::RecordBatch;
use egui::Ui;
use egui_plot::{Plot, PlotPoints, Line};
use pika_core::plots::{PlotConfig, PlotDataConfig, TimeAggregation};
use std::collections::HashMap;

/// Time series plot for showing temporal data
pub struct TimeSeriesPlot {
    time_column: String,
    value_columns: Vec<String>,
    aggregation: TimeAggregation,
    show_range_selector: bool,
}

impl TimeSeriesPlot {
    /// Create a new time series plot
    pub fn new(time_column: String, value_columns: Vec<String>) -> Self {
        Self {
            time_column,
            value_columns,
            aggregation: TimeAggregation::None,
            show_range_selector: false,
        }
    }

    /// Create from plot configuration
    pub fn from_config(config: &PlotConfig) -> Self {
        if let PlotDataConfig::TimeSeriesConfig {
            time_column,
            value_columns,
            aggregation,
            show_range_selector,
        } = &config.specific
        {
            Self {
                time_column: time_column.clone(),
                value_columns: value_columns.clone(),
                aggregation: *aggregation,
                show_range_selector: *show_range_selector,
            }
        } else {
            Self::new("timestamp".to_string(), vec!["value".to_string()])
        }
    }

    /// Render the time series plot
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        let plot = Plot::new("time_series_plot")
            .height(400.0)
            .show_axes([true, true])
            .show_grid([true, true])
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            self.render_time_series_lines(plot_ui, data);
        });

        // Show range selector if enabled
        if self.show_range_selector {
            self.render_range_selector(ui, data);
        }
    }

    fn render_time_series_lines(&self, plot_ui: &mut egui_plot::PlotUi, data: &RecordBatch) {
        // Get time column data
        let time_data = self.extract_time_data(data);
        if time_data.is_empty() {
            return;
        }

        // Render each value column as a separate line
        for (i, value_column) in self.value_columns.iter().enumerate() {
            if let Some(value_data) = self.extract_value_data(data, value_column) {
                let points = self.combine_time_value_data(&time_data, &value_data);
                
                if !points.is_empty() {
                    let color = self.get_line_color(i);
                    let line = Line::new(PlotPoints::new(points))
                        .color(color)
                        .width(2.0)
                        .name(value_column);
                    plot_ui.line(line);
                }
            }
        }
    }

    fn extract_time_data(&self, data: &RecordBatch) -> Vec<f64> {
        if let Some(column) = data.column_by_name(&self.time_column) {
            // Try timestamp column first
            if let Some(timestamp_array) = column.as_any().downcast_ref::<TimestampMillisecondArray>() {
                return (0..timestamp_array.len())
                    .filter(|&i| !timestamp_array.is_null(i))
                    .map(|i| timestamp_array.value(i) as f64 / 1000.0) // Convert to seconds
                    .collect();
            }
            
            // Try string column (parse as timestamp)
            if let Some(string_array) = column.as_any().downcast_ref::<StringArray>() {
                return (0..string_array.len())
                    .filter(|&i| !string_array.is_null(i))
                    .filter_map(|i| self.parse_timestamp_string(string_array.value(i)))
                    .collect();
            }
            
            // Try numeric column (assume unix timestamp)
            if let Some(float_array) = column.as_any().downcast_ref::<Float64Array>() {
                return (0..float_array.len())
                    .filter(|&i| !float_array.is_null(i))
                    .map(|i| float_array.value(i))
                    .collect();
            }
        }
        
        Vec::new()
    }

    fn extract_value_data(&self, data: &RecordBatch, column_name: &str) -> Option<Vec<f64>> {
        data.column_by_name(column_name)
            .and_then(|col| col.as_any().downcast_ref::<Float64Array>())
            .map(|array| {
                (0..array.len())
                    .filter(|&i| !array.is_null(i))
                    .map(|i| array.value(i))
                    .collect()
            })
    }

    fn combine_time_value_data(&self, time_data: &[f64], value_data: &[f64]) -> Vec<[f64; 2]> {
        let min_len = time_data.len().min(value_data.len());
        let mut points = Vec::with_capacity(min_len);
        
        for i in 0..min_len {
            points.push([time_data[i], value_data[i]]);
        }
        
        // Sort by time
        points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        
        // Apply aggregation if needed
        match self.aggregation {
            TimeAggregation::None => points,
            _ => self.aggregate_time_series(points),
        }
    }

    fn aggregate_time_series(&self, points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
        if points.is_empty() {
            return points;
        }
        
        let interval = self.get_aggregation_interval();
        let mut aggregated = HashMap::new();
        
        for point in points {
            let bucket = (point[0] / interval).floor() * interval;
            aggregated.entry(bucket as i64).or_insert(Vec::new()).push(point[1]);
        }
        
        let mut result: Vec<[f64; 2]> = aggregated
            .into_iter()
            .map(|(time_bucket, values)| {
                let avg_value = values.iter().sum::<f64>() / values.len() as f64;
                [time_bucket as f64, avg_value]
            })
            .collect();
        
        result.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        result
    }

    fn get_aggregation_interval(&self) -> f64 {
        match self.aggregation {
            TimeAggregation::None => 1.0,
            TimeAggregation::Second => 1.0,
            TimeAggregation::Minute => 60.0,
            TimeAggregation::Hour => 3600.0,
            TimeAggregation::Day => 86400.0,
            TimeAggregation::Week => 604800.0,
            TimeAggregation::Month => 2592000.0, // Approximate
            TimeAggregation::Year => 31536000.0, // Approximate
        }
    }

    fn parse_timestamp_string(&self, s: &str) -> Option<f64> {
        // Try common timestamp formats
        if let Ok(timestamp) = s.parse::<f64>() {
            return Some(timestamp);
        }
        
        // Try ISO 8601 format parsing (simplified)
        if s.contains('T') || s.contains(' ') {
            // This is a simplified parser - in a real implementation,
            // you'd use a proper datetime parsing library like chrono
            return None;
        }
        
        None
    }

    fn get_line_color(&self, index: usize) -> egui::Color32 {
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

    fn render_range_selector(&self, ui: &mut Ui, _data: &RecordBatch) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Time Range:");
            
            // Simple range selector - in a real implementation, 
            // this would be more sophisticated with date pickers
            if ui.button("Last Hour").clicked() {
                // Implement time range filtering
            }
            if ui.button("Last Day").clicked() {
                // Implement time range filtering
            }
            if ui.button("Last Week").clicked() {
                // Implement time range filtering
            }
            if ui.button("Last Month").clicked() {
                // Implement time range filtering
            }
            if ui.button("All Time").clicked() {
                // Implement time range filtering
            }
        });
    }
}

impl Default for TimeSeriesPlot {
    fn default() -> Self {
        Self::new("timestamp".to_string(), vec!["value".to_string()])
    }
} 