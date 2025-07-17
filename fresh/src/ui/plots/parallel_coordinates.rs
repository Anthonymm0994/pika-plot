use super::{Plot as PlotTrait, PlotData, PlotConfiguration, PlotPoint, extract_plot_points};
use egui::{Ui, Color32, RichText, Vec2, Pos2, Rect, Response, Stroke};
use egui_plot::{Plot, PlotPoints, PlotBounds, Line, PlotUi};
use datafusion::arrow::datatypes::DataType;
use crate::core::QueryResult;
use std::collections::HashMap;

pub struct ParallelCoordinatesPlot;

impl PlotTrait for ParallelCoordinatesPlot {
    fn name(&self) -> &'static str {
        "Parallel Coordinates"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        None // Parallel coordinates don't require a specific X column
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64, DataType::Int64, DataType::Utf8]
    }
    
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)> {
        vec![
            ("Color", vec![DataType::Float64, DataType::Int64, DataType::Utf8]),
            ("Size", vec![DataType::Float64, DataType::Int64]),
        ]
    }
    
    fn supports_color_mapping(&self) -> bool { true }
    fn supports_multiple_series(&self) -> bool { true }
    
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        if config.y_column.is_empty() {
            return Err("At least one Y column is required for parallel coordinates".to_string());
        }
        
        // For parallel coordinates, we need multiple columns
        let selected_columns = vec![&config.y_column];
        let mut all_columns = selected_columns.clone();
        
        if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() {
                all_columns.push(color_col);
            }
        }
        
        if let Some(size_col) = &config.size_column {
            if !size_col.is_empty() {
                all_columns.push(size_col);
            }
        }
        
        // Get column indices
        let mut column_indices = Vec::new();
        for col in &all_columns {
            let idx = query_result.columns.iter().position(|c| c == *col)
                .ok_or_else(|| format!("Column '{}' not found", col))?;
            column_indices.push(idx);
        }
        
        let mut points = Vec::new();
        let mut series_data = Vec::new();
        
        for (row_idx, row) in query_result.rows.iter().enumerate() {
            if row.len() >= column_indices.iter().max().unwrap_or(&0) + 1 {
                let mut point_data = Vec::new();
                let mut tooltip_data = HashMap::new();
                
                // Extract values for each column
                for (i, &col_idx) in column_indices.iter().enumerate() {
                    if row.len() > col_idx {
                        let value = row[col_idx].parse::<f64>().unwrap_or(0.0);
                        point_data.push(value);
                        tooltip_data.insert(query_result.columns[col_idx].clone(), row[col_idx].clone());
                    } else {
                        point_data.push(0.0);
                    }
                }
                
                // Create color mapping
                let color = if let Some(color_col) = &config.color_column {
                    if !color_col.is_empty() {
                        let color_idx = query_result.columns.iter().position(|c| c == color_col);
                        if let Some(idx) = color_idx {
                            if row.len() > idx {
                                let color_value = &row[idx];
                                // Create color based on value
                                if let Ok(num_val) = color_value.parse::<f64>() {
                                    let normalized = (num_val - 0.0).max(0.0).min(1.0);
                                    Color32::from_rgb(
                                        (normalized * 255.0) as u8,
                                        ((1.0 - normalized) * 255.0) as u8,
                                        128
                                    )
                                } else {
                                    // Categorical color
                                    let hash = color_value.chars().map(|c| c as u32).sum::<u32>();
                                    Color32::from_rgb(
                                        (hash % 256) as u8,
                                        ((hash >> 8) % 256) as u8,
                                        ((hash >> 16) % 256) as u8,
                                    )
                                }
                            } else {
                                Color32::BLUE
                            }
                        } else {
                            Color32::BLUE
                        }
                    } else {
                        Color32::BLUE
                    }
                } else {
                    Color32::BLUE
                };
                
                points.push(PlotPoint {
                    x: 0.0, // Will be set during rendering
                    y: 0.0, // Will be set during rendering
                    z: None,
                    label: None,
                    color: Some(color),
                    size: None,
                    series_id: Some(format!("row_{}", row_idx)),
                    tooltip_data,
                });
                
                series_data.push(point_data);
            }
        }
        
        Ok(PlotData {
            points,
            series: vec![],
            metadata: super::PlotMetadata {
                title: config.title.clone(),
                x_label: "Dimensions".to_string(),
                y_label: "Values".to_string(),
                show_legend: config.show_legend,
                show_grid: config.show_grid,
                color_scheme: config.color_scheme.clone(),
            },
            statistics: None,
        })
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No data available for parallel coordinates plot").color(Color32::GRAY));
            });
            return;
        }
        
        ui.group(|ui| {
            ui.label(RichText::new("Parallel Coordinates Plot").heading());
            ui.separator();
            
            // Show data summary
            ui.horizontal(|ui| {
                ui.label("Data Points:");
                ui.label(format!("{}", data.points.len()));
            });
            
            ui.horizontal(|ui| {
                ui.label("Dimensions:");
                ui.label(format!("{}", 3)); // Simplified for now
            });
            
            ui.separator();
            
            // Parallel coordinates visualization
            let plot_rect = ui.available_rect_before_wrap();
            let plot_size = Vec2::new(plot_rect.width(), plot_rect.height().min(400.0));
            
            ui.allocate_ui(plot_size, |ui| {
                render_parallel_coordinates(ui, data, config, plot_size);
            });
            
            // Controls
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Display Mode:");
                ui.radio_value(&mut 0, 0, "Lines");
                ui.radio_value(&mut 0, 1, "Points");
                ui.radio_value(&mut 0, 2, "Both");
            });
            
            ui.horizontal(|ui| {
                ui.label("Color By:");
                ui.radio_value(&mut 0, 0, "None");
                ui.radio_value(&mut 0, 1, "First Column");
                ui.radio_value(&mut 0, 2, "Last Column");
            });
        });
    }
}

