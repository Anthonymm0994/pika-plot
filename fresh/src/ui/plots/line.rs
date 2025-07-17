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
    data_processor::DataProcessor
};

pub struct LineChartPlot;

impl LineChartPlot {
    /// Extract and optimize temporal data points
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
        let colors = super::get_categorical_colors(&config.color_scheme);
        let mut color_map: HashMap<String, Color32> = HashMap::new();
        let mut color_index = 0;
        
        // Convert temporal values to numeric timestamps for plotting
        for (row_idx, row) in query_result.rows.iter().enumerate() {
            if row.len() > y_idx && row.len() > x_idx {
                let y_val = row[y_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse Y value '{}' as number", row[y_idx]))?;
                
                // Parse temporal value based on the data type
                let x_val = match &query_result.column_types[x_idx] {
                    DataType::Date32 => {
                        // Convert days since epoch to milliseconds
                        let days = row[x_idx].parse::<i32>()
                            .map_err(|_| format!("Failed to parse Date32 value '{}'", row[x_idx]))?;
                        (days as f64) * 86400000.0 // days to ms
                    },
                    DataType::Date64 => {
                        // Date64 is already milliseconds since epoch
                        row[x_idx].parse::<f64>()
                            .map_err(|_| format!("Failed to parse Date64 value '{}'", row[x_idx]))?
                    },
                    DataType::Timestamp(time_unit, _) => {
                        // Convert timestamp to milliseconds based on time unit
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
                        // Convert time of day to milliseconds
                        let time_val = row[x_idx].parse::<i32>()
                            .map_err(|_| format!("Failed to parse Time32 value '{}'", row[x_idx]))?;
                        
                        match time_unit {
                            TimeUnit::Second => (time_val as f64) * 1000.0,
                            TimeUnit::Millisecond => time_val as f64,
                            _ => time_val as f64, // Shouldn't happen for Time32
                        }
                    },
                    DataType::Time64(time_unit) => {
                        // Convert time of day to milliseconds
                        let time_val = row[x_idx].parse::<i64>()
                            .map_err(|_| format!("Failed to parse Time64 value '{}'", row[x_idx]))?;
                        
                        match time_unit {
                            TimeUnit::Microsecond => (time_val as f64) / 1000.0,
                            TimeUnit::Nanosecond => (time_val as f64) / 1_000_000.0,
                            _ => time_val as f64, // Shouldn't happen for Time64
                        }
                    },
                    _ => {
                        // Fallback to regular parsing for non-temporal types
                        row[x_idx].parse::<f64>()
                            .map_err(|_| format!("Failed to parse X value '{}' as number", row[x_idx]))?
                    }
                };

                // Handle color mapping
                let point_color = if let Some(color_idx) = color_idx {
                    if row.len() > color_idx {
                        let color_value = &row[color_idx];
                        if let Some(&existing_color) = color_map.get(color_value) {
                            Some(existing_color)
                        } else {
                            let new_color = colors[color_index % colors.len()];
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

                // Create tooltip data with formatted temporal value
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("X".to_string(), row[x_idx].clone()); // Original formatted date/time
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
    
    /// Handle missing data in line charts
    fn handle_missing_data(&self, series: &mut Vec<DataSeries>, config: &PlotConfiguration) {
        // Get line chart specific config
        let line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
            cfg
        } else {
            return; // Use default behavior if config not available
        };
        
        // For each series, check for large gaps that might indicate missing data
        for series in series.iter_mut() {
            if series.points.len() < 2 {
                continue; // Need at least 2 points to detect gaps
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
                    // Mark the previous point as end of segment
                    series.points[i-1].z = Some(-1.0);
                    // Mark the current point as start of new segment
                    series.points[i].z = Some(1.0);
                }
            }
        }
    }
    
    /// Create filled area under line if enabled
    fn create_fill_area(&self, series: &DataSeries, plot_ui: &mut PlotUi, bounds: &PlotBounds) {
        if series.points.len() < 2 {
            return;
        }
        
        // Create polygon points for the filled area
        let mut polygon_points = Vec::new();
        
        // Add the first point at the bottom
        if let Some(first) = series.points.first() {
            polygon_points.push(EguiPlotPoint::new(first.x, bounds.min()[1]));
        }
        
        // Add all the line points
        for point in &series.points {
            polygon_points.push(EguiPlotPoint::new(point.x, point.y));
        }
        
        // Add the last point at the bottom to close the polygon
        if let Some(last) = series.points.last() {
            polygon_points.push(EguiPlotPoint::new(last.x, bounds.min()[1]));
        }
        
        // Create a semi-transparent fill color
        let fill_color = Color32::from_rgba_unmultiplied(
            series.color.r(),
            series.color.g(),
            series.color.b(),
            50, // Low alpha for transparency
        );
        
        // Create and draw the polygon
        let polygon = Polygon::new(PlotPoints::from(
            polygon_points.iter().map(|p| [p.x, p.y]).collect::<Vec<[f64; 2]>>()
        ))
        .fill_color(fill_color)
        .stroke(egui::Stroke::NONE);
        
        plot_ui.polygon(polygon);
    }
    
    /// Handle tooltips for line chart
    fn handle_tooltips(&self, ui: &mut Ui, plot_ui: &PlotUi, data: &PlotData) {
        if let Some(pointer_coord) = plot_ui.pointer_coordinate() {
            // Find the closest point to the cursor
            let mut closest_point = None;
            let mut min_distance = f64::MAX;
            
            for point in &data.points {
                let dx = point.x - pointer_coord.x;
                let dy = point.y - pointer_coord.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < min_distance && distance < 0.5 { // Threshold for detection
                    min_distance = distance;
                    closest_point = Some(point);
                }
            }
            
            // Show tooltip for the closest point
            if let Some(point) = closest_point {
                // Show tooltip with point data
                let mut tooltip_text = String::new();
                
                if let Some(label) = &point.label {
                    tooltip_text.push_str(&format!("{}\n", label));
                }
                
                tooltip_text.push_str(&format!("X: {:.2}\n", point.x));
                tooltip_text.push_str(&format!("Y: {:.2}", point.y));
                
                // Add any additional tooltip data
                for (key, value) in &point.tooltip_data {
                    if key != "X" && key != "Y" {
                        tooltip_text.push_str(&format!("\n{}: {}", key, value));
                    }
                }
                
                // Tooltip functionality
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("line_tooltip")),
                    egui::Id::new("line_tooltip"),
                    |ui: &mut egui::Ui| {
                        ui.label(tooltip_text);
                    }
                );
                
                // Highlight the point
                let highlight_color = if let Some(color) = point.color {
                    // Make the color brighter for highlighting
                    Color32::from_rgb(
                        (color.r() as u16 + 40).min(255) as u8,
                        (color.g() as u16 + 40).min(255) as u8,
                        (color.b() as u16 + 40).min(255) as u8,
                    )
                } else {
                    Color32::from_rgb(120, 180, 250) // Highlight blue
                };
                
                // Draw highlight marker
                let highlight_points = Points::new(vec![[point.x, point.y]])
                    .color(highlight_color)
                    .radius(6.0)
                    .shape(EguiMarkerShape::Circle);
                
                // plot_ui.points(highlight_points);
            }
        }
    }
    
    /// Convert MarkerShape to egui_plot MarkerShape
    fn to_egui_marker_shape(shape: &MarkerShape) -> EguiMarkerShape {
        match shape {
            MarkerShape::Circle => EguiMarkerShape::Circle,
            MarkerShape::Square => EguiMarkerShape::Square,
            MarkerShape::Diamond => EguiMarkerShape::Diamond,
            MarkerShape::Triangle => EguiMarkerShape::Up, // Triangle is called "Up" in egui_plot
            MarkerShape::Cross => EguiMarkerShape::Cross,
            MarkerShape::Plus => EguiMarkerShape::Plus,
        }
    }
    
    /// Process data for line chart with proper grouping and temporal optimization
    fn process_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        // Get line chart specific config
        let line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
            cfg
        } else {
            &LineChartConfig {
                line_style: LineStyle::Solid,
                show_points: true,
                smooth_lines: false,
                fill_area: false,
            }
        };
        
        // Check if X column is a temporal type for optimization
        let is_temporal = if !config.x_column.is_empty() {
            let x_idx = query_result.columns.iter().position(|c| c == &config.x_column)
                .ok_or_else(|| format!("X column '{}' not found", config.x_column))?;
            
            if x_idx < query_result.column_types.len() {
                match &query_result.column_types[x_idx] {
                    DataType::Date32 | DataType::Date64 | 
                    DataType::Timestamp(_, _) | DataType::Time32(_) | DataType::Time64(_) => true,
                    _ => false
                }
            } else {
                false
            }
        } else {
            false
        };
        
        // Extract points from query result with temporal optimization if needed
        let points = if is_temporal {
            self.extract_temporal_points(query_result, config)?
        } else {
            super::extract_plot_points(query_result, config)?
        };
        
        // Group points by series if a group column is specified
        if let Some(group_col) = &config.group_column {
            // Create a map of series_id -> points
            let mut series_map: HashMap<String, Vec<super::PlotPoint>> = HashMap::new();
            
            for point in points {
                let series_id = if let Some(id) = &point.series_id {
                    id.clone()
                } else if let Some(value) = point.tooltip_data.get(group_col) {
                    value.clone()
                } else {
                    "default".to_string()
                };
                
                series_map.entry(series_id).or_default().push(point);
            }
            
            // Create a DataSeries for each group
            let colors = super::get_categorical_colors(&config.color_scheme);
            let mut series_vec = Vec::new();
            
            for (i, (id, mut points)) in series_map.into_iter().enumerate() {
                // Sort points by X value for proper line rendering
                points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
                
                let color = colors[i % colors.len()];
                
                // Set the color for all points in this series
                let points = points.into_iter()
                    .map(|mut p| {
                        p.color = Some(color);
                        p.series_id = Some(id.clone());
                        p
                    })
                    .collect();
                
                series_vec.push(DataSeries {
                    id: id.clone(),
                    name: id,
                    points,
                    color,
                    visible: true,
                    style: SeriesStyle::Line {
                        width: config.line_width,
                        dashed: matches!(line_config.line_style, LineStyle::Dashed),
                    },
                });
            }
            
            Ok(series_vec)
        } else {
            // No grouping, create a single series
            let mut sorted_points = points;
            sorted_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            
            // Set series_id for all points
            let series_id = "main".to_string();
            let color = Color32::from_rgb(31, 119, 180); // Default blue
            
            let points = sorted_points.into_iter()
                .map(|mut p| {
                    p.color = Some(color);
                    p.series_id = Some(series_id.clone());
                    p
                })
                .collect();
            
            let series = DataSeries {
                id: series_id.clone(),
                name: config.y_column.clone(),
                points,
                color,
                visible: true,
                style: SeriesStyle::Line {
                    width: config.line_width,
                    dashed: matches!(line_config.line_style, LineStyle::Dashed),
                },
            };
            
            Ok(vec![series])
        }
    }
    
