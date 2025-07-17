use super::{Plot as PlotTrait, PlotData, PlotConfiguration, PlotPoint, extract_plot_points};
use egui::{Ui, Color32, RichText, Vec2, Pos2, Rect, Response, Stroke};
use egui_plot::{Plot, PlotPoints, PlotBounds, Line, PlotUi};
use datafusion::arrow::datatypes::DataType;
use crate::core::QueryResult;
use std::collections::HashMap;
use std::f64::consts::PI;

pub struct PolarPlot;

impl PlotTrait for PolarPlot {
    fn name(&self) -> &'static str {
        "Polar Plot"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64, DataType::Int64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64, DataType::Int64]
    }
    
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)> {
        vec![
            ("Color", vec![DataType::Float64, DataType::Int64, DataType::Utf8]),
            ("Size", vec![DataType::Float64, DataType::Int64]),
        ]
    }
    
    fn supports_color_mapping(&self) -> bool { true }
    
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        if config.x_column.is_empty() || config.y_column.is_empty() {
            return Err("X and Y columns are required for polar plots".to_string());
        }
        
        let x_idx = query_result.columns.iter().position(|c| c == &config.x_column)
            .ok_or("X column not found")?;
        let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
            .ok_or("Y column not found")?;
        
        // Find color column
        let color_idx = if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() {
                query_result.columns.iter().position(|c| c == color_col)
            } else {
                None
            }
        } else {
            None
        };
        
        let mut points = Vec::new();
        let mut polar_data = Vec::new();
        
        for (row_idx, row) in query_result.rows.iter().enumerate() {
            if row.len() > x_idx && row.len() > y_idx {
                // Parse angle (theta) value
                let angle_val = row[x_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse angle value '{}' as number", row[x_idx]))?;
                
                // Parse radius value
                let radius_val = row[y_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse radius value '{}' as number", row[y_idx]))?;
                
                // Create color mapping
                let color = if let Some(color_idx) = color_idx {
                    if row.len() > color_idx {
                        let color_value = &row[color_idx];
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
                };
                
                // Convert polar to Cartesian coordinates
                let x = radius_val * angle_val.cos();
                let y = radius_val * angle_val.sin();
                
                // Create tooltip data
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("Angle".to_string(), angle_val.to_string());
                tooltip_data.insert("Radius".to_string(), radius_val.to_string());
                tooltip_data.insert("X".to_string(), x.to_string());
                tooltip_data.insert("Y".to_string(), y.to_string());
                tooltip_data.insert(config.x_column.clone(), row[x_idx].clone());
                tooltip_data.insert(config.y_column.clone(), row[y_idx].clone());
                
                points.push(PlotPoint {
                    x,
                    y,
                    z: None,
                    label: None,
                    color: Some(color),
                    size: None,
                    series_id: None,
                    tooltip_data,
                });
                
                polar_data.push(PolarDataPoint {
                    angle: angle_val,
                    radius: radius_val,
                    color,
                });
            }
        }
        
        // Calculate polar statistics
        let statistics = calculate_polar_statistics(&polar_data);
        
        Ok(PlotData {
            points,
            series: vec![],
            metadata: super::PlotMetadata {
                title: config.title.clone(),
                x_label: config.x_column.clone(),
                y_label: config.y_column.clone(),
                show_legend: config.show_legend,
                show_grid: config.show_grid,
                color_scheme: config.color_scheme.clone(),
            },
            statistics: Some(statistics),
        })
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No data available for polar plot").color(Color32::GRAY));
            });
            return;
        }
        
        ui.group(|ui| {
            ui.label(RichText::new("Polar Plot").heading());
            ui.separator();
            
            // Show data summary
            ui.horizontal(|ui| {
                ui.label("Data Points:");
                ui.label(format!("{}", data.points.len()));
            });
            
            if let Some(first_point) = data.points.first() {
                ui.horizontal(|ui| {
                    ui.label("Radius Range:");
                    ui.label(format!("{:.2} to {:.2}", 
                        data.points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
                        data.points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max)
                    ));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Angle Range:");
                    ui.label(format!("{:.2} to {:.2}", 
                        data.points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
                        data.points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max)
                    ));
                });
            }
            
            // Show polar statistics
            if let Some(stats) = &data.statistics {
                ui.separator();
                ui.label(RichText::new("Polar Statistics").strong());
                ui.horizontal(|ui| {
                    ui.label("Mean Radius:");
                    ui.label(format!("{:.3}", stats.mean_y));
                });
                ui.horizontal(|ui| {
                    ui.label("Max Radius:");
                    ui.label(format!("{:.3}", stats.std_y));
                });
                if let Some(corr) = stats.correlation {
                    ui.horizontal(|ui| {
                        ui.label("Distribution:");
                        ui.label(if corr.abs() > 0.5 { "Concentrated" } 
                               else { "Scattered" });
                    });
                }
            }
            
            ui.separator();
            
            // Polar plot visualization
            let plot_rect = ui.available_rect_before_wrap();
            let plot_size = Vec2::new(plot_rect.width(), plot_rect.height().min(400.0));
            
            ui.allocate_ui(plot_size, |ui| {
                render_polar_plot(ui, data, config, plot_size);
            });
            
            // Controls
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Display Mode:");
                ui.radio_value(&mut 0, 0, "Points");
                ui.radio_value(&mut 0, 1, "Lines");
                ui.radio_value(&mut 0, 2, "Both");
            });
            
            ui.horizontal(|ui| {
                ui.label("Angle Units:");
                ui.radio_value(&mut 0, 0, "Radians");
                ui.radio_value(&mut 0, 1, "Degrees");
            });
        });
    }
}

