use super::{Plot as PlotTrait, PlotData, PlotConfiguration, PlotInteraction};
use egui::{Ui, RichText};
use egui_plot::{Plot, Legend, Line, PlotPoints};
use datafusion::arrow::datatypes::DataType;
use std::collections::HashMap;

pub struct StreamPlot;

impl StreamPlot {
    /// Process data for stream graph visualization
    fn process_data(&self, data: &PlotData) -> Vec<(String, Vec<(f64, f64)>)> {
        // Group data points by series
        let mut series_data: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
        
        for point in &data.points {
            let series_id = point.series_id.clone().unwrap_or_else(|| "default".to_string());
            series_data.entry(series_id.clone())
                .or_insert_with(Vec::new)
                .push((point.x, point.y));
        }
        
        // Sort each series by x value
        for points in series_data.values_mut() {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
        
        // Convert to vector of (series_name, points)
        let mut result: Vec<(String, Vec<(f64, f64)>)> = series_data.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        
        result
    }
}

impl PlotTrait for StreamPlot {
    fn name(&self) -> &'static str {
        "Stream Graph"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![
            DataType::Float64, DataType::Float32,
            DataType::Int64, DataType::Int32, DataType::Int16, DataType::Int8,
            DataType::UInt64, DataType::UInt32, DataType::UInt16, DataType::UInt8,
            // Also support temporal types for time series
            DataType::Date32, DataType::Date64,
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Second, None),
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Millisecond, None),
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Microsecond, None),
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Nanosecond, None),
        ])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![
            DataType::Float64, DataType::Float32,
            DataType::Int64, DataType::Int32, DataType::Int16, DataType::Int8,
            DataType::UInt64, DataType::UInt32, DataType::UInt16, DataType::UInt8,
        ]
    }
    
    fn supports_multiple_series(&self) -> bool {
        true
    }
    
    fn supports_color_mapping(&self) -> bool {
        true
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
                ui.label(RichText::new("Configure X and Y columns for stream graph visualization").weak());
            });
            return;
        }
        
        // Process data for stream graph
        let series_data = self.process_data(data);
        
        if series_data.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No valid series data for stream graph");
            });
            return;
        }
        
        // Create plot
        let plot = Plot::new("stream_plot")
            .x_axis_label(&data.metadata.x_label)
            .y_axis_label(&data.metadata.y_label)
            .show_grid(data.metadata.show_grid)
            .legend(Legend::default());
        
        plot.show(ui, |plot_ui| {
            // Render each series as a line
            for (i, (series_name, points)) in series_data.iter().enumerate() {
                // Find the corresponding series in data.series if it exists
                let color = data.series.iter()
                    .find(|s| &s.id == series_name)
                    .map(|s| s.color)
                    .unwrap_or_else(|| {
                        // Use default color scheme if series not found
                        let colors = super::get_categorical_colors(&data.metadata.color_scheme);
                        colors[i % colors.len()]
                    });
                
                // Convert points to PlotPoints
                let plot_points: Vec<[f64; 2]> = points.iter()
                    .map(|(x, y)| [*x, *y])
                    .collect();
                
                // Draw the line
                let line = Line::new(PlotPoints::from(plot_points))
                    .color(color)
                    .name(series_name);
                
                plot_ui.line(line);
            }
        });
        
        // Show statistics
        ui.horizontal(|ui| {
            ui.label(format!("Series: {}", series_data.len()));
            ui.label(format!("Total points: {}", data.points.len()));
        });
    }
    
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) {
        if !data.series.is_empty() {
            ui.group(|ui| {
                ui.label(RichText::new("Series:").strong());
                ui.separator();
                
                for series in &data.series {
                    if series.visible {
                        ui.horizontal(|ui| {
                            ui.colored_label(series.color, "—");
                            ui.label(&series.name);
                        });
                    }
                }
            });
        }
    }
    
    fn handle_interaction(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) -> Option<PlotInteraction> {
        if !data.series.is_empty() {
            ui.vertical(|ui| {
                for series in &data.series {
                    let mut is_visible = series.visible;
                    if ui.checkbox(&mut is_visible, &series.name).changed() {
                        return Some(PlotInteraction::SeriesToggled(series.id.clone()));
                    }
                }
                
                None
            }).inner
        } else {
            None
        }
    }
}
