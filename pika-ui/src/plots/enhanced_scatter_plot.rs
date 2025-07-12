//! Enhanced scatter plot implementation extracted from frog-viz
//! Provides advanced scatter plotting with categorical coloring, interactive tooltips,
//! configurable markers, legend support, and performance optimization

use egui::{Color32, Ui, Response, Vec2};
use egui_plot::{Plot, PlotPoints, PlotPoint, Points, Legend, Corner, MarkerShape};
use arrow::record_batch::RecordBatch;
use arrow::array::{Array, StringArray, Float64Array, Int64Array};
use pika_core::types::TableInfo;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedScatterConfig {
    pub x_column: String,
    pub y_column: String,
    pub color_column: Option<String>,
    pub size_column: Option<String>,
    pub marker_shape: String, // Store as string instead of MarkerShape
    pub marker_size: f32,
    pub show_legend: bool,
    pub show_tooltips: bool,
    pub alpha: f32,
    pub point_limit: usize,
    pub color_palette: Vec<String>, // Store colors as hex strings
}

impl Default for EnhancedScatterConfig {
    fn default() -> Self {
        Self {
            x_column: String::new(),
            y_column: String::new(),
            color_column: None,
            size_column: None,
            marker_shape: "Circle".to_string(),
            marker_size: 3.0,
            show_legend: true,
            show_tooltips: true,
            alpha: 0.8,
            point_limit: 10000,
            color_palette: vec![
                "#1f77b4".to_string(), "#ff7f0e".to_string(), "#2ca02c".to_string(),
                "#d62728".to_string(), "#9467bd".to_string(), "#8c564b".to_string(),
                "#e377c2".to_string(), "#7f7f7f".to_string(), "#bcbd22".to_string(),
                "#17becf".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    pub category: Option<String>,
    pub size: f32,
    pub color: Color32,
    pub tooltip: String,
}

pub struct EnhancedScatterPlot {
    config: EnhancedScatterConfig,
    points: Vec<ScatterPoint>,
    categories: Vec<String>,
    color_map: HashMap<String, Color32>,
    bounds: Option<(f64, f64, f64, f64)>, // (min_x, max_x, min_y, max_y)
}

impl EnhancedScatterPlot {
    pub fn new(config: EnhancedScatterConfig) -> Self {
        Self {
            config,
            points: Vec::new(),
            categories: Vec::new(),
            color_map: HashMap::new(),
            bounds: None,
        }
    }

    pub fn update_data(&mut self, data: &RecordBatch) -> Result<(), String> {
        self.points.clear();
        self.categories.clear();
        self.color_map.clear();

        let num_rows = data.num_rows();
        if num_rows == 0 {
            return Ok(());
        }

        // Get column arrays
        let x_values = self.get_numeric_column(data, &self.config.x_column)?;
        let y_values = self.get_numeric_column(data, &self.config.y_column)?;
        
        let color_values = if let Some(ref col) = self.config.color_column {
            Some(self.get_string_column(data, col)?)
        } else {
            None
        };

        let size_values = if let Some(ref col) = self.config.size_column {
            Some(self.get_numeric_column(data, col)?)
        } else {
            None
        };

        // Build category color map
        if let Some(ref categories) = color_values {
            let unique_categories: std::collections::HashSet<String> = 
                categories.iter().cloned().collect();
            self.categories = unique_categories.into_iter().collect();
            
            for (i, category) in self.categories.iter().enumerate() {
                let color_hex = &self.config.color_palette[i % self.config.color_palette.len()];
                self.color_map.insert(category.clone(), hex_to_color32(color_hex));
            }
        }

        // Create points
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let limit = self.config.point_limit.min(num_rows);
        let step = if num_rows > limit { num_rows / limit } else { 1 };

        for i in (0..num_rows).step_by(step) {
            if self.points.len() >= limit {
                break;
            }

            let x = x_values[i];
            let y = y_values[i];
            
            if x.is_finite() && y.is_finite() {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);

                let category = color_values.as_ref().map(|cats| cats[i].clone());
                let color = if let Some(ref cat) = category {
                    *self.color_map.get(cat).unwrap_or(&Color32::BLUE)
                } else {
                    Color32::BLUE
                };

                let size = if let Some(ref sizes) = size_values {
                    (sizes[i] as f32 * self.config.marker_size).max(1.0)
                } else {
                    self.config.marker_size
                };

                let tooltip = format!(
                    "{}: {:.3}\n{}: {:.3}{}{}",
                    self.config.x_column, x,
                    self.config.y_column, y,
                    if let Some(ref cat) = category {
                        format!("\nCategory: {}", cat)
                    } else {
                        String::new()
                    },
                    if let Some(ref sizes) = size_values {
                        format!("\nSize: {:.3}", sizes[i])
                    } else {
                        String::new()
                    }
                );

                self.points.push(ScatterPoint {
                    x,
                    y,
                    category,
                    size,
                    color,
                    tooltip,
                });
            }
        }

        self.bounds = Some((min_x, max_x, min_y, max_y));
        Ok(())
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let mut plot = Plot::new("enhanced_scatter_plot")
            .legend(Legend::default().position(Corner::RightTop))
            .show_axes([true, true])
            .show_grid([true, true])
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true);

        if let Some((min_x, max_x, min_y, max_y)) = self.bounds {
            let margin = 0.05;
            let x_range = max_x - min_x;
            let y_range = max_y - min_y;
            plot = plot.include_x(min_x - x_range * margin)
                      .include_x(max_x + x_range * margin)
                      .include_y(min_y - y_range * margin)
                      .include_y(max_y + y_range * margin);
        }

        plot.show(ui, |plot_ui| {
            if self.config.color_column.is_some() && self.config.show_legend {
                // Group points by category for legend
                let mut category_points: HashMap<String, Vec<PlotPoint>> = HashMap::new();
                
                for point in &self.points {
                    if let Some(ref category) = point.category {
                        category_points.entry(category.clone())
                            .or_insert_with(Vec::new)
                            .push(PlotPoint::new(point.x, point.y));
                    }
                }

                // Plot each category separately for legend
                for (category, points) in category_points {
                    let color = self.color_map.get(&category).copied().unwrap_or(Color32::BLUE);
                    let plot_points: Vec<[f64; 2]> = points.iter()
                        .map(|p| [p.x, p.y])
                        .collect();
                    
                    plot_ui.points(
                        Points::new(PlotPoints::new(plot_points))
                            .name(&category)
                            .color(color)
                            .radius(self.config.marker_size)
                            .shape(string_to_marker_shape(&self.config.marker_shape))
                    );
                }
            } else {
                // Plot all points as single series
                let plot_points: Vec<[f64; 2]> = self.points.iter()
                    .map(|p| [p.x, p.y])
                    .collect();
                
                plot_ui.points(
                    Points::new(PlotPoints::new(plot_points))
                        .name("Data")
                        .color(Color32::BLUE)
                        .radius(self.config.marker_size)
                        .shape(string_to_marker_shape(&self.config.marker_shape))
                );
            }
        }).response
    }

    fn get_numeric_column(&self, data: &RecordBatch, column_name: &str) -> Result<Vec<f64>, String> {
        let column = data.column_by_name(column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        if let Some(float_array) = column.as_any().downcast_ref::<Float64Array>() {
            Ok((0..float_array.len())
                .map(|i| float_array.value(i))
                .collect())
        } else if let Some(int_array) = column.as_any().downcast_ref::<Int64Array>() {
            Ok((0..int_array.len())
                .map(|i| int_array.value(i) as f64)
                .collect())
        } else {
            Err(format!("Column '{}' is not numeric", column_name))
        }
    }

    fn get_string_column(&self, data: &RecordBatch, column_name: &str) -> Result<Vec<String>, String> {
        let column = data.column_by_name(column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        if let Some(string_array) = column.as_any().downcast_ref::<StringArray>() {
            Ok((0..string_array.len())
                .map(|i| string_array.value(i).to_string())
                .collect())
        } else {
            Err(format!("Column '{}' is not a string column", column_name))
        }
    }
}

fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let Ok(rgb) = u32::from_str_radix(hex, 16) {
            return Color32::from_rgb(
                ((rgb >> 16) & 0xFF) as u8,
                ((rgb >> 8) & 0xFF) as u8,
                (rgb & 0xFF) as u8,
            );
        }
    }
    Color32::BLUE // fallback
}

fn string_to_marker_shape(shape: &str) -> MarkerShape {
    match shape {
        "Circle" => MarkerShape::Circle,
        "Square" => MarkerShape::Square,
        "Diamond" => MarkerShape::Diamond,
        "Up" => MarkerShape::Up,
        "Down" => MarkerShape::Down,
        "Left" => MarkerShape::Left,
        "Right" => MarkerShape::Right,
        "Plus" => MarkerShape::Plus,
        "Cross" => MarkerShape::Cross,
        _ => MarkerShape::Circle,
    }
}

// Configuration UI
pub fn show_scatter_config(ui: &mut Ui, config: &mut EnhancedScatterConfig, table_info: &TableInfo) {
    ui.heading("Scatter Plot Configuration");
    
    ui.horizontal(|ui| {
        ui.label("X Column:");
        egui::ComboBox::from_id_source("x_column")
            .selected_text(&config.x_column)
            .show_ui(ui, |ui| {
                for column in &table_info.columns {
                    ui.selectable_value(&mut config.x_column, column.name.clone(), &column.name);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("Y Column:");
        egui::ComboBox::from_id_source("y_column")
            .selected_text(&config.y_column)
            .show_ui(ui, |ui| {
                for column in &table_info.columns {
                    ui.selectable_value(&mut config.y_column, column.name.clone(), &column.name);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("Color Column:");
        let current_color = config.color_column.as_ref().map(|s| s.as_str()).unwrap_or("None");
        egui::ComboBox::from_id_source("color_column")
            .selected_text(current_color)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.color_column, None, "None");
                for column in &table_info.columns {
                    ui.selectable_value(&mut config.color_column, Some(column.name.clone()), &column.name);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("Size Column:");
        let current_size = config.size_column.as_ref().map(|s| s.as_str()).unwrap_or("None");
        egui::ComboBox::from_id_source("size_column")
            .selected_text(current_size)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.size_column, None, "None");
                for column in &table_info.columns {
                    ui.selectable_value(&mut config.size_column, Some(column.name.clone()), &column.name);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("Marker Shape:");
        egui::ComboBox::from_id_source("marker_shape")
            .selected_text(&config.marker_shape)
            .show_ui(ui, |ui| {
                for shape in &["Circle", "Square", "Diamond", "Up", "Down", "Left", "Right", "Plus", "Cross"] {
                    ui.selectable_value(&mut config.marker_shape, shape.to_string(), *shape);
                }
            });
    });

    ui.horizontal(|ui| {
        ui.label("Marker Size:");
        ui.add(egui::Slider::new(&mut config.marker_size, 1.0..=20.0));
    });

    ui.horizontal(|ui| {
        ui.label("Point Limit:");
        ui.add(egui::Slider::new(&mut config.point_limit, 100..=50000).logarithmic(true));
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut config.show_legend, "Show Legend");
        ui.checkbox(&mut config.show_tooltips, "Show Tooltips");
    });

    ui.horizontal(|ui| {
        ui.label("Alpha:");
        ui.add(egui::Slider::new(&mut config.alpha, 0.1..=1.0));
    });
} 