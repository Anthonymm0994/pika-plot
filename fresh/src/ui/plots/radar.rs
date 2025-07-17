use super::{Plot as PlotTrait, PlotData, PlotConfiguration, PlotPoint, extract_plot_points};
use egui::{Ui, Color32, RichText, Vec2, Pos2, Rect, Response, Stroke};
use egui_plot::{Plot, PlotPoints, PlotBounds, Line, PlotUi};
use datafusion::arrow::datatypes::DataType;
use crate::core::QueryResult;
use std::collections::HashMap;
use std::f64::consts::PI;

pub struct RadarPlot;

impl PlotTrait for RadarPlot {
    fn name(&self) -> &'static str {
        "Radar Chart"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        None // Radar charts don't require a specific X column
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
            return Err("At least one Y column is required for radar charts".to_string());
        }
        
        // For radar charts, we need multiple dimensions
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
                ui.label(RichText::new("No data available for radar chart").color(Color32::GRAY));
            });
            return;
        }
        
        ui.group(|ui| {
            ui.label(RichText::new("Radar Chart").heading());
            ui.separator();
            
            // Show data summary
            ui.horizontal(|ui| {
                ui.label("Data Points:");
                ui.label(format!("{}", data.points.len()));
            });
            
            ui.horizontal(|ui| {
                ui.label("Dimensions:");
                ui.label(format!("{}", 5)); // Simplified for now
            });
            
            ui.separator();
            
            // Radar chart visualization
            let plot_rect = ui.available_rect_before_wrap();
            let plot_size = Vec2::new(plot_rect.width(), plot_rect.height().min(400.0));
            
            ui.allocate_ui(plot_size, |ui| {
                render_radar_chart(ui, data, config, plot_size);
            });
            
            // Controls
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Display Mode:");
                ui.radio_value(&mut 0, 0, "Lines");
                ui.radio_value(&mut 0, 1, "Area");
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

/// Render radar chart
fn render_radar_chart(ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration, size: Vec2) {
    let center = Pos2::new(size.x / 2.0, size.y / 2.0);
    let radius = (size.x.min(size.y) / 2.0 - 50.0).max(50.0);
    
    // Define dimensions (simplified - in real implementation, this would come from config)
    let dimensions = vec!["Speed", "Power", "Range", "Accuracy", "Durability"];
    let num_dimensions = dimensions.len();
    
    if num_dimensions < 3 {
        ui.centered_and_justified(|ui| {
            ui.label("Need at least 3 dimensions for radar chart");
        });
        return;
    }
    
    // Draw radar grid
    draw_radar_grid(ui, center, radius, num_dimensions);
    
    // Draw dimension labels
    draw_radar_labels(ui, center, radius, &dimensions);
    
    // Draw data polygons
    for (point_idx, point) in data.points.iter().enumerate() {
        let color = point.color.unwrap_or(Color32::BLUE);
        
        // Generate sample data for demonstration
        let values = vec![
            (point_idx as f64 * 0.1 + 0.3) % 1.0,
            (point_idx as f64 * 0.2 + 0.5) % 1.0,
            (point_idx as f64 * 0.3 + 0.7) % 1.0,
            (point_idx as f64 * 0.4 + 0.4) % 1.0,
            (point_idx as f64 * 0.5 + 0.6) % 1.0,
        ];
        
        // Draw radar polygon
        draw_radar_polygon(ui, center, radius, &values, color);
    }
}

/// Draw radar grid (concentric circles and radial lines)
fn draw_radar_grid(ui: &mut Ui, center: Pos2, radius: f32, num_dimensions: usize) {
    // Draw concentric circles
    for i in 1..=5 {
        let circle_radius = radius * i as f32 / 5.0;
        ui.painter().circle_stroke(
            center,
            circle_radius,
            Stroke::new(1.0, Color32::from_gray(200)),
        );
    }
    
    // Draw radial lines
    for i in 0..num_dimensions {
        let angle = 2.0 * PI * i as f64 / num_dimensions as f64 - PI / 2.0;
        let end_x = center.x + (radius * angle.cos() as f32);
        let end_y = center.y + (radius * angle.sin() as f32);
        
        ui.painter().line_segment(
            [center, Pos2::new(end_x, end_y)],
            Stroke::new(1.0, Color32::from_gray(200)),
        );
    }
}

/// Draw dimension labels around the radar
fn draw_radar_labels(ui: &mut Ui, center: Pos2, radius: f32, dimensions: &[&str]) {
    let label_radius = radius + 30.0;
    
    for (i, dimension) in dimensions.iter().enumerate() {
        let angle = 2.0 * PI * i as f64 / dimensions.len() as f64 - PI / 2.0;
        let x = center.x + (label_radius * angle.cos() as f32);
        let y = center.y + (label_radius * angle.sin() as f32);
        
        ui.painter().text(
            Pos2::new(x, y),
            egui::Align2::CENTER_CENTER,
            dimension,
            egui::FontId::proportional(12.0),
            Color32::BLACK,
        );
    }
}

/// Draw radar polygon for a data series
fn draw_radar_polygon(ui: &mut Ui, center: Pos2, radius: f32, values: &[f64], color: Color32) {
    if values.is_empty() {
        return;
    }
    
    let num_dimensions = values.len();
    let mut polygon_points = Vec::new();
    
    // Calculate polygon vertices
    for (i, &value) in values.iter().enumerate() {
        let angle = 2.0 * PI * i as f64 / num_dimensions as f64 - PI / 2.0;
        let point_radius = radius * value as f32;
        let x = center.x + (point_radius * angle.cos() as f32);
        let y = center.y + (point_radius * angle.sin() as f32);
        polygon_points.push(Pos2::new(x, y));
    }
    
    // Draw filled polygon
    if polygon_points.len() >= 3 {
        ui.painter().add(egui::Shape::convex_polygon(
            polygon_points.clone(),
            color.linear_multiply(0.3),
            Stroke::new(2.0, color),
        ));
    }
    
    // Draw polygon outline
    if polygon_points.len() >= 2 {
        for i in 0..polygon_points.len() {
            let start = polygon_points[i];
            let end = polygon_points[(i + 1) % polygon_points.len()];
            
            ui.painter().line_segment(
                [start, end],
                Stroke::new(2.0, color),
            );
        }
    }
    
    // Draw vertices
    for &point in &polygon_points {
        ui.painter().circle_filled(
            point,
            4.0,
            color,
        );
    }
}
