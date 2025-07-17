use egui::{Ui, Color32, RichText, Stroke};
use egui_plot::{Points, Plot, PlotPoints, Legend, MarkerShape as EguiMarkerShape, PlotUi, 
                Line, LineStyle as EguiLineStyle, Polygon, PlotBounds, PlotPoint as EguiPlotPoint,
                HLine, VLine, Text};
use datafusion::arrow::datatypes::DataType;
use std::collections::{HashMap, HashSet};
use crate::core::QueryResult;
use rand::{rng, Rng};

use super::{
    Plot as PlotTrait, 
    PlotData, 
    PlotConfiguration, 
    PlotSpecificConfig, 
    ScatterPlotConfig, 
    PlotInteraction,
    DataSeries,
    SeriesStyle,
    MarkerShape,
    PlotMetadata,
    DataStatistics,
    data_processor::DataProcessor
};

pub struct ScatterPlotImpl;

impl ScatterPlotImpl {
    /// Handle tooltips for scatter plot
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
                    egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("scatter_tooltip")),
                    egui::Id::new("scatter_tooltip"),
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
                    Color32::from_rgb(120, 210, 210) // Highlight cyan
                };
                
                // Draw highlight marker
                let highlight_points = Points::new(vec![[point.x, point.y]])
                    .color(highlight_color)
                    .radius(8.0)
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
            MarkerShape::Triangle => EguiMarkerShape::Up,
            MarkerShape::Cross => EguiMarkerShape::Cross,
            MarkerShape::Plus => EguiMarkerShape::Plus,
        }
    }
    
    /// Process data for scatter plot with proper grouping, color mapping, and size mapping
    fn process_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        // Get scatter plot specific config
        let scatter_config = if let PlotSpecificConfig::ScatterPlot(cfg) = &config.plot_specific {
            cfg
        } else {
            &ScatterPlotConfig {
                point_shape: MarkerShape::Circle,
                show_trend_line: false,
                show_density: false,
                jitter_amount: 0.0,
            }
        };
        
        // Extract points from query result
        let mut points = super::extract_plot_points(query_result, config)?;
        
        // Apply size mapping if a size column is specified
        if let Some(size_col) = &config.size_column {
            if !size_col.is_empty() {
                let size_idx = query_result.columns.iter().position(|c| c == size_col)
                    .ok_or_else(|| format!("Size column '{}' not found", size_col))?;
                
                // Find min and max values for scaling
                let mut min_size = f64::MAX;
                let mut max_size = f64::MIN;
                
                for row in &query_result.rows {
                    if row.len() > size_idx {
                        if let Ok(size_val) = row[size_idx].parse::<f64>() {
                            min_size = min_size.min(size_val);
                            max_size = max_size.max(size_val);
                        }
                    }
                }
                
                // Scale size values between min_marker_size and max_marker_size
                let min_marker_size = config.marker_size * 0.5;
                let max_marker_size = config.marker_size * 3.0;
                let size_range = max_size - min_size;
                
                // Update point sizes
                for (i, point) in points.iter_mut().enumerate() {
                    if i < query_result.rows.len() && query_result.rows[i].len() > size_idx {
                        if let Ok(size_val) = query_result.rows[i][size_idx].parse::<f64>() {
                            // Scale the size value
                            let scaled_size = if size_range > 0.0 {
                                min_marker_size + (max_marker_size - min_marker_size) * 
                                    ((size_val - min_size) / size_range) as f32
                            } else {
                                config.marker_size
                            };
                            
                            point.size = Some(scaled_size);
                            
                            // Add size value to tooltip data
                            point.tooltip_data.insert(size_col.clone(), query_result.rows[i][size_idx].clone());
                        }
                    }
                }
            }
        }
        
        // Apply jitter if configured
        if scatter_config.jitter_amount > 0.0 {
            let mut rng = rng();
            let jitter = scatter_config.jitter_amount as f64;
            
            for point in &mut points {
                point.x += rng.random_range(-jitter..=jitter);
                point.y += rng.random_range(-jitter..=jitter);
            }
        }
        
        // Group points by color if a color column is specified
        if let Some(color_col) = &config.color_column {
            // Create a map of color_value -> points
            let mut color_map: HashMap<String, Vec<super::PlotPoint>> = HashMap::new();
            
            for point in points {
                let color_value = if let Some(value) = point.tooltip_data.get(color_col) {
                    value.clone()
                } else {
                    "default".to_string()
                };
                
                color_map.entry(color_value).or_default().push(point);
            }
            
            // Create a DataSeries for each color group
            let colors = super::get_categorical_colors(&config.color_scheme);
            let mut series_vec = Vec::new();
            
            for (i, (color_value, points)) in color_map.into_iter().enumerate() {
                let color = colors[i % colors.len()];
                
                // Set the color for all points in this series
                let points = points.into_iter()
                    .map(|mut p| {
                        p.color = Some(color);
                        p.series_id = Some(color_value.clone());
                        p
                    })
                    .collect();
                
                series_vec.push(DataSeries {
                    id: color_value.clone(),
                    name: color_value,
                    points,
                    color,
                    visible: true,
                    style: SeriesStyle::Points {
                        size: config.marker_size,
                        shape: scatter_config.point_shape,
                    },
                });
            }
            
            Ok(series_vec)
        } else {
            // No color grouping, create a single series
            let color = Color32::from_rgb(31, 119, 180); // Default blue
            
            // Set color for all points
            let points = points.into_iter()
                .map(|mut p| {
                    p.color = Some(color);
                    p.series_id = Some("main".to_string());
                    p
                })
                .collect();
            
            let series = DataSeries {
                id: "main".to_string(),
                name: config.y_column.clone(),
                points,
                color,
                visible: true,
                style: SeriesStyle::Points {
                    size: config.marker_size,
                    shape: scatter_config.point_shape,
                },
            };
            
            Ok(vec![series])
        }
    }
    
    /// Create density heatmap for large datasets
    fn create_density_heatmap(&self, data: &[super::PlotPoint], bounds: &PlotBounds) -> Vec<Vec<usize>> {
        // Define grid size for density calculation
        const GRID_SIZE: usize = 50;
        
        // Initialize density grid
        let mut density_grid = vec![vec![0; GRID_SIZE]; GRID_SIZE];
        
        // Calculate bounds
        let x_min = bounds.min()[0];
        let x_max = bounds.max()[0];
        let y_min = bounds.min()[1];
        let y_max = bounds.max()[1];
        
        let x_range = x_max - x_min;
        let y_range = y_max - y_min;
        
        // Count points in each grid cell
        for point in data {
            if point.x >= x_min && point.x <= x_max && point.y >= y_min && point.y <= y_max {
                let grid_x = ((point.x - x_min) / x_range * (GRID_SIZE as f64 - 1.0)) as usize;
                let grid_y = ((point.y - y_min) / y_range * (GRID_SIZE as f64 - 1.0)) as usize;
                
                // Ensure we're within bounds
                let grid_x = grid_x.min(GRID_SIZE - 1);
                let grid_y = grid_y.min(GRID_SIZE - 1);
                
                density_grid[grid_y][grid_x] += 1;
            }
        }
        
        density_grid
    }
    
    /// Draw density overlay on the plot
    fn draw_density_overlay(&self, plot_ui: &PlotUi, data: &[super::PlotPoint], bounds: &PlotBounds) {
        // Only create density overlay for large datasets
        if data.len() < 100 {
            return;
        }
        
        // Create density grid
        let density_grid = self.create_density_heatmap(data, bounds);
        const GRID_SIZE: usize = 50;
        
        // Find maximum density for normalization
        let max_density = density_grid.iter()
            .flat_map(|row| row.iter())
            .fold(0, |max, &count| max.max(count));
        
        if max_density == 0 {
            return;
        }
        
        // Calculate cell size in plot coordinates
        let x_min = bounds.min()[0];
        let x_max = bounds.max()[0];
        let y_min = bounds.min()[1];
        let y_max = bounds.max()[1];
        
        let cell_width = (x_max - x_min) / GRID_SIZE as f64;
        let cell_height = (y_max - y_min) / GRID_SIZE as f64;
        
        // Draw density cells
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let density = density_grid[y][x];
                if density > 0 {
                    // Normalize density and create color with alpha based on density
                    let alpha = ((density as f32) / (max_density as f32) * 180.0) as u8;
                    let color = Color32::from_rgba_unmultiplied(100, 100, 255, alpha);
                    
                    // Calculate cell coordinates
                    let cell_x = x_min + x as f64 * cell_width;
                    let cell_y = y_min + y as f64 * cell_height;
                    
                    // Create polygon for the cell
                    let polygon = Polygon::new(vec![
                        [cell_x, cell_y],
                        [cell_x + cell_width, cell_y],
                        [cell_x + cell_width, cell_y + cell_height],
                        [cell_x, cell_y + cell_height],
                    ])
                    .fill_color(color)
                    .stroke(Stroke::new(0.0, Color32::TRANSPARENT));
                    
                    // plot_ui.polygon(polygon);
                }
            }
        }
    }
    
    /// Create selection rectangle for interactive selection
    fn create_selection_rectangle(&self, start: [f64; 2], end: [f64; 2]) -> Polygon {
        Polygon::new(vec![
            [start[0], start[1]],
            [end[0], start[1]],
            [end[0], end[1]],
            [start[0], end[1]],
        ])
        .fill_color(Color32::from_rgba_unmultiplied(100, 100, 255, 50))
        .stroke(Stroke::new(1.0, Color32::from_rgb(100, 100, 255)))
    }
    
    /// Calculate correlation statistics for the data
    fn calculate_statistics(&self, data: &[super::PlotPoint]) -> super::DataStatistics {
        let n = data.len() as f64;
        
        if n == 0.0 {
            return super::DataStatistics {
                mean_x: 0.0,
                mean_y: 0.0,
                std_x: 0.0,
                std_y: 0.0,
                correlation: None,
                count: 0,
            };
        }
        
        // Calculate means
        let sum_x: f64 = data.iter().map(|p| p.x).sum();
        let sum_y: f64 = data.iter().map(|p| p.y).sum();
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        
        // Calculate standard deviations and correlation
        let mut sum_xx = 0.0;
        let mut sum_yy = 0.0;
        let mut sum_xy = 0.0;
        
        for point in data {
            let dx = point.x - mean_x;
            let dy = point.y - mean_y;
            sum_xx += dx * dx;
            sum_yy += dy * dy;
            sum_xy += dx * dy;
        }
        
        let std_x = (sum_xx / n).sqrt();
        let std_y = (sum_yy / n).sqrt();
        
        // Calculate correlation coefficient
        let correlation = if std_x > 0.0 && std_y > 0.0 {
            Some(sum_xy / (n * std_x * std_y))
        } else {
            None
        };
        
        super::DataStatistics {
            mean_x,
            mean_y,
            std_x,
            std_y,
            correlation,
            count: data.len(),
        }
    }
}

