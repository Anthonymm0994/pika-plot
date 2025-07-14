use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Polygon, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, ColorScale};
use pika_engine::plot::{extract_numeric_values, extract_string_values};
use crate::theme::{PlotTheme, get_theme_mode};
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
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        // Extract data
        let x_array = data.column_by_name(&self.x_column);
        let y_array = data.column_by_name(&self.y_column);
        let value_array = data.column_by_name(&self.value_column);
        
        if let (Some(x_arr), Some(y_arr), Some(val_arr)) = (x_array, y_array, value_array) {
            // Try to extract as strings first (for categorical data)
            let x_result = extract_string_values(x_arr);
            let y_result = extract_string_values(y_arr);
            let value_result = extract_numeric_values(val_arr);
            
            match (x_result, y_result, &value_result) {
                (Ok(x_categories), Ok(y_categories), Ok(values)) => {
                    self.render_categorical_heatmap(ui, &x_categories, &y_categories, values, &plot_theme);
                }
                _ => {
                    // Try numeric data
                    let x_numeric = extract_numeric_values(x_arr);
                    let y_numeric = extract_numeric_values(y_arr);
                    
                    match (x_numeric, y_numeric, value_result) {
                        (Ok(x_vals), Ok(y_vals), Ok(values)) => {
                            self.render_numeric_heatmap(ui, &x_vals, &y_vals, &values, &plot_theme);
                        }
                        _ => {
                            ui.colored_label(Color32::RED, "Error: Could not extract heatmap data");
                        }
                    }
                }
            }
        } else {
            ui.colored_label(Color32::RED, "Error: Required columns not found for heatmap");
        }
    }
    
    fn render_categorical_heatmap(&self, ui: &mut Ui, x_categories: &[String], y_categories: &[String], values: &[f64], plot_theme: &PlotTheme) {
        // Create plot
        let mut plot = Plot::new("heatmap_plot")
            .legend(Legend::default())
            .show_grid(true)
            .show_axes([true, true]);
        
        // Apply theme colors to plot
        if get_theme_mode(ui.ctx()) == crate::theme::ThemeMode::Dark {
            plot = plot.show_background(false);
        }
        
        plot.show(ui, |plot_ui| {
            // Group data by categories
            let mut data_map: HashMap<(String, String), f64> = HashMap::new();
            
            for i in 0..x_categories.len().min(y_categories.len()).min(values.len()) {
                let key = (x_categories[i].clone(), y_categories[i].clone());
                data_map.insert(key, values[i]);
            }
            
            // Get unique categories
            let mut unique_x: Vec<String> = x_categories.iter().cloned().collect();
            unique_x.sort();
            unique_x.dedup();
            
            let mut unique_y: Vec<String> = y_categories.iter().cloned().collect();
            unique_y.sort();
            unique_y.dedup();
            
            // Find min/max values for color scaling
            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            // Render heatmap cells
            for (x_idx, x_cat) in unique_x.iter().enumerate() {
                for (y_idx, y_cat) in unique_y.iter().enumerate() {
                    if let Some(&value) = data_map.get(&(x_cat.clone(), y_cat.clone())) {
                        let normalized_value = if max_val > min_val {
                            (value - min_val) / (max_val - min_val)
                        } else {
                            0.5
                        };
                        
                        let color = self.get_color_for_value(normalized_value, plot_theme);
                        
                        let cell_points = vec![
                            [x_idx as f64, y_idx as f64],
                            [x_idx as f64 + 1.0, y_idx as f64],
                            [x_idx as f64 + 1.0, y_idx as f64 + 1.0],
                            [x_idx as f64, y_idx as f64 + 1.0],
                        ];
                        
                        let cell = Polygon::new(PlotPoints::new(cell_points))
                            .fill_color(color)
                            .stroke(egui::Stroke::new(0.5, plot_theme.grid_color));
                        
                        plot_ui.polygon(cell);
                    }
                }
            }
        });
    }
    
    fn render_numeric_heatmap(&self, ui: &mut Ui, x_values: &[f64], y_values: &[f64], values: &[f64], plot_theme: &PlotTheme) {
        // Create plot
        let mut plot = Plot::new("heatmap_plot")
            .legend(Legend::default())
            .show_grid(true)
            .show_axes([true, true]);
        
        // Apply theme colors to plot
        if get_theme_mode(ui.ctx()) == crate::theme::ThemeMode::Dark {
            plot = plot.show_background(false);
        }
        
        plot.show(ui, |plot_ui| {
            // Find min/max values for color scaling
            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            // Create grid-based heatmap
            let grid_size = 20; // Configurable grid resolution
            let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let x_max = x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let y_min = y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let y_max = y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            let x_step = (x_max - x_min) / grid_size as f64;
            let y_step = (y_max - y_min) / grid_size as f64;
            
            for i in 0..grid_size {
                for j in 0..grid_size {
                    let x_start = x_min + i as f64 * x_step;
                    let x_end = x_start + x_step;
                    let y_start = y_min + j as f64 * y_step;
                    let y_end = y_start + y_step;
                    
                    // Find points in this grid cell
                    let mut cell_values = Vec::new();
                    for k in 0..x_values.len().min(y_values.len()).min(values.len()) {
                        if x_values[k] >= x_start && x_values[k] < x_end &&
                           y_values[k] >= y_start && y_values[k] < y_end {
                            cell_values.push(values[k]);
                        }
                    }
                    
                    if !cell_values.is_empty() {
                        // Average values in this cell
                        let avg_value = cell_values.iter().sum::<f64>() / cell_values.len() as f64;
                        let normalized_value = if max_val > min_val {
                            (avg_value - min_val) / (max_val - min_val)
                        } else {
                            0.5
                        };
                        
                        let color = self.get_color_for_value(normalized_value, plot_theme);
                        
                        let cell_points = vec![
                            [x_start, y_start],
                            [x_end, y_start],
                            [x_end, y_end],
                            [x_start, y_end],
                        ];
                        
                        let cell = Polygon::new(PlotPoints::new(cell_points))
                            .fill_color(color)
                            .stroke(egui::Stroke::new(0.5, plot_theme.grid_color));
                        
                        plot_ui.polygon(cell);
                    }
                }
            }
        });
    }
    
    fn get_color_for_value(&self, normalized_value: f64, plot_theme: &PlotTheme) -> Color32 {
        let value = normalized_value.clamp(0.0, 1.0);
        
        match self.color_scale {
            ColorScale::Viridis => self.viridis_color(value),
            ColorScale::Plasma => self.plasma_color(value),
            ColorScale::Blues => self.blues_color(value, plot_theme),
            ColorScale::Reds => self.reds_color(value),
            ColorScale::Greens => self.greens_color(value),
            _ => {
                // Default gradient from blue to red
                let r = (255.0 * value) as u8;
                let b = (255.0 * (1.0 - value)) as u8;
                Color32::from_rgb(r, 0, b)
            }
        }
    }
    
    fn viridis_color(&self, t: f64) -> Color32 {
        // Simplified viridis colormap
        let r = (255.0 * (0.267 + 0.005 * t + 0.322 * t * t)) as u8;
        let g = (255.0 * (0.004 + 0.396 * t + 0.154 * t * t)) as u8;
        let b = (255.0 * (0.329 + 0.456 * t - 0.203 * t * t)) as u8;
        Color32::from_rgb(r, g, b)
    }
    
    fn plasma_color(&self, t: f64) -> Color32 {
        // Simplified plasma colormap
        let r = (255.0 * (0.050 + 0.839 * t)) as u8;
        let g = (255.0 * (0.023 + 0.382 * t + 0.234 * t * t)) as u8;
        let b = (255.0 * (0.847 - 0.805 * t + 0.239 * t * t)) as u8;
        Color32::from_rgb(r, g, b)
    }
    
    fn blues_color(&self, t: f64, plot_theme: &PlotTheme) -> Color32 {
        let base_blue = plot_theme.categorical_color(0);
        let intensity = (255.0 * (0.1 + 0.9 * t)) as u8;
        Color32::from_rgb(
            (base_blue.r() as f64 * t) as u8,
            (base_blue.g() as f64 * t) as u8,
            intensity
        )
    }
    
    fn reds_color(&self, t: f64) -> Color32 {
        let intensity = (255.0 * (0.1 + 0.9 * t)) as u8;
        Color32::from_rgb(intensity, 0, 0)
    }
    
    fn greens_color(&self, t: f64) -> Color32 {
        let intensity = (255.0 * (0.1 + 0.9 * t)) as u8;
        Color32::from_rgb(0, intensity, 0)
    }
} 