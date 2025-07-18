use egui::{Ui, Color32, RichText};
use egui_plot::{Line, Plot, PlotPoints, Legend, PlotUi, Points, MarkerShape as EguiMarkerShape, 
                PlotBounds, Polygon, PlotPoint as EguiPlotPoint, LineStyle as EguiLineStyle};
use datafusion::arrow::datatypes::{DataType, TimeUnit};
use std::collections::HashMap;
use crate::core::QueryResult;

use super::{
    Plot as PlotTrait, 
    PlotData, 
    PlotConfiguration, 
    PlotSpecificConfig, 
    LineChartConfig, 
    LineStyle,
    PlotInteraction,
    DataSeries,
    SeriesStyle,
    MarkerShape,
    PlotMetadata,
    DataStatistics,
    data_processor::DataProcessor,
    // Enhanced utilities
    categorical_color, viridis_color, plasma_color, diverging_color,
    calculate_statistics, extract_numeric_values, extract_temporal_values,
    get_categorical_colors
};

pub struct LineChartPlot;

impl LineChartPlot {
    /// Enhanced data processing based on frog-viz patterns
    fn extract_temporal_points(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<super::PlotPoint>, String> {
        if config.y_column.is_empty() {
            return Err("Y column not selected".to_string());
        }

        let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
            .ok_or("Y column not found")?;
        
        let x_idx = if !config.x_column.is_empty() {
            query_result.columns.iter().position(|c| c == &config.x_column)
                .ok_or_else(|| format!("X column '{}' not found", config.x_column))?
        } else {
            return Err("X column is required for temporal data".to_string());
        };

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
        let mut color_map = HashMap::new();
        let mut color_index = 0;
        
        // Enhanced temporal data handling based on frog-viz patterns
        for (_row_idx, row) in query_result.rows.iter().enumerate() {
            if row.len() > y_idx && row.len() > x_idx {
                let y_val = row[y_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse Y value '{}' as number", row[y_idx]))?;
                
                // Enhanced temporal value parsing with proper error handling
                let x_val = match &query_result.column_types[x_idx] {
                    DataType::Date32 => {
                        let days = row[x_idx].parse::<i32>()
                            .map_err(|_| format!("Failed to parse Date32 value '{}'", row[x_idx]))?;
                        (days as f64) * 86400000.0 // days to ms
                    },
                    DataType::Date64 => {
                        row[x_idx].parse::<f64>()
                            .map_err(|_| format!("Failed to parse Date64 value '{}'", row[x_idx]))?
                    },
                    DataType::Timestamp(time_unit, _) => {
                        let timestamp = row[x_idx].parse::<i64>()
                            .map_err(|_| format!("Failed to parse Timestamp value '{}'", row[x_idx]))?;
                        
                        match time_unit {
                            TimeUnit::Second => (timestamp as f64) * 1000.0,
                            TimeUnit::Millisecond => timestamp as f64,
                            TimeUnit::Microsecond => (timestamp as f64) / 1000.0,
                            TimeUnit::Nanosecond => (timestamp as f64) / 1_000_000.0,
                        }
                    },
                    DataType::Time32(time_unit) => {
                        let time_val = row[x_idx].parse::<i32>()
                            .map_err(|_| format!("Failed to parse Time32 value '{}'", row[x_idx]))?;
                        
                        match time_unit {
                            TimeUnit::Second => (time_val as f64) * 1000.0,
                            TimeUnit::Millisecond => time_val as f64,
                            _ => time_val as f64,
                        }
                    },
                    DataType::Time64(time_unit) => {
                        let time_val = row[x_idx].parse::<i64>()
                            .map_err(|_| format!("Failed to parse Time64 value '{}'", row[x_idx]))?;
                        
                        match time_unit {
                            TimeUnit::Microsecond => (time_val as f64) / 1000.0,
                            TimeUnit::Nanosecond => (time_val as f64) / 1_000_000.0,
                            _ => time_val as f64,
                        }
                    },
                    _ => {
                        row[x_idx].parse::<f64>()
                            .map_err(|_| format!("Failed to parse X value '{}' as number", row[x_idx]))?
                    }
                };

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

                // Enhanced tooltip data with rich information
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("X".to_string(), row[x_idx].clone());
                tooltip_data.insert("Y".to_string(), row[y_idx].clone());
                
                if let Some(color_idx) = color_idx {
                    if row.len() > color_idx {
                        tooltip_data.insert(config.color_column.as_ref().unwrap().clone(), row[color_idx].clone());
                    }
                }

                points.push(super::PlotPoint {
                    x: x_val,
                    y: y_val,
                    z: None,
                    label: None,
                    color: point_color,
                    size: None,
                    series_id: None,
                    tooltip_data,
                });
            }
        }
        
        Ok(points)
    }
    
    /// Enhanced missing data handling based on frog-viz patterns
    fn handle_missing_data(&self, series: &mut Vec<DataSeries>, config: &PlotConfiguration) {
        let _line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
            cfg
        } else {
            return;
        };
        
        // For each series, check for large gaps that might indicate missing data
        for series in series.iter_mut() {
            if series.points.len() < 2 {
                continue;
            }
            
            // Sort points by X value
            series.points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            
            // Calculate average X distance between consecutive points
            let mut total_distance = 0.0;
            let mut count = 0;
            
            for i in 1..series.points.len() {
                let distance = (series.points[i].x - series.points[i-1].x).abs();
                total_distance += distance;
                count += 1;
            }
            
            if count == 0 {
                continue;
            }
            
            let avg_distance = total_distance / count as f64;
            let gap_threshold = avg_distance * 3.0; // Consider a gap if distance is 3x the average
            
            // Mark gaps by setting z value to indicate a discontinuity
            for i in 1..series.points.len() {
                let distance = (series.points[i].x - series.points[i-1].x).abs();
                if distance > gap_threshold {
                    // Mark this as a gap by setting z to a special value
                    series.points[i].z = Some(-1.0); // Special value to indicate gap
                }
            }
        }
    }
    
    /// Create fill area for line charts
    fn create_fill_area(&self, series: &DataSeries, plot_ui: &mut PlotUi, bounds: &PlotBounds) {
        if series.points.len() < 2 {
            return;
        }
        
        // Create polygon points for fill area
        let mut polygon_points = Vec::new();
        
        // Add points from the line
        for point in &series.points {
            polygon_points.push([point.x, point.y]);
        }
        
        // Add points from the bottom of the plot to close the polygon
        for point in series.points.iter().rev() {
            polygon_points.push([point.x, bounds.min()[1]]);
        }
        
        if polygon_points.len() >= 3 {
            let fill_color = series.color.linear_multiply(0.3); // Semi-transparent
            let polygon = Polygon::new(PlotPoints::from(polygon_points))
                .fill_color(fill_color);
            plot_ui.polygon(polygon);
        }
    }
    
    /// Enhanced tooltip handling with better positioning and information
    fn handle_tooltips(&self, ui: &mut Ui, plot_ui: &PlotUi, data: &PlotData) {
        if let Some(pointer_coord) = plot_ui.pointer_coordinate() {
            let mut closest_point = None;
            let mut min_distance = f64::MAX;
            
            // Find the closest point to the cursor
            for series in &data.series {
                for point in &series.points {
                    let dx = point.x - pointer_coord.x;
                    let dy = point.y - pointer_coord.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance < min_distance && distance < 0.1 { // Threshold for detection
                        min_distance = distance;
                        closest_point = Some((point, series));
                    }
                }
            }
            
            // Show tooltip for the closest point
            if let Some((point, series)) = closest_point {
                let mut tooltip_text = String::new();
                tooltip_text.push_str(&format!("Series: {}\n", series.name));
                tooltip_text.push_str(&format!("X: {:.3}\n", point.x));
                tooltip_text.push_str(&format!("Y: {:.3}", point.y));
                
                // Add additional tooltip data if available
                for (key, value) in &point.tooltip_data {
                    if key != "X" && key != "Y" {
                        tooltip_text.push_str(&format!("\n{}: {}", key, value));
                    }
                }
                
                // Show tooltip at pointer position
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("line_tooltip")),
                    egui::Id::new("line_tooltip"),
                    |ui: &mut egui::Ui| {
                        ui.label(RichText::new(tooltip_text).monospace());
                    }
                );
            }
        }
    }
    
    /// Convert marker shape to egui marker shape
    fn to_egui_marker_shape(shape: &MarkerShape) -> EguiMarkerShape {
        match shape {
            MarkerShape::Circle => EguiMarkerShape::Circle,
            MarkerShape::Square => EguiMarkerShape::Square,
            MarkerShape::Diamond => EguiMarkerShape::Diamond,
            MarkerShape::Triangle => EguiMarkerShape::Up,
            MarkerShape::Cross => EguiMarkerShape::Cross,
            MarkerShape::Plus => EguiMarkerShape::Plus,
            MarkerShape::Star => EguiMarkerShape::Asterisk,
        }
    }
    
    /// Enhanced data processing with professional series grouping
    fn process_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        let data_processor = DataProcessor::new();
        
        // Extract points with temporal support
        let points = self.extract_temporal_points(query_result, config)?;
        
        if points.is_empty() {
            return Err("No valid data points found".to_string());
        }
        
        // Group points by series if color column is specified
        let mut series = Vec::new();
        
        if let Some(color_col) = &config.color_column {
            if !color_col.is_empty() {
                // Group by color column
                let mut grouped_data: HashMap<String, Vec<super::PlotPoint>> = HashMap::new();
                
                for point in points {
                    let series_name = point.tooltip_data.get(color_col)
                        .unwrap_or(&"default".to_string())
                        .clone();
                    grouped_data.entry(series_name).or_insert_with(Vec::new).push(point);
                }
                
                // Create series for each group
                for (i, (series_name, series_points)) in grouped_data.into_iter().enumerate() {
                    let color = categorical_color(i);
                    let mut data_series = DataSeries {
                        id: series_name.clone(),
                        name: series_name,
                        points: series_points,
                        color,
                        visible: true,
                        style: SeriesStyle::Lines { width: config.line_width, style: LineStyle::Solid },
                    };
                    
                    // Handle missing data
                    self.handle_missing_data(&mut vec![data_series.clone()], config);
                    
                    series.push(data_series);
                }
            } else {
                // Single series
                let mut data_series = DataSeries {
                    id: "main".to_string(),
                    name: "Line".to_string(),
                    points,
                    color: categorical_color(0),
                    visible: true,
                    style: SeriesStyle::Lines { width: config.line_width, style: LineStyle::Solid },
                };
                
                self.handle_missing_data(&mut vec![data_series.clone()], config);
                series.push(data_series);
            }
        } else {
            // Single series
            let mut data_series = DataSeries {
                id: "main".to_string(),
                name: "Line".to_string(),
                points,
                color: categorical_color(0),
                visible: true,
                style: SeriesStyle::Lines { width: config.line_width, style: LineStyle::Solid },
            };
            
            self.handle_missing_data(&mut vec![data_series.clone()], config);
            series.push(data_series);
        }
        
        Ok(series)
    }
    
    /// Get line pattern for different line styles
    fn get_line_pattern(&self, style: &LineStyle) -> &'static [f32] {
        match style {
            LineStyle::Solid => &[],
            LineStyle::Dashed => &[5.0, 5.0],
            LineStyle::Dotted => &[2.0, 2.0],
            LineStyle::DashDot => &[5.0, 2.0, 2.0, 2.0],
        }
    }
}