/// Polar data point structure
#[derive(Debug, Clone)]
struct PolarDataPoint {
    angle: f64,
    radius: f64,
    color: Color32,
}

/// Calculate polar statistics
fn calculate_polar_statistics(polar_data: &[PolarDataPoint]) -> super::DataStatistics {
    if polar_data.is_empty() {
        return super::DataStatistics {
            mean_x: 0.0,
            mean_y: 0.0,
            std_x: 0.0,
            std_y: 0.0,
            correlation: None,
            count: 0,
        };
    }
    
    let angles: Vec<f64> = polar_data.iter().map(|d| d.angle).collect();
    let radii: Vec<f64> = polar_data.iter().map(|d| d.radius).collect();
    
    let mean_angle = angles.iter().sum::<f64>() / angles.len() as f64;
    let mean_radius = radii.iter().sum::<f64>() / radii.len() as f64;
    
    let variance_angle = angles.iter()
        .map(|a| (a - mean_angle).powi(2))
        .sum::<f64>() / angles.len() as f64;
    let std_angle = variance_angle.sqrt();
    
    let variance_radius = radii.iter()
        .map(|r| (r - mean_radius).powi(2))
        .sum::<f64>() / radii.len() as f64;
    let std_radius = variance_radius.sqrt();
    
    // Calculate correlation between angle and radius
    let correlation = if std_angle > 0.0 && std_radius > 0.0 {
        let covariance = angles.iter().zip(radii.iter())
            .map(|(a, r)| (a - mean_angle) * (r - mean_radius))
            .sum::<f64>() / angles.len() as f64;
        Some(covariance / (std_angle * std_radius))
    } else {
        None
    };
    
    super::DataStatistics {
        mean_x: mean_angle,
        mean_y: mean_radius,
        std_x: std_angle,
        std_y: std_radius,
        correlation,
        count: angles.len(),
    }
}

/// Render polar plot
fn render_polar_plot(ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration, size: Vec2) {
    if data.points.is_empty() {
        return;
    }
    
    let center = Pos2::new(size.x / 2.0, size.y / 2.0);
    let max_radius = (size.x.min(size.y) / 2.0 - 50.0).max(50.0);
    
    // Draw polar grid
    draw_polar_grid(ui, center, max_radius);
    
    // Draw polar axis labels
    draw_polar_labels(ui, center, max_radius);
    
    // Draw data points
    for point in &data.points {
        let color = point.color.unwrap_or(Color32::BLUE);
        
        // Convert Cartesian to polar for display
        let x = point.x;
        let y = point.y;
        let radius = (x * x + y * y).sqrt();
        let angle = y.atan2(x);
        
        // Scale radius to fit display
        let display_radius = (radius / 10.0).min(1.0) * max_radius as f64;
        let display_x = center.x + (display_radius * angle.cos()) as f32;
        let display_y = center.y + (display_radius * angle.sin()) as f32;
        
        // Draw point
        ui.painter().circle_filled(
            Pos2::new(display_x, display_y),
            4.0,
            color,
        );
        
        // Draw line from center to point
        ui.painter().line_segment(
            [center, Pos2::new(display_x, display_y)],
            Stroke::new(1.0, color.linear_multiply(0.5)),
        );
    }
}

/// Draw polar grid (concentric circles and radial lines)
fn draw_polar_grid(ui: &mut Ui, center: Pos2, max_radius: f32) {
    // Draw concentric circles
    for i in 1..=5 {
        let circle_radius = max_radius * i as f32 / 5.0;
        ui.painter().circle_stroke(
            center,
            circle_radius,
            Stroke::new(1.0, Color32::from_gray(200)),
        );
    }
    
    // Draw radial lines (every 30 degrees)
    for i in 0..12 {
        let angle = 2.0 * PI * i as f64 / 12.0;
        let end_x = center.x + (max_radius * angle.cos() as f32);
        let end_y = center.y + (max_radius * angle.sin() as f32);
        
        ui.painter().line_segment(
            [center, Pos2::new(end_x, end_y)],
            Stroke::new(1.0, Color32::from_gray(200)),
        );
    }
}

/// Draw polar axis labels
fn draw_polar_labels(ui: &mut Ui, center: Pos2, max_radius: f32) {
    // Draw radius labels
    for i in 1..=5 {
        let radius = max_radius * i as f32 / 5.0;
        let label_pos = Pos2::new(center.x + radius + 10.0, center.y);
        
        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            &format!("{:.1}", i as f32 / 5.0),
            egui::FontId::proportional(10.0),
            Color32::GRAY,
        );
    }
    
    // Draw angle labels
    for i in 0..12 {
        let angle = 2.0 * PI * i as f64 / 12.0;
        let label_radius = max_radius + 20.0;
        let label_x = center.x + (label_radius * angle.cos() as f32);
        let label_y = center.y + (label_radius * angle.sin() as f32);
        
        ui.painter().text(
            Pos2::new(label_x, label_y),
            egui::Align2::CENTER_CENTER,
            &format!("{}°", i * 30),
            egui::FontId::proportional(10.0),
            Color32::GRAY,
        );
    }
}
