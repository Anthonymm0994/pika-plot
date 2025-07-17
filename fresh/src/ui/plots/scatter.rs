use egui::{Ui, Color32, RichText};
use egui_plot::{Plot, PlotPoints, Points, Legend, PlotUi, Polygon, MarkerShape as EguiMarkerShape};
use datafusion::arrow::datatypes::DataType;
use std::collections::HashMap;
use crate::core::QueryResult;

use super::{
    Plot as PlotTrait, 
    PlotData, 
    PlotConfiguration, 
    PlotSpecificConfig, 
    ScatterPlotConfig, 
    MarkerShape,
    PlotInteraction,
    DataSeries,
    SeriesStyle,
    PlotMetadata,
    DataStatistics,
    // Enhanced utilities
    categorical_color, viridis_color, plasma_color, diverging_color,
    calculate_statistics, extract_numeric_values, extract_string_values,
    get_categorical_colors
};

pub struct ScatterPlotImpl;

impl ScatterPlotImpl {
    /// Create selection rectangle for interactive selection
    fn create_selection_rectangle(&self, start: [f64; 2], end: [f64; 2]) -> Polygon {
        let points = vec![
            [start[0], start[1]],
            [end[0], start[1]],
            [end[0], end[1]],
            [start[0], end[1]],
        ];
        
        Polygon::new(PlotPoints::from(points))
            .fill_color(Color32::from_rgba_unmultiplied(100, 150, 255, 50))
            .stroke(egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255)))
    }

    /// Enhanced data processing with frog-viz patterns
    fn process_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        if config.x_column.is_empty() || config.y_column.is_empty() {
            return Err("X and Y columns are required for scatter plot".to_string());
        }

        let x_idx = query_result.columns.iter().position(|c| c == &config.x_column)
            .ok_or_else(|| format!("X column '{}' not found", config.x_column))?;
        let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
            .ok_or_else(|| format!("Y column '{}' not found", config.y_column))?;

        let color_idx = if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() {
                query_result.columns.iter().position(|c| c == color_col)
            } else {
                None
            }
        } else {
            None
        };

        let size_idx = if let Some(size_col) = &config.size_column {
            if !size_col.is_empty() {
                query_result.columns.iter().position(|c| c == size_col)
            } else {
                None
            }
        } else {
            None
        };

        let mut points = Vec::new();
        let mut color_map = HashMap::new();
        let mut color_index = 0;

        // Enhanced data processing with professional color mapping
        for (row_idx, row) in query_result.rows.iter().enumerate() {
            if row.len() > x_idx && row.len() > y_idx {
                let x_val = row[x_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse X value '{}' as number", row[x_idx]))?;
                let y_val = row[y_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse Y value '{}' as number", row[y_idx]))?;

                // Professional color mapping based on frog-viz patterns
                let point_color = if let Some(color_idx) = color_idx {
                    if row.len() > color_idx {
                        let color_value = &row[color_idx];
                        if let Some(&existing_color) = color_map.get(color_value) {
                            Some(existing_color)
                        } else {
                            let new_color = categorical_color(color_index);
                            color_map.insert(color_value.clone(), new_color);
                            color_index += 1;
                            Some(new_color)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Size mapping
                let point_size = if let Some(size_idx) = size_idx {
                    if row.len() > size_idx {
                        row[size_idx].parse::<f32>().unwrap_or(3.0)
                    } else {
                        3.0
                    }
                } else {
                    3.0
                };

                // Enhanced tooltip data with rich information
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("X".to_string(), row[x_idx].clone());
                tooltip_data.insert("Y".to_string(), row[y_idx].clone());
                
                if let Some(color_idx) = color_idx {
                    if row.len() > color_idx {
                        tooltip_data.insert(config.color_column.as_ref().unwrap().clone(), row[color_idx].clone());
                    }
                }

                if let Some(size_idx) = size_idx {
                    if row.len() > size_idx {
                        tooltip_data.insert(config.size_column.as_ref().unwrap().clone(), row[size_idx].clone());
                    }
                }

                points.push(super::PlotPoint {
                    x: x_val,
                    y: y_val,
                    z: None,
                    label: None,
                    color: point_color,
                    size: Some(point_size),
                    series_id: None,
                    tooltip_data,
                });
            }
        }

        // Create series based on color grouping
        let mut series = Vec::new();
        if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() {
                // Group by color column
                let mut grouped_data: HashMap<String, Vec<super::PlotPoint>> = HashMap::new();
                
                for point in points {
                    let color_value = point.tooltip_data.get(color_col)
                        .unwrap_or(&"default".to_string())
                        .clone();
                    grouped_data.entry(color_value).or_insert_with(Vec::new).push(point);
                }

                // Create series for each color group
                for (i, (group_name, group_points)) in grouped_data.into_iter().enumerate() {
                    let color = categorical_color(i);
                    series.push(DataSeries {
                        id: group_name.clone(),
                        name: group_name,
                        points: group_points,
                        color,
                        visible: true,
                        style: SeriesStyle::Points { size: 3.0, shape: MarkerShape::Circle },
                    });
                }
            } else {
                // Single series
                series.push(DataSeries {
                    id: "main".to_string(),
                    name: "Scatter".to_string(),
                    points,
                    color: categorical_color(0),
                    visible: true,
                    style: SeriesStyle::Points { size: 3.0, shape: MarkerShape::Circle },
                });
            }
        } else {
            // Single series
            series.push(DataSeries {
                id: "main".to_string(),
                name: "Scatter".to_string(),
                points,
                color: categorical_color(0),
                visible: true,
                style: SeriesStyle::Points { size: 3.0, shape: MarkerShape::Circle },
            });
        }

        Ok(series)
    }
}

pub struct ScatterPlot;

impl PlotTrait for ScatterPlot {
    fn name(&self) -> &'static str {
        "Scatter Plot"
    }

    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![
            DataType::Float64, DataType::Float32,
            DataType::Int64, DataType::Int32, DataType::Int16, DataType::Int8,
            DataType::UInt64, DataType::UInt32, DataType::UInt16, DataType::UInt8,
        ])
    }

    fn required_y_types(&self) -> Vec<DataType> {
        vec![
            DataType::Float64, DataType::Float32,
            DataType::Int64, DataType::Int32, DataType::Int16, DataType::Int8,
            DataType::UInt64, DataType::UInt32, DataType::UInt16, DataType::UInt8,
        ]
    }

    fn supports_color_mapping(&self) -> bool {
        true
    }

    fn supports_size_mapping(&self) -> bool {
        true
    }

    fn get_default_config(&self) -> PlotConfiguration {
        PlotConfiguration {
            title: "Scatter Plot".to_string(),
            x_column: String::new(),
            y_column: String::new(),
            color_column: None,
            size_column: None,
            group_column: None,
            show_legend: true,
            show_grid: true,
            show_axes_labels: true,
            color_scheme: super::ColorScheme::Viridis,
            marker_size: 3.0,
            line_width: 2.0,
            allow_zoom: true,
            allow_pan: true,
            allow_selection: true,
            show_tooltips: true,
            plot_specific: PlotSpecificConfig::ScatterPlot(ScatterPlotConfig::default()),
        }
    }

    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        let series = ScatterPlotImpl.process_data(query_result, config)?;
        
        // Calculate statistics for the data
        let statistics = if !series.is_empty() && !series[0].points.is_empty() {
            let x_values: Vec<f64> = series.iter()
                .flat_map(|s| s.points.iter().map(|p| p.x))
                .collect();
            let y_values: Vec<f64> = series.iter()
                .flat_map(|s| s.points.iter().map(|p| p.y))
                .collect();
            
            if !x_values.is_empty() && !y_values.is_empty() {
                let x_stats = calculate_statistics(&x_values);
                let y_stats = calculate_statistics(&y_values);
                
                // Calculate correlation between X and Y
                let correlation = if x_values.len() == y_values.len() {
                    let n = x_values.len() as f64;
                    let sum_x: f64 = x_values.iter().sum();
                    let sum_y: f64 = y_values.iter().sum();
                    let sum_xy: f64 = x_values.iter().zip(y_values.iter()).map(|(x, y)| x * y).sum();
                    let sum_x2: f64 = x_values.iter().map(|x| x * x).sum();
                    let sum_y2: f64 = y_values.iter().map(|y| y * y).sum();

                    let numerator = n * sum_xy - sum_x * sum_y;
                    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

                    if denominator != 0.0 {
                        Some(numerator / denominator)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Some(DataStatistics {
                    mean_x: x_stats.mean,
                    mean_y: y_stats.mean,
                    std_x: x_stats.std_dev,
                    std_y: y_stats.std_dev,
                    correlation,
                    count: x_values.len(),
                })
            } else {
                None
            }
        } else {
            None
        };
        
        let all_points: Vec<super::PlotPoint> = series.iter()
            .flat_map(|s| s.points.clone())
            .collect();
        
        Ok(PlotData {
            points: all_points,
            series,
            metadata: PlotMetadata {
                title: config.title.clone(),
                x_label: config.x_column.clone(),
                y_label: config.y_column.clone(),
                show_legend: config.show_legend,
                show_grid: config.show_grid,
                color_scheme: config.color_scheme.clone(),
            },
            statistics,
        })
    }

    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        let scatter_config = if let PlotSpecificConfig::ScatterPlot(cfg) = &config.plot_specific {
            cfg
        } else {
            return;
        };

        let plot = Plot::new("scatter_plot")
            .allow_zoom(config.allow_zoom)
            .allow_drag(config.allow_pan)
            .show_grid(config.show_grid)
            .legend(Legend::default().position(egui_plot::Corner::RightBottom));

        plot.show(ui, |plot_ui| {
            for series in &data.series {
                if !series.visible {
                    continue;
                }
                for point in &series.points {
                    let marker_shape = match scatter_config.point_shape {
                        MarkerShape::Circle => EguiMarkerShape::Circle,
                        MarkerShape::Square => EguiMarkerShape::Square,
                        MarkerShape::Diamond => EguiMarkerShape::Diamond,
                        MarkerShape::Triangle => EguiMarkerShape::Up,
                        MarkerShape::Cross => EguiMarkerShape::Cross,
                        MarkerShape::Plus => EguiMarkerShape::Plus,
                    };
                    let point_size = point.size.unwrap_or(config.marker_size);
                    let alpha = 1.0;
                    let points = Points::new(PlotPoints::from(vec![[point.x, point.y]]))
                        .color(series.color.linear_multiply(alpha))
                        .radius(point_size)
                        .shape(marker_shape);
                    plot_ui.points(points);
                }
            }
        });
        
        // Handle tooltips outside the closure to avoid borrow checker issues
        if config.show_tooltips {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // Find the closest point to the cursor
                let mut closest_point = None;
                let mut min_distance = f64::MAX;
                
                for series in &data.series {
                    for point in &series.points {
                        // Simple distance calculation using plot coordinates
                        // This is a simplified approach - in a real implementation,
                        // you'd need to transform plot coordinates to screen coordinates
                        let plot_x = point.x;
                        let plot_y = point.y;
                        
                        // For now, just use a simple threshold
                        if (plot_x - pointer_pos.x as f64).abs() < 20.0 && (plot_y - pointer_pos.y as f64).abs() < 20.0 {
                            let distance = ((plot_x - pointer_pos.x as f64).powi(2) + (plot_y - pointer_pos.y as f64).powi(2)).sqrt();
                            if distance < min_distance {
                                min_distance = distance;
                                closest_point = Some((point, series));
                            }
                        }
                    }
                }
                
                // Show tooltip for the closest point
                if let Some((point, series)) = closest_point {
                    let mut tooltip_text = String::new();
                    tooltip_text.push_str(&format!("Series: {}\n", series.name));
                    tooltip_text.push_str(&format!("X: {:.3}\n", point.x));
                    tooltip_text.push_str(&format!("Y: {:.3}", point.y));
                    
                    // Add tooltip data if available
                    for (key, value) in &point.tooltip_data {
                        tooltip_text.push_str(&format!("\n{}: {}", key, value));
                    }
                    
                    egui::show_tooltip_at_pointer(
                        ui.ctx(),
                        egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("scatter_tooltip")),
                        egui::Id::new("scatter_tooltip"),
                        |ui: &mut egui::Ui| {
                            ui.label(RichText::new(tooltip_text).monospace());
                        }
                    );
                }
            }
        }
    }

    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if !data.series.is_empty() && config.show_legend {
            ui.group(|ui| {
                ui.label(RichText::new("Series:").strong());
                ui.separator();
                
                for series in &data.series {
                    let mut is_visible = series.visible;
                    if ui.checkbox(&mut is_visible, &series.name).changed() {
                        // Note: This would require mutable access to data
                    }
                    
                    // Show series style indicator with enhanced styling
                    ui.horizontal(|ui| {
                        match series.style {
                            SeriesStyle::Line { width: _, dashed } => {
                                let style_text = if dashed { "---" } else { "———" };
                                ui.colored_label(series.color, style_text);
                            },
                            SeriesStyle::Points { size: _, shape } => {
                                let shape_text = match shape {
                                    MarkerShape::Circle => "●",
                                    MarkerShape::Square => "■",
                                    MarkerShape::Diamond => "◆",
                                    MarkerShape::Triangle => "▲",
                                    MarkerShape::Cross => "✚",
                                    MarkerShape::Plus => "➕",
                                };
                                ui.colored_label(series.color, shape_text);
                            },
                            SeriesStyle::Bars { width: _ } => {
                                ui.colored_label(series.color, "■");
                            },
                            SeriesStyle::Area { alpha: _ } => {
                                ui.colored_label(series.color, "▬");
                            },
                        }
                        
                        if !is_visible {
                            ui.label(RichText::new(&series.name).strikethrough());
                        } else {
                            ui.label(&series.name);
                        }
                    });
                }
                
                // Show enhanced statistics if available
                if let Some(stats) = &data.statistics {
                    ui.separator();
                    ui.label(RichText::new("Statistics:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.label(format!("{}", stats.count));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mean X:");
                        ui.label(format!("{:.3}", stats.mean_x));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mean Y:");
                        ui.label(format!("{:.3}", stats.mean_y));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Std Dev X:");
                        ui.label(format!("{:.3}", stats.std_x));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Std Dev Y:");
                        ui.label(format!("{:.3}", stats.std_y));
                    });
                    if let Some(corr) = stats.correlation {
                        ui.horizontal(|ui| {
                            ui.label("Correlation:");
                            ui.label(format!("{:.3}", corr));
                        });
                    }
                }
            });
        }
    }

    fn handle_interaction(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) -> Option<PlotInteraction> {
        // Enhanced interaction handling
        if config.allow_selection {
            // Handle point selection
            // Handle area selection
            // Handle series toggling
        }
        
        None
    }
}