impl PlotTrait for LineChartPlot {
    fn name(&self) -> &'static str {
        "Line Chart"
    }

    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![
            DataType::Float64, DataType::Float32,
            DataType::Int64, DataType::Int32, DataType::Int16, DataType::Int8,
            DataType::UInt64, DataType::UInt32, DataType::UInt16, DataType::UInt8,
            DataType::Date32, DataType::Date64,
            DataType::Timestamp(TimeUnit::Second, None),
            DataType::Timestamp(TimeUnit::Millisecond, None),
            DataType::Timestamp(TimeUnit::Microsecond, None),
            DataType::Timestamp(TimeUnit::Nanosecond, None),
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

    fn get_default_config(&self) -> PlotConfiguration {
        PlotConfiguration {
            title: "Line Chart".to_string(),
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
            plot_specific: PlotSpecificConfig::LineChart(LineChartConfig::default()),
        }
    }

    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        let series = self.process_data(query_result, config)?;
        
        // Calculate statistics for the data
        let statistics = if !series.is_empty() && !series[0].points.is_empty() {
            let y_values: Vec<f64> = series.iter()
                .flat_map(|s| s.points.iter().map(|p| p.y))
                .collect();
            
            if !y_values.is_empty() {
                let stats = calculate_statistics(&y_values);
                Some(DataStatistics {
                    mean_x: 0.0, // Would need X values to calculate
                    mean_y: stats.mean,
                    std_x: 0.0, // Would need X values to calculate
                    std_y: stats.std_dev,
                    correlation: None, // Would need both X and Y to calculate
                    count: stats.count,
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
        let line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
            cfg
        } else {
            return;
        };

        // Create plot with proper configuration
        let mut plot = Plot::new("line_chart")
            .allow_zoom(config.allow_zoom)
            .allow_drag(config.allow_pan)
            .show_grid(config.show_grid)
            .legend(Legend::default().position(egui_plot::Corner::RightBottom));

        // Add axis labels if enabled
        if config.show_axes_labels {
            plot = plot
                .x_axis_label(config.x_column.clone())
                .y_axis_label(config.y_column.clone());
        }

        // Add title if provided
        if !config.title.is_empty() {
            // Note: egui_plot doesn't have a title method, we'll handle this differently
        }

        plot.show(ui, |plot_ui| {
            for series in &data.series {
                if !series.visible {
                    continue;
                }

                // Draw lines between points
                if series.points.len() > 1 {
                    let line_points: Vec<[f64; 2]> = series.points.iter()
                        .filter(|p| p.z != Some(-1.0)) // Skip gap points
                        .map(|p| [p.x, p.y])
                        .collect();

                    if line_points.len() >= 2 {
                        let mut line = Line::new(PlotPoints::from(line_points))
                            .color(series.color)
                            .width(config.line_width);

                        // Apply line style
                        if let SeriesStyle::Lines { style, .. } = &series.style {
                            let pattern = self.get_line_pattern(style);
                            if !pattern.is_empty() {
                                line = line.style(egui_plot::LineStyle::dashed_dense());
                            }
                        }

                        plot_ui.line(line);
                    }
                }

                // Draw points if enabled
                if line_config.show_points {
                    for point in &series.points {
                        if point.z != Some(-1.0) { // Skip gap points
                            let points = Points::new(PlotPoints::from(vec![[point.x, point.y]]))
                                .color(series.color)
                                .radius(config.marker_size)
                                .shape(EguiMarkerShape::Circle);
                            plot_ui.points(points);
                        }
                    }
                }

                // Create fill area if enabled
                if line_config.fill_area {
                    self.create_fill_area(series, plot_ui, &plot_ui.plot_bounds());
                }
            }
        });

        // Handle tooltips outside the closure to avoid borrow checker issues
        if config.show_tooltips {
            // Note: plot_ui is not available outside the closure
            // We'll handle tooltips differently
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
                        match &series.style {
                            SeriesStyle::Points { size: _, shape } => {
                                let shape_text = match shape {
                                    MarkerShape::Circle => "●",
                                    MarkerShape::Square => "■",
                                    MarkerShape::Diamond => "◆",
                                    MarkerShape::Triangle => "▲",
                                    MarkerShape::Cross => "✚",
                                    MarkerShape::Plus => "➕",
                                    MarkerShape::Star => "★",
                                };
                                ui.colored_label(series.color, shape_text);
                            },
                            SeriesStyle::Lines { width: _, style } => {
                                let style_text = match style {
                                    LineStyle::Solid => "———",
                                    LineStyle::Dashed => "---",
                                    LineStyle::Dotted => "...",
                                    LineStyle::DashDot => "-.-.",
                                };
                                ui.colored_label(series.color, style_text);
                            },
                            SeriesStyle::Bars { width: _ } => {
                                ui.colored_label(series.color, "■");
                            },
                            SeriesStyle::Area { fill: _ } => {
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
                        ui.label("Mean Y:");
                        ui.label(format!("{:.3}", stats.mean_y));
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