use egui::{Ui, Color32, Rect, Pos2, Vec2};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, ColorScale};
use pika_engine::plot::{extract_numeric_values, extract_string_values};
use std::collections::HashMap;

pub struct HeatmapPlot {
    x_column: String,
    y_column: String,
    value_column: String,
    color_scale: ColorScale,
    interpolation: bool,
}

impl HeatmapPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::HeatmapConfig {
                x_column,
                y_column,
                value_column,
                color_scale,
                interpolation,
            } => Self {
                x_column: x_column.clone(),
                y_column: y_column.clone(),
                value_column: value_column.clone(),
                color_scale: *color_scale,
                interpolation: *interpolation,
            },
            _ => panic!("Invalid config for heatmap plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Extract data
        let x_array = match data.column_by_name(&self.x_column) {
            Some(arr) => arr,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.x_column));
                return;
            }
        };
        
        let y_array = match data.column_by_name(&self.y_column) {
            Some(arr) => arr,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.y_column));
                return;
            }
        };
        
        let value_array = match data.column_by_name(&self.value_column) {
            Some(arr) => arr,
            None => {
                ui.colored_label(Color32::RED, format!("Column '{}' not found", self.value_column));
                return;
            }
        };
        
        // Try to extract as categories first
        let x_categories = extract_string_values(x_array).ok();
        let y_categories = extract_string_values(y_array).ok();
        let values = match extract_numeric_values(value_array) {
            Ok(v) => v,
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error extracting values: {}", e));
                return;
            }
        };
        
        if let (Some(x_cats), Some(y_cats)) = (x_categories, y_categories) {
            // Create a matrix from categorical data
            let mut matrix: HashMap<(String, String), f64> = HashMap::new();
            let mut x_unique: Vec<String> = Vec::new();
            let mut y_unique: Vec<String> = Vec::new();
            
            for i in 0..x_cats.len().min(y_cats.len()).min(values.len()) {
                matrix.insert((x_cats[i].clone(), y_cats[i].clone()), values[i]);
                
                if !x_unique.contains(&x_cats[i]) {
                    x_unique.push(x_cats[i].clone());
                }
                if !y_unique.contains(&y_cats[i]) {
                    y_unique.push(y_cats[i].clone());
                }
            }
            
            // Sort categories
            x_unique.sort();
            y_unique.sort();
            
            // Find min/max values
            let min_val = values.iter().filter(|v| !v.is_nan()).fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().filter(|v| !v.is_nan()).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            // Render heatmap
            let available_rect = ui.available_rect_before_wrap();
            let cell_width = available_rect.width() / x_unique.len() as f32;
            let cell_height = available_rect.height() / y_unique.len() as f32;
            
            for (i, y_cat) in y_unique.iter().enumerate() {
                for (j, x_cat) in x_unique.iter().enumerate() {
                    let value = matrix.get(&(x_cat.clone(), y_cat.clone())).copied().unwrap_or(f64::NAN);
                    
                    if !value.is_nan() {
                        let normalized = (value - min_val) / (max_val - min_val);
                        let color = get_color_from_scale(self.color_scale, normalized);
                        
                        let rect = Rect::from_min_size(
                            Pos2::new(
                                available_rect.left() + j as f32 * cell_width,
                                available_rect.top() + i as f32 * cell_height,
                            ),
                            Vec2::new(cell_width, cell_height),
                        );
                        
                        ui.painter().rect_filled(rect, 0.0, color);
                    }
                }
            }
            
            // Add labels
            ui.advance_cursor_after_rect(available_rect);
            ui.label(format!("Heatmap: {} x {} = {}", self.x_column, self.y_column, self.value_column));
        } else {
            ui.label("Heatmap requires categorical X and Y columns");
        }
    }
}

fn get_color_from_scale(scale: ColorScale, value: f64) -> Color32 {
    let value = value.clamp(0.0, 1.0);
    
    match scale {
        ColorScale::Viridis => {
            // Simplified viridis colormap
            let r = (value * 255.0) as u8;
            let g = ((1.0 - (value - 0.5).abs() * 2.0) * 255.0) as u8;
            let b = ((1.0 - value) * 255.0) as u8;
            Color32::from_rgb(r, g, b)
        }
        ColorScale::Blues => {
            let intensity = (value * 255.0) as u8;
            Color32::from_rgb(255 - intensity, 255 - intensity, 255)
        }
        ColorScale::Reds => {
            let intensity = (value * 255.0) as u8;
            Color32::from_rgb(255, 255 - intensity, 255 - intensity)
        }
        ColorScale::Greens => {
            let intensity = (value * 255.0) as u8;
            Color32::from_rgb(255 - intensity, 255, 255 - intensity)
        }
        _ => {
            // Default grayscale
            let intensity = (value * 255.0) as u8;
            Color32::from_gray(intensity)
        }
    }
} 