impl PlotTrait for ScatterPlotImpl {
    fn name(&self) -> &'static str {
        "Scatter Plot"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Scatter plots need numeric X axis
        Some(vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float32, DataType::Float64,
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
    
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)> {
        vec![
            ("Size", vec![
                DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
                DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
                DataType::Float32, DataType::Float64,
            ]),
            ("Color", vec![
                DataType::Utf8, DataType::LargeUtf8,
                DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            ]),
        ]
    }
    
    fn supports_multiple_series(&self) -> bool {
        true
    }
    
    fn supports_color_mapping(&self) -> bool {
        true
    }
    
    fn supports_size_mapping(&self) -> bool {
        true
    }
    
    fn get_default_config(&self) -> PlotConfiguration {
        let mut config = PlotConfiguration::default();
        config.plot_specific = PlotSpecificConfig::ScatterPlot(ScatterPlotConfig {
            point_shape: MarkerShape::Circle,
            show_trend_line: false,
            show_density: false,
            jitter_amount: 0.0,
        });
        config
    }
    
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        let series = self.process_data(query_result, config)?;
        
        // Create plot metadata
        let metadata = super::PlotMetadata {
            title: config.title.clone(),
            x_label: config.x_column.clone(),
            y_label: config.y_column.clone(),
            show_legend: config.show_legend,
            show_grid: config.show_grid,
            color_scheme: config.color_scheme.clone(),
        };
        