    /// Convert LineStyle to line pattern
    fn get_line_pattern(&self, style: &LineStyle) -> &'static [f32] {
        match style {
            LineStyle::Solid => &[],
            LineStyle::Dashed => &[12.0, 6.0],
            LineStyle::Dotted => &[2.0, 4.0],
            LineStyle::DashDot => &[12.0, 6.0, 2.0, 6.0],
        }
    }
}

impl PlotTrait for LineChartPlot {
    fn name(&self) -> &'static str {
        "Line Chart"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Line charts typically need numeric or temporal X axis
        Some(vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float32, DataType::Float64,
            DataType::Date32, DataType::Date64,
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Millisecond, None),
        ])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        // Y axis must be numeric
        vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float16, DataType::Float32, DataType::Float64,
            DataType::Decimal128(38, 10), DataType::Decimal256(76, 10),
        ]
    }
    
    fn supports_multiple_series(&self) -> bool {
        true
    }
    
    fn get_default_config(&self) -> PlotConfiguration {
        let mut config = PlotConfiguration::default();
        config.plot_specific = PlotSpecificConfig::LineChart(LineChartConfig {
            line_style: LineStyle::Solid,
            show_points: true,
            smooth_lines: false,
            fill_area: false,
        });
        config
    }
    
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        // Process data into series
        let mut series = self.process_data(query_result, config)?;
        
        // Handle missing data gaps
        self.handle_missing_data(&mut series, config);
        
        // Calculate statistics for the data
        let statistics = if !series.is_empty() {
            let mut total_x = 0.0;
            let mut total_y = 0.0;
            let mut total_xy = 0.0;
            let mut total_x_squared = 0.0;
            let mut total_y_squared = 0.0;
            let mut count = 0;
            
            // Calculate means and correlation
            for s in &series {
                for point in &s.points {
                    total_x += point.x;
                    total_y += point.y;
                    total_xy += point.x * point.y;
                    total_x_squared += point.x * point.x;
                    total_y_squared += point.y * point.y;
                    count += 1;
                }
            }
            
            if count > 0 {
                let mean_x = total_x / count as f64;
                let mean_y = total_y / count as f64;
                
                // Calculate standard deviations
                let mut sum_squared_diff_x = 0.0;
                let mut sum_squared_diff_y = 0.0;
                
                for s in &series {
                    for point in &s.points {
                        sum_squared_diff_x += (point.x - mean_x).powi(2);
                        sum_squared_diff_y += (point.y - mean_y).powi(2);
                    }
                }
                
                let std_x = (sum_squared_diff_x / count as f64).sqrt();
                let std_y = (sum_squared_diff_y / count as f64).sqrt();
                
                // Calculate correlation coefficient
                let correlation = if std_x > 0.0 && std_y > 0.0 {
                    let numerator = total_xy - count as f64 * mean_x * mean_y;
                    let denominator = (total_x_squared - count as f64 * mean_x * mean_x) *
                                     (total_y_squared - count as f64 * mean_y * mean_y);
                    
                    if denominator > 0.0 {
                        Some(numerator / denominator.sqrt())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                Some(DataStatistics {
                    mean_x,
                    mean_y,
                    std_x,
                    std_y,
                    correlation,
                    count,
                })
            } else {
                None
            }
        } else {
            None
        };
        
        // Create plot metadata
        let metadata = super::PlotMetadata {
            title: config.title.clone(),
            x_label: config.x_column.clone(),
            y_label: config.y_column.clone(),
            show_legend: config.show_legend,
            show_grid: config.show_grid,
            color_scheme: config.color_scheme.clone(),
        };
        
        // Flatten points for backward compatibility
        let points = series.iter().flat_map(|s| s.points.clone()).collect();
        
        Ok(PlotData {
            points,
            series,
            metadata,
            statistics,
        })
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
                ui.label(RichText::new("Configure X and Y columns").weak());
            });
            return;
        }
        
        // Get line chart specific config
        let line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
            cfg
        } else {
            &LineChartConfig {
                line_style: LineStyle::Solid,
                show_points: true,
                smooth_lines: false,
                fill_area: false,
            }
        };
        
        // Create plot
        let plot = Plot::new("line_chart")
            .x_axis_label(&data.metadata.x_label)
            .y_axis_label(&data.metadata.y_label)
            .show_grid(data.metadata.show_grid)
            .allow_zoom(config.allow_zoom)
            .allow_drag(config.allow_pan)
            .allow_boxed_zoom(config.allow_zoom);
        
        // Add legend if enabled
        let plot = if data.metadata.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| {
            // Get plot bounds for filled areas
            let bounds = plot_ui.plot_bounds();
            
            // Add filled areas first (if enabled) so they appear behind the lines
            if line_config.fill_area {
                for series in &data.series {
                    if series.visible {
                        self.create_fill_area(series, plot_ui, &bounds);
                    }
                }
            }
            
            // Render each series
            for series in &data.series {
                if !series.visible {
                    continue;
                }
                
                // Get line style
                let line_width = if let SeriesStyle::Line { width, .. } = series.style {
                    width
                } else {
                    config.line_width
                };
                
                // Handle missing data gaps by splitting into segments
                let mut current_segment = Vec::new();
                let mut segments = Vec::new();
                
                for (i, point) in series.points.iter().enumerate() {
                    // Check if this is a segment boundary
                    let is_segment_end = point.z.map_or(false, |z| z < 0.0);
                    let is_segment_start = point.z.map_or(false, |z| z > 0.0);
                    
                    // If this is the end of a segment, add the point and finish the segment
                    if is_segment_end {
                        current_segment.push([point.x, point.y]);
                        segments.push(current_segment.clone());
                        current_segment.clear();
                    }
                    // If this is the start of a segment, start a new segment with this point
                    else if is_segment_start {
                        current_segment = vec![[point.x, point.y]];
                    }
                    // Otherwise, add to the current segment
                    else {
                        current_segment.push([point.x, point.y]);
                    }
                    
                    // If this is the last point and we have a non-empty segment, add it
                    if i == series.points.len() - 1 && !current_segment.is_empty() {
                        segments.push(current_segment.clone());
                    }
                }
                
                // If we didn't find any segments (no gaps), use all points as one segment
                if segments.is_empty() && !series.points.is_empty() {
                    segments.push(series.points.iter().map(|p| [p.x, p.y]).collect());
                }
                
                // Render each segment as a separate line
                for (i, segment) in segments.iter().enumerate() {
                    if segment.len() < 2 {
                        // For single points, render as a point
                        if segment.len() == 1 {
                            let points = Points::new(segment.clone())
                                .color(series.color)
                                .radius(config.marker_size * 1.2) // Slightly larger to be visible
                                .shape(EguiMarkerShape::Circle);
                            
                            plot_ui.points(points);
                        }
                        continue;
                    }
                    
                    // Create plot points for this segment
                    let plot_points = PlotPoints::new(segment.clone());
                    
                    // Create line with appropriate style
                    let mut line = Line::new(plot_points)
                        .color(series.color)
                        .width(line_width);
                    
                    // Only add name to the first segment to avoid duplicate legend entries
                    if i == 0 {
                        line = line.name(&series.name);
                    }
                    
                    // Apply line style
                    match line_config.line_style {
                        LineStyle::Solid => line = line.style(EguiLineStyle::Solid),
                        LineStyle::Dashed => line = line.style(EguiLineStyle::dashed_dense()),
                        LineStyle::Dotted => line = line.style(EguiLineStyle::dotted_dense()),
                        LineStyle::DashDot => line = line.style(EguiLineStyle::dashed_dense()),
                    }
                    
                    // Note: Curve smoothing is not available in current egui_plot version
                    // if line_config.smooth_lines && segment.len() > 2 {
                    //     line = line.curve_smoothing(0.3); // Moderate smoothing factor
                    // }
                    
                    // Add line to plot
                    plot_ui.line(line);
                }
                
                // Add points if enabled
                if line_config.show_points {
                    let points = Points::new(series.points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>())
                        .color(series.color)
                        .radius(config.marker_size)
                        .shape(EguiMarkerShape::Circle);
                    
                    plot_ui.points(points);
                }
            }
            
            // Add statistics if available
            if let Some(stats) = &data.statistics {
                if data.series.len() == 1 && !data.series[0].points.is_empty() {
                    // Add mean lines
                    let mean_x_line = Line::new(vec![[stats.mean_x, bounds.min()[1]], [stats.mean_x, bounds.max()[1]]])
                        .color(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
                        .width(1.0)
                        .style(EguiLineStyle::dashed_dense());
                    
                    let mean_y_line = Line::new(vec![[bounds.min()[0], stats.mean_y], [bounds.max()[0], stats.mean_y]])
                        .color(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
                        .width(1.0)
                        .style(EguiLineStyle::dashed_dense());
                    
                    plot_ui.line(mean_x_line);
                    plot_ui.line(mean_y_line);
                }
            }
            
            // Handle hover tooltips
            // Note: Commenting out due to borrow conflict with ui
            // if config.show_tooltips {
            //     self.handle_tooltips(ui, plot_ui, data);
            // }
        });
    }
    
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if !data.series.is_empty() {
            ui.group(|ui| {
                ui.label(RichText::new("Series:").strong());
                ui.separator();
                
                // Get line chart specific config
                let line_config = if let PlotSpecificConfig::LineChart(cfg) = &config.plot_specific {
                    cfg
                } else {
                    &LineChartConfig {
                        line_style: LineStyle::Solid,
                        show_points: true,
                        smooth_lines: false,
                        fill_area: false,
                    }
                };
                
                // Show each series with its style
                for series in &data.series {
                    let mut is_checked = series.visible;
                    if ui.checkbox(&mut is_checked, &series.name).changed() {
                        // This would require mutable access to data, which we don't have here
                        // We'll return a PlotInteraction to handle this in handle_interaction
                    }
                    
                    ui.horizontal(|ui| {
                        // Show line style
                        let line_style = match line_config.line_style {
                            LineStyle::Solid => "———",
                            LineStyle::Dashed => "---",
                            LineStyle::Dotted => "···",
                            LineStyle::DashDot => "-·-",
                        };
                        
                        ui.colored_label(series.color, line_style);
                        
                        // Show point marker if enabled
                        if line_config.show_points {
                            ui.colored_label(series.color, "●");
                        }
                        
                        ui.label(&series.name);
                    });
                }
                
                // Show statistics if available
                if let Some(stats) = &data.statistics {
                    if data.series.len() == 1 {
                        ui.separator();
                        ui.label(RichText::new("Statistics:").strong());
                        
                        if let Some(corr) = stats.correlation {
                            ui.label(format!("Correlation: {:.3}", corr));
                        }
                        
                        ui.label(format!("Points: {}", stats.count));
                    }
                }
                
                // Show line chart features
                ui.separator();
                ui.label(RichText::new("Features:").strong());
                
                if line_config.smooth_lines {
                    ui.label("• Smoothed lines");
                }
                
                if line_config.fill_area {
                    ui.label("• Area fill");
                }
                
                if config.allow_zoom {
                    ui.label("• Zoom: scroll wheel");
                }
                
                if config.allow_pan {
                    ui.label("• Pan: drag");
                }
            });
        }
    }
    
    fn handle_interaction(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) -> Option<PlotInteraction> {
        // Handle series toggling from legend clicks
        if data.metadata.show_legend && data.series.len() > 1 {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Toggle series:").strong());
                
                for series in &data.series {
                    let mut is_visible = series.visible;
                    if ui.checkbox(&mut is_visible, &series.name).changed() {
                        // Note: Can't return from closure, handle this elsewhere
                        // return Some(PlotInteraction::SeriesToggled(series.id.clone()));
                    }
                }
            });
        }
        
        // Handle zoom interaction
        if config.allow_zoom {
            ui.horizontal(|ui| {
                if ui.button("Reset Zoom").clicked() {
                    // Note: Can't return from closure, handle this elsewhere
                    // return Some(PlotInteraction::ZoomChanged(0.0, 0.0, 0.0, 0.0));
                }
            });
        }
        
        // Handle line style changes
        if let PlotSpecificConfig::LineChart(line_config) = &config.plot_specific {
            ui.horizontal(|ui| {
                ui.label("Line Style:");
                
                let mut show_points = line_config.show_points;
                if ui.checkbox(&mut show_points, "Show Points").changed() {
                    // We can't modify the config directly, so we return an interaction
                    // The parent component would need to handle this
                }
                
                let mut smooth_lines = line_config.smooth_lines;
                if ui.checkbox(&mut smooth_lines, "Smooth Lines").changed() {
                    // Same as above
                }
                
                let mut fill_area = line_config.fill_area;
                if ui.checkbox(&mut fill_area, "Fill Area").changed() {
                    // Same as above
                }
            });
        }
        
        None
    }
}