/// Render parallel coordinates plot
fn render_parallel_coordinates(ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration, size: Vec2) {
    let margin = 50.0;
    let plot_width = size.x - 2.0 * margin;
    let plot_height = size.y - 2.0 * margin;
    
    // Define dimensions (simplified - in real implementation, this would come from config)
    let dimensions = vec!["X", "Y", "Z"];
    let num_dimensions = dimensions.len();
    
    if num_dimensions < 2 {
        ui.centered_and_justified(|ui| {
            ui.label("Need at least 2 dimensions for parallel coordinates");
        });
        return;
    }
    
    let dimension_width = plot_width / (num_dimensions - 1) as f32;
    
    // Draw dimension axes
    for (i, dimension) in dimensions.iter().enumerate() {
        let x = margin + i as f32 * dimension_width;
        
        // Draw axis line
        ui.painter().line_segment(
            [Pos2::new(x, margin), Pos2::new(x, margin + plot_height)],
            Stroke::new(2.0, Color32::GRAY),
        );
        
        // Draw axis label
        ui.painter().text(
            Pos2::new(x, margin + plot_height + 20.0),
            egui::Align2::CENTER_TOP,
            dimension,
            egui::FontId::proportional(14.0),
            Color32::BLACK,
        );
    }
    
    // Draw data lines
    for (point_idx, point) in data.points.iter().enumerate() {
        let color = point.color.unwrap_or(Color32::BLUE);
        
        // Generate sample data for demonstration
        let values = vec![
            (point_idx as f64 * 0.1) % 1.0,
            (point_idx as f64 * 0.3) % 1.0,
            (point_idx as f64 * 0.7) % 1.0,
        ];
        
        // Draw polyline connecting all dimensions
        let mut line_points = Vec::new();
        for (i, &value) in values.iter().enumerate() {
            let x = margin + i as f32 * dimension_width;
            let y = margin + plot_height - (value * plot_height as f64) as f32;
            line_points.push(Pos2::new(x, y));
        }
        
        if line_points.len() >= 2 {
            // Draw the polyline
            for i in 0..line_points.len() - 1 {
                ui.painter().line_segment(
                    [line_points[i], line_points[i + 1]],
                    Stroke::new(1.0, color),
                );
            }
            
            // Draw points at each dimension
            for &point_pos in &line_points {
                ui.painter().circle_filled(
                    point_pos,
                    3.0,
                    color,
                );
            }
        }
    }
    
    // Draw value labels on axes
    for (i, dimension) in dimensions.iter().enumerate() {
        let x = margin + i as f32 * dimension_width;
        
        // Draw min and max labels
        ui.painter().text(
            Pos2::new(x, margin + plot_height + 5.0),
            egui::Align2::CENTER_TOP,
            "1.0",
            egui::FontId::proportional(10.0),
            Color32::GRAY,
        );
        
        ui.painter().text(
            Pos2::new(x, margin - 5.0),
            egui::Align2::CENTER_BOTTOM,
            "0.0",
            egui::FontId::proportional(10.0),
            Color32::GRAY,
        );
    }
}