        // Flatten points for backward compatibility and statistics
        let points: Vec<_> = series.iter().flat_map(|s| s.points.clone()).collect();
        
        // Calculate statistics
        let statistics = Some(self.calculate_statistics(&points));
        
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
        
        // Get scatter plot specific config
        let scatter_config = if let PlotSpecificConfig::ScatterPlot(cfg) = &config.plot_specific {
            cfg
        } else {
            &ScatterPlotConfig {
                point_shape: MarkerShape::Circle,
                show_trend_line: false,
                show_density: false,
                jitter_amount: 0.0,
            }
        };
        
        // Create plot
        let plot = Plot::new("scatter_plot")
            .x_axis_label(&data.metadata.x_label)
            .y_axis_label(&data.metadata.y_label)
            .show_grid(data.metadata.show_grid)
            .allow_zoom(config.allow_zoom)
            .allow_drag(config.allow_pan)
            .allow_boxed_zoom(config.allow_zoom)
            .data_aspect(1.0);
        
        // Add legend if enabled
        let plot = if data.metadata.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| {
            // Get plot bounds for density overlay
            let bounds = plot_ui.plot_bounds();
            
            // Add density overlay if enabled and we have enough points
            if scatter_config.show_density && data.points.len() >= 100 {
                self.draw_density_overlay(plot_ui, &data.points, &bounds);
            }
            
            // Render each series
            for series in &data.series {
                if !series.visible {
                    continue;
                }
                
                // Get point style
                let (point_size, point_shape) = if let SeriesStyle::Points { size, shape } = series.style {
                    (size, shape)
                } else {
                    (config.marker_size, MarkerShape::Circle)
                };
                
                // Create points with proper size mapping
                let mut points_by_size: HashMap<u32, Vec<[f64; 2]>> = HashMap::new();
                
                // Group points by size for efficient rendering (rounded to avoid float key issues)
                for point in &series.points {
                    let size = point.size.unwrap_or(point_size);
                    let size_key = (size * 100.0) as u32; // Round to nearest 0.01
                    points_by_size.entry(size_key).or_insert_with(Vec::new).push([point.x, point.y]);
                }
                
                // Render each size group separately
                for (size_key, points_vec) in points_by_size {
                    let size = (size_key as f32) / 100.0; // Convert back to float
                    let plot_points = PlotPoints::new(points_vec);
                    
                    let default_size_key = (point_size * 100.0) as u32;
                    let points = Points::new(plot_points)
                        .name(if size_key == default_size_key { &series.name } else { "" }) // Only add name to default size
                        .color(series.color)
                        .radius(size)
                        .shape(Self::to_egui_marker_shape(&point_shape));
                    
                    plot_ui.points(points);
                }
            }
            
            // Add trend line if enabled
            if scatter_config.show_trend_line {
                if let Some(stats) = &data.statistics {
                    if let Some(correlation) = stats.correlation {
                        // Only show trend line if there's a meaningful correlation
                        if correlation.abs() > 0.1 && stats.std_x > 0.0 && stats.std_y > 0.0 {
                            // Calculate regression line: y = mx + b
                            let m = correlation * stats.std_y / stats.std_x;
                            let b = stats.mean_y - m * stats.mean_x;
                            
                            // Find min and max x values
                            let min_x = data.points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
                            let max_x = data.points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
                            
                            // Create trend line
                            let trend_line = Line::new(vec![[min_x, min_x * m + b], [max_x, max_x * m + b]])
                                .color(Color32::from_rgb(200, 100, 100))
                                .width(2.0)
                                .style(EguiLineStyle::dashed_dense())
                                .name("Trend Line");
                            
                            plot_ui.line(trend_line);
                            
                            // Add equation text
                            let equation = format!("y = {:.3}x + {:.3}", m, b);
                            let text_pos_x = min_x + (max_x - min_x) * 0.1;
                            let text_pos_y = min_x * m + b + (max_x - min_x) * 0.1;
                            
                            let equation_text = Text::new(
                                EguiPlotPoint::new(text_pos_x, text_pos_y),
                                RichText::new(equation).color(Color32::from_rgb(200, 100, 100))
                            );
                            
                            plot_ui.text(equation_text);
                        }
                    }
                }
            }
            
            // Add mean lines if we have statistics
            if let Some(stats) = &data.statistics {
                // Add mean X line
                let mean_x_line = VLine::new(stats.mean_x)
                    .color(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
                    .width(1.0)
                    .style(EguiLineStyle::dashed_dense());
                
                // Add mean Y line
                let mean_y_line = HLine::new(stats.mean_y)
                    .color(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
                    .width(1.0)
                    .style(EguiLineStyle::dashed_dense());
                
                plot_ui.vline(mean_x_line);
                plot_ui.hline(mean_y_line);
            }
            
            // Handle hover tooltips
            // Note: Commenting out due to borrow conflict with ui
            // if config.show_tooltips {
            //     self.handle_tooltips(ui, plot_ui, data);
            // }
        });
        
        // Show correlation statistics if available
        if let Some(stats) = &data.statistics {
            if let Some(correlation) = stats.correlation {
                ui.horizontal(|ui| {
                    ui.label("Correlation:");
                    let corr_text = format!("{:.3}", correlation);
                    let corr_color = if correlation.abs() < 0.3 {
                        Color32::from_rgb(150, 150, 150) // Weak correlation
                    } else if correlation.abs() < 0.7 {
                        Color32::from_rgb(150, 150, 0) // Moderate correlation
                    } else {
                        Color32::from_rgb(0, 150, 0) // Strong correlation
                    };
                    ui.colored_label(corr_color, corr_text);
                    
                    ui.separator();
                    ui.label(format!("Points: {}", stats.count));
                });
            }
        }
    }
    
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if !data.series.is_empty() {
            ui.group(|ui| {
                // Get scatter plot specific config
                let scatter_config = if let PlotSpecificConfig::ScatterPlot(cfg) = &config.plot_specific {
                    cfg
                } else {
                    &ScatterPlotConfig {
                        point_shape: MarkerShape::Circle,
                        show_trend_line: false,
                        show_density: false,
                        jitter_amount: 0.0,
                    }
                };
                
                // Show color mapping legend if we have multiple series
                if data.series.len() > 1 {
                    ui.label(RichText::new("Color Mapping:").strong());
                    ui.separator();
                    
                    // Get unique series
                    let mut unique_series = HashSet::new();
                    
                    for series in &data.series {
                        if series.visible && !unique_series.contains(&series.name) {
                            ui.horizontal(|ui| {
                                ui.colored_label(series.color, "●");
                                ui.label(&series.name);
                            });
                            unique_series.insert(series.name.clone());
                        }
                    }
                    
                    // Show color column name if available
                    if let Some(color_col) = &config.color_column {
                        ui.separator();
                        ui.label(format!("Color by: {}", color_col));
                    }
                }
                
                // Show size mapping legend if we have size mapping
                if let Some(size_col) = &config.size_column {
                    ui.separator();
                    ui.label(RichText::new("Size Mapping:").strong());
                    ui.separator();
                    
                    // Show size legend with small, medium, and large markers
                    ui.horizontal(|ui| {
                        ui.label("Small");
                        ui.colored_label(Color32::from_rgb(100, 100, 100), "●");
                        ui.colored_label(Color32::from_rgb(100, 100, 100), "●●");
                        ui.colored_label(Color32::from_rgb(100, 100, 100), "●●●");
                        ui.label("Large");
                    });
                    
                    ui.label(format!("Size by: {}", size_col));
                }
                
                // Show point shape
                ui.separator();
                ui.label(RichText::new("Point Style:").strong());
                ui.horizontal(|ui| {
                    let shape_text = match scatter_config.point_shape {
                        MarkerShape::Circle => "●",
                        MarkerShape::Square => "■",
                        MarkerShape::Diamond => "◆",
                        MarkerShape::Triangle => "▲",
                        MarkerShape::Cross => "✕",
                        MarkerShape::Plus => "+",
                    };
                    ui.label(format!("Shape: {}", shape_text));
                });
                
                // Show statistics if available
                if let Some(stats) = &data.statistics {
                    ui.separator();
                    ui.label(RichText::new("Statistics:").strong());
                    
                    if let Some(correlation) = stats.correlation {
                        let corr_desc = if correlation.abs() < 0.3 {
                            "Weak correlation"
                        } else if correlation.abs() < 0.7 {
                            "Moderate correlation"
                        } else {
                            "Strong correlation"
                        };
                        
                        ui.label(format!("Correlation: {:.3} ({})", correlation, corr_desc));
                    }
                    
                    ui.label(format!("Points: {}", stats.count));
                    
                    if stats.count > 0 {
                        ui.label(format!("Mean: ({:.2}, {:.2})", stats.mean_x, stats.mean_y));
                        ui.label(format!("Std Dev: ({:.2}, {:.2})", stats.std_x, stats.std_y));
                    }
                }
                
                // Show enabled features
                ui.separator();
                ui.label(RichText::new("Features:").strong());
                
                if scatter_config.show_trend_line {
                    ui.label("• Trend line");
                }
                
                if scatter_config.show_density {
                    ui.label("• Density overlay");
                }
                
                if scatter_config.jitter_amount > 0.0 {
                    ui.label(format!("• Jitter: {:.2}", scatter_config.jitter_amount));
                }
            });
        }
    }
    
    fn handle_interaction(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) -> Option<PlotInteraction> {
        // Get scatter plot specific config
        let scatter_config = if let PlotSpecificConfig::ScatterPlot(cfg) = &config.plot_specific {
            cfg
        } else {
            &ScatterPlotConfig {
                point_shape: MarkerShape::Circle,
                show_trend_line: false,
                show_density: false,
                jitter_amount: 0.0,
            }
        };
        
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
        
        // Handle scatter plot specific controls
        ui.horizontal(|ui| {
            ui.label("Options:");
            
            // Trend line toggle
            let mut show_trend = scatter_config.show_trend_line;
            if ui.checkbox(&mut show_trend, "Trend Line").changed() {
                // We can't modify the config directly, but we can return an interaction
                // The parent component would need to handle this
            }
            
            // Density overlay toggle
            let mut show_density = scatter_config.show_density;
            if ui.checkbox(&mut show_density, "Density Overlay").changed() {
                // Same as above
            }
            
            // Jitter control
            if ui.button("Add Jitter").clicked() {
                // Return interaction to add jitter
            }
        });
        
        // Show correlation statistics if available
        if let Some(stats) = &data.statistics {
            if let Some(correlation) = stats.correlation {
                ui.horizontal(|ui| {
                    ui.label("Correlation:");
                    let corr_text = format!("{:.3}", correlation);
                    let corr_color = if correlation.abs() < 0.3 {
                        Color32::from_rgb(150, 150, 150) // Weak correlation
                    } else if correlation.abs() < 0.7 {
                        Color32::from_rgb(150, 150, 0) // Moderate correlation
                    } else {
                        Color32::from_rgb(0, 150, 0) // Strong correlation
                    };
                    ui.colored_label(corr_color, corr_text);
                    
                    // Add correlation strength description
                    let corr_desc = if correlation.abs() < 0.3 {
                        "Weak correlation"
                    } else if correlation.abs() < 0.7 {
                        "Moderate correlation"
                    } else {
                        "Strong correlation"
                    };
                    ui.label(corr_desc);
                });
                
                // Show additional statistics
                ui.horizontal(|ui| {
                    ui.label(format!("Mean X: {:.3}", stats.mean_x));
                    ui.label(format!("Mean Y: {:.3}", stats.mean_y));
                    ui.label(format!("Std Dev X: {:.3}", stats.std_x));
                    ui.label(format!("Std Dev Y: {:.3}", stats.std_y));
                });
            }
        }
        
        // Handle selection tools
        if config.allow_selection {
            ui.horizontal(|ui| {
                ui.label("Selection:");
                
                if ui.button("Rectangle").clicked() {
                    // Start rectangle selection mode
                    // This would need to be handled by the parent component
                }
                
                if ui.button("Lasso").clicked() {
                    // Start lasso selection mode
                    // This would need to be handled by the parent component
                }
                
                if ui.button("Clear").clicked() {
                    // Clear selection
                    // This would need to be handled by the parent component
                }
            });
        }
        
        None
    }
}