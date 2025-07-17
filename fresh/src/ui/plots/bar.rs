use egui::{Ui, Color32, RichText};
use egui_plot::{Bar, BarChart, Plot, Legend, PlotUi, PlotPoint as EguiPlotPoint};
use datafusion::arrow::datatypes::DataType;
use std::collections::HashMap;
use crate::core::QueryResult;

use super::{
    Plot as PlotTrait, 
    PlotData, 
    PlotConfiguration, 
    PlotSpecificConfig, 
    BarChartConfig, 
    StackingMode, 
    SortOrder, 
    PlotInteraction,
    DataSeries,
    SeriesStyle,
    PlotMetadata,
    data_processor::DataProcessor
};

pub struct BarChartPlot;

impl BarChartPlot {
    /// Process data for grouped bar charts
    async fn process_grouped_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        let data_processor = DataProcessor::new();
        
        // Ensure we have a group column
        if config.group_column.is_none() || config.group_column.as_ref().unwrap().is_empty() {
            return Err("Group column is required for grouped bar charts".to_string());
        }
        
        let group_col = config.group_column.as_ref().unwrap();
        
        // Get unique categories from X column
        let categories = self.get_unique_categories(query_result, &config.x_column)?;
        
        // Get unique groups from group column
        let groups = self.get_unique_categories(query_result, group_col)?;
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Create a series for each group
        let mut all_series = Vec::new();
        let colors = super::get_categorical_colors(&config.color_scheme);
        
        for (group_idx, group) in groups.iter().enumerate() {
            let color = colors[group_idx % colors.len()];
            let mut points = Vec::new();
            
            // For each category, get the value for this group
            for (cat_idx, category) in categories.iter().enumerate() {
                // Filter data for this category and group
                let filtered_rows: Vec<&Vec<String>> = query_result.rows.iter()
                    .filter(|row| {
                        let x_idx = query_result.columns.iter().position(|c| c == &config.x_column).unwrap_or(0);
                        let group_idx = query_result.columns.iter().position(|c| c == group_col).unwrap_or(0);
                        
                        row.len() > x_idx && row.len() > group_idx && 
                        row[x_idx] == *category && row[group_idx] == *group
                    })
                    .collect();
                
                // Aggregate values
                let y_idx = query_result.columns.iter().position(|c| c == &config.y_column).unwrap_or(0);
                let sum: f64 = filtered_rows.iter()
                    .filter_map(|row| {
                        if row.len() > y_idx {
                            row[y_idx].parse::<f64>().ok()
                        } else {
                            None
                        }
                    })
                    .sum();
                
                // Calculate x position with offset for grouped bars
                let group_count = groups.len() as f64;
                let bar_width = bar_config.bar_width as f64;
                let group_spacing = bar_config.group_spacing as f64;
                let total_width = group_count * bar_width + (group_count - 1.0) * group_spacing;
                let start_x = cat_idx as f64 - total_width / 2.0 + bar_width / 2.0;
                let x = start_x + (group_idx as f64) * (bar_width + group_spacing);
                
                // Create tooltip data
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("Category".to_string(), category.clone());
                tooltip_data.insert("Group".to_string(), group.clone());
                tooltip_data.insert("Value".to_string(), format!("{:.2}", sum));
                
                points.push(super::PlotPoint {
                    x,
                    y: sum,
                    z: None,
                    label: Some(category.clone()),
                    color: Some(color),
                    size: None,
                    series_id: Some(group.clone()),
                    tooltip_data,
                });
            }
            
            // Create series for this group
            let series = DataSeries {
                id: group.clone(),
                name: group.clone(),
                points,
                color,
                visible: true,
                style: SeriesStyle::Bars { width: bar_config.bar_width },
            };
            
            all_series.push(series);
        }
        
        Ok(all_series)
    }
    
    /// Process data for stacked bar charts
    async fn process_stacked_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        let data_processor = DataProcessor::new();
        
        // Ensure we have a group column
        if config.group_column.is_none() || config.group_column.as_ref().unwrap().is_empty() {
            return Err("Group column is required for stacked bar charts".to_string());
        }
        
        let group_col = config.group_column.as_ref().unwrap();
        
        // Get unique categories from X column
        let categories = self.get_unique_categories(query_result, &config.x_column)?;
        
        // Get unique groups from group column
        let groups = self.get_unique_categories(query_result, group_col)?;
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Create a series for each group
        let mut all_series = Vec::new();
        let colors = super::get_categorical_colors(&config.color_scheme);
        
        // For stacked bars, we need to track the cumulative height for each category
        let mut cumulative_heights: HashMap<String, f64> = HashMap::new();
        for category in &categories {
            cumulative_heights.insert(category.clone(), 0.0);
        }
        
        // Process each group
        for (group_idx, group) in groups.iter().enumerate() {
            let color = colors[group_idx % colors.len()];
            let mut points = Vec::new();
            
            // For each category, get the value for this group
            for (cat_idx, category) in categories.iter().enumerate() {
                // Filter data for this category and group
                let filtered_rows: Vec<&Vec<String>> = query_result.rows.iter()
                    .filter(|row| {
                        let x_idx = query_result.columns.iter().position(|c| c == &config.x_column).unwrap_or(0);
                        let group_idx = query_result.columns.iter().position(|c| c == group_col).unwrap_or(0);
                        
                        row.len() > x_idx && row.len() > group_idx && 
                        row[x_idx] == *category && row[group_idx] == *group
                    })
                    .collect();
                
                // Aggregate values
                let y_idx = query_result.columns.iter().position(|c| c == &config.y_column).unwrap_or(0);
                let sum: f64 = filtered_rows.iter()
                    .filter_map(|row| {
                        if row.len() > y_idx {
                            row[y_idx].parse::<f64>().ok()
                        } else {
                            None
                        }
                    })
                    .sum();
                
                // Get current cumulative height for this category
                let base_height = *cumulative_heights.get(category).unwrap_or(&0.0);
                let total_height = base_height + sum;
                
                // Create tooltip data
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("Category".to_string(), category.clone());
                tooltip_data.insert("Group".to_string(), group.clone());
                tooltip_data.insert("Value".to_string(), format!("{:.2}", sum));
                tooltip_data.insert("Total".to_string(), format!("{:.2}", total_height));
                
                points.push(super::PlotPoint {
                    x: cat_idx as f64,
                    y: sum,
                    z: Some(base_height), // Use z to store the base height for stacking
                    label: Some(category.clone()),
                    color: Some(color),
                    size: None,
                    series_id: Some(group.clone()),
                    tooltip_data,
                });
                
                // Update cumulative height
                cumulative_heights.insert(category.clone(), total_height);
            }
            
            // Create series for this group
            let series = DataSeries {
                id: group.clone(),
                name: group.clone(),
                points,
                color,
                visible: true,
                style: SeriesStyle::Bars { width: bar_config.bar_width },
            };
            
            all_series.push(series);
        }
        
        Ok(all_series)
    }
    
    /// Process data for percent stacked bar charts
    async fn process_percent_stacked_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        let data_processor = DataProcessor::new();
        
        // Ensure we have a group column
        if config.group_column.is_none() || config.group_column.as_ref().unwrap().is_empty() {
            return Err("Group column is required for percent stacked bar charts".to_string());
        }
        
        let group_col = config.group_column.as_ref().unwrap();
        
        // Get unique categories from X column
        let categories = self.get_unique_categories(query_result, &config.x_column)?;
        
        // Get unique groups from group column
        let groups = self.get_unique_categories(query_result, group_col)?;
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Calculate total values for each category for percentage calculation
        let mut category_totals: HashMap<String, f64> = HashMap::new();
        
        for category in &categories {
            let y_idx = query_result.columns.iter().position(|c| c == &config.y_column).unwrap_or(0);
            let total: f64 = query_result.rows.iter()
                .filter(|row| {
                    let x_idx = query_result.columns.iter().position(|c| c == &config.x_column).unwrap_or(0);
                    row.len() > x_idx && row[x_idx] == *category
                })
                .filter_map(|row| {
                    if row.len() > y_idx {
                        row[y_idx].parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .sum();
            
            category_totals.insert(category.clone(), total);
        }
        
        // Create a series for each group
        let mut all_series = Vec::new();
        let colors = super::get_categorical_colors(&config.color_scheme);
        
        // For stacked bars, we need to track the cumulative percentage for each category
        let mut cumulative_percents: HashMap<String, f64> = HashMap::new();
        for category in &categories {
            cumulative_percents.insert(category.clone(), 0.0);
        }
        
        // Process each group
        for (group_idx, group) in groups.iter().enumerate() {
            let color = colors[group_idx % colors.len()];
            let mut points = Vec::new();
            
            // For each category, get the value for this group
            for (cat_idx, category) in categories.iter().enumerate() {
                // Filter data for this category and group
                let filtered_rows: Vec<&Vec<String>> = query_result.rows.iter()
                    .filter(|row| {
                        let x_idx = query_result.columns.iter().position(|c| c == &config.x_column).unwrap_or(0);
                        let group_idx = query_result.columns.iter().position(|c| c == group_col).unwrap_or(0);
                        
                        row.len() > x_idx && row.len() > group_idx && 
                        row[x_idx] == *category && row[group_idx] == *group
                    })
                    .collect();
                
                // Aggregate values
                let y_idx = query_result.columns.iter().position(|c| c == &config.y_column).unwrap_or(0);
                let sum: f64 = filtered_rows.iter()
                    .filter_map(|row| {
                        if row.len() > y_idx {
                            row[y_idx].parse::<f64>().ok()
                        } else {
                            None
                        }
                    })
                    .sum();
                
                // Calculate percentage
                let total = *category_totals.get(category).unwrap_or(&1.0);
                let percentage = if total > 0.0 { (sum / total) * 100.0 } else { 0.0 };
                
                // Get current cumulative percentage for this category
                let base_percent = *cumulative_percents.get(category).unwrap_or(&0.0);
                let total_percent = base_percent + percentage;
                
                // Create tooltip data
                let mut tooltip_data = HashMap::new();
                tooltip_data.insert("Category".to_string(), category.clone());
                tooltip_data.insert("Group".to_string(), group.clone());
                tooltip_data.insert("Value".to_string(), format!("{:.2}", sum));
                tooltip_data.insert("Percentage".to_string(), format!("{:.1}%", percentage));
                
                points.push(super::PlotPoint {
                    x: cat_idx as f64,
                    y: percentage,
                    z: Some(base_percent), // Use z to store the base percentage for stacking
                    label: Some(category.clone()),
                    color: Some(color),
                    size: None,
                    series_id: Some(group.clone()),
                    tooltip_data,
                });
                
                // Update cumulative percentage
                cumulative_percents.insert(category.clone(), total_percent);
            }
            
            // Create series for this group
            let series = DataSeries {
                id: group.clone(),
                name: group.clone(),
                points,
                color,
                visible: true,
                style: SeriesStyle::Bars { width: bar_config.bar_width },
            };
            
            all_series.push(series);
        }
        
        Ok(all_series)
    }
    
    /// Helper method to get unique categories from a column
    fn get_unique_categories(&self, query_result: &QueryResult, column: &str) -> Result<Vec<String>, String> {
        let col_idx = query_result.columns.iter().position(|c| c == column)
            .ok_or_else(|| format!("Column '{}' not found", column))?;
        
        let mut categories = std::collections::HashSet::new();
        for row in &query_result.rows {
            if row.len() > col_idx {
                categories.insert(row[col_idx].clone());
            }
        }
        
        Ok(categories.into_iter().collect())
    }
    
    /// Handle tooltips for bar chart
    fn handle_tooltips(&self, plot_ui: &PlotUi, data: &PlotData) {
        if let Some(pointer_coord) = plot_ui.pointer_coordinate() {
            // Get bar chart specific config from the first series
            let bar_width = if let Some(series) = data.series.first() {
                if let SeriesStyle::Bars { width } = series.style {
                    width as f64
                } else {
                    0.7 // Default width
                }
            } else {
                0.7 // Default width
            };
            
            // Check if we're using stacked bars by looking for z values
            let is_stacked = data.points.iter().any(|p| p.z.is_some());
            
            // Find the bar under the cursor
            for point in &data.points {
                // Check if pointer is over this bar
                let bar_x = point.x;
                
                // For stacked bars, we need to check if the pointer is within the bar's vertical segment
                let y_min = if is_stacked {
                    point.z.unwrap_or(0.0)
                } else {
                    0.0
                };
                
                let y_max = if is_stacked {
                    point.z.unwrap_or(0.0) + point.y
                } else {
                    point.y
                };
                
                // Check if pointer is within bar boundaries
                if pointer_coord.x >= bar_x - bar_width/2.0 && 
                   pointer_coord.x <= bar_x + bar_width/2.0 && 
                   ((pointer_coord.y >= y_min && pointer_coord.y <= y_max) || 
                    (pointer_coord.y <= y_min && pointer_coord.y >= y_max)) {
                    
                    // Show tooltip with bar data
                    let mut tooltip_text = String::new();
                    
                    // Add category/label
                    if let Some(label) = &point.label {
                        tooltip_text.push_str(&format!("Category: {}\n", label));
                    }
                    
                    // Add series name if available
                    if let Some(series_id) = &point.series_id {
                        if let Some(series) = data.series.iter().find(|s| &s.id == series_id) {
                            tooltip_text.push_str(&format!("Series: {}\n", series.name));
                        }
                    }
                    
                    // Add value
                    if is_stacked {
                        tooltip_text.push_str(&format!("Value: {:.2}\n", point.y));
                    } else {
                        tooltip_text.push_str(&format!("Value: {:.2}", point.y));
                    }
                    
                    // Add any additional tooltip data
                    for (key, value) in &point.tooltip_data {
                        if key != "Category" && key != "Value" && key != "Series" {
                            tooltip_text.push_str(&format!("\n{}: {}", key, value));
                        }
                    }
                    
                    // plot_ui.show_tooltip(|ui| {
                    //     ui.label(tooltip_text);
                    // });
                    
                    // Highlight the bar
                    let highlight_color = if let Some(color) = point.color {
                        // Make the color brighter for highlighting
                        Color32::from_rgb(
                            (color.r() as u16 + 40).min(255) as u8,
                            (color.g() as u16 + 40).min(255) as u8,
                            (color.b() as u16 + 40).min(255) as u8,
                        )
                    } else {
                        Color32::from_rgb(120, 210, 120) // Highlight green
                    };
                    
                    // Create highlight bar with proper stacking if needed
                    let highlight_bar = if is_stacked {
                        Bar::new(point.x, point.y)
                            .width(bar_width)
                            .base_offset(point.z.unwrap_or(0.0))
                            .fill(highlight_color)
                    } else {
                        Bar::new(point.x, point.y)
                            .width(bar_width)
                            .fill(highlight_color)
                    };
                    
                    // plot_ui.bar(highlight_bar);
                    
                    break; // Only show tooltip for one bar at a time
                }
            }
        }
    }

    /// Process data for bar chart with proper aggregation
    async fn process_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<Vec<DataSeries>, String> {
        let data_processor = DataProcessor::new();
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Aggregate data using DataFusion
        let aggregated_data = data_processor.aggregate_for_bar_chart(
            query_result, 
            &config.x_column, 
            &config.y_column
        ).await?;
        
        // Sort data if needed
        let mut sorted_data = aggregated_data;
        match bar_config.sort_order {
            SortOrder::Ascending => {
                sorted_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            },
            SortOrder::Descending => {
                sorted_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            },
            SortOrder::ByValue => {
                sorted_data.sort_by(|a, b| a.0.cmp(&b.0));
            },
            SortOrder::None => {}
        }
        
        // Create a single data series for now
        // (Multi-series support will be added when implementing grouped bars)
        let mut points = Vec::new();
        let colors = super::get_categorical_colors(&config.color_scheme);
        
        for (i, (category, value)) in sorted_data.iter().enumerate() {
            let color = colors[i % colors.len()];
            
            // Create tooltip data
            let mut tooltip_data = HashMap::new();
            tooltip_data.insert("Category".to_string(), category.clone());
            tooltip_data.insert("Value".to_string(), format!("{:.2}", value));
            
            points.push(super::PlotPoint {
                x: i as f64,
                y: *value,
                z: None,
                label: Some(category.clone()),
                color: Some(color),
                size: None,
                series_id: Some("main".to_string()),
                tooltip_data,
            });
        }
        
        let series = DataSeries {
            id: "main".to_string(),
            name: config.y_column.clone(),
            points,
            color: Color32::from_rgb(92, 140, 97), // Default bar color
            visible: true,
            style: SeriesStyle::Bars { width: bar_config.bar_width },
        };
        
        Ok(vec![series])
    }
    
    /// Helper method to get bar chart specific config
    fn as_bar_config(config: &PlotConfiguration) -> &BarChartConfig {
        if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            panic!("Expected BarChartConfig")
        }
    }
}

impl PlotTrait for BarChartPlot {
    fn name(&self) -> &'static str {
        "Bar Chart"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Bar charts can have categorical or numeric X axis
        Some(vec![
            DataType::Utf8,
            DataType::LargeUtf8,
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
    
    fn supports_multiple_series(&self) -> bool {
        true
    }
    
    fn supports_color_mapping(&self) -> bool {
        true
    }
    
    fn get_default_config(&self) -> PlotConfiguration {
        let mut config = PlotConfiguration::default();
        config.plot_specific = PlotSpecificConfig::BarChart(BarChartConfig {
            bar_width: 0.7,
            group_spacing: 0.2,
            stacking_mode: StackingMode::None,
            sort_order: SortOrder::None,
        });
        config
    }
    
    fn prepare_data(&self, query_result: &QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        // Use tokio runtime to run async data processing
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Process data based on stacking mode
        let series = match bar_config.stacking_mode {
            StackingMode::None => {
                // If we have a group column, use grouped bars
                if config.group_column.is_some() && !config.group_column.as_ref().unwrap().is_empty() {
                    rt.block_on(self.process_grouped_data(query_result, config))?
                } else {
                    // Otherwise use simple bars
                    rt.block_on(self.process_data(query_result, config))?
                }
            },
            StackingMode::Stacked => {
                rt.block_on(self.process_stacked_data(query_result, config))?
            },
            StackingMode::Percent => {
                rt.block_on(self.process_percent_stacked_data(query_result, config))?
            },
        };
        
        // Create plot metadata with proper axis labels
        let y_label = match bar_config.stacking_mode {
            StackingMode::Percent => format!("{} (%)", config.y_column),
            _ => config.y_column.clone(),
        };
        
        let metadata = super::PlotMetadata {
            title: config.title.clone(),
            x_label: config.x_column.clone(),
            y_label,
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
            statistics: None,
        })
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
                ui.label(RichText::new("Configure category and value columns").weak());
            });
            return;
        }
        
        // Get bar chart specific config
        let default_config;
        let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_bar_chart()
        };
        
        // Create plot with proper axis labels
        let plot = Plot::new("bar_chart")
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
            // Check if we're using stacked bars
            let is_stacked = match bar_config.stacking_mode {
                StackingMode::Stacked | StackingMode::Percent => true,
                _ => false,
            };
            
            // For stacked bars, we need to render in reverse order to ensure proper stacking
            let series_to_render = if is_stacked {
                // Clone and reverse the series for stacked rendering
                let mut reversed = data.series.clone();
                reversed.reverse();
                reversed
            } else {
                data.series.clone()
            };
            
            // Render each series
            for series in &series_to_render {
                if !series.visible {
                    continue;
                }
                
                // Create bars for this series
                let bars: Vec<Bar> = series.points.iter()
                    .map(|point| {
                        let mut bar = if is_stacked {
                            // For stacked bars, we need to use the base height stored in z
                            let base_height = point.z.unwrap_or(0.0);
                            
                            // Create a bar with the base height
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                                .base_offset(base_height)
                        } else {
                            // Regular bar
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                        };
                        
                        if let Some(label) = &point.label {
                            bar = bar.name(label);
                        }
                        
                        if let Some(color) = point.color {
                            bar = bar.fill(color);
                        } else {
                            bar = bar.fill(series.color);
                        }
                        
                        bar
                    })
                    .collect();
                
                // Create bar chart
                let chart = BarChart::new(bars)
                    .name(&series.name)
                    .color(series.color);
                    //.horizontal(false); // Vertical bars by default
                
                // Add to plot
                plot_ui.bar_chart(chart);
            }
            
            // Add category labels on X axis for categorical data
            if !data.points.is_empty() {
                // Get unique categories and their x positions
                let mut categories: Vec<(f64, String)> = Vec::new();
                for point in &data.points {
                    if let Some(label) = &point.label {
                        if !categories.iter().any(|(x, _)| (*x - point.x).abs() < 0.001) {
                            categories.push((point.x, label.clone()));
                        }
                    }
                }
                
                // Sort by x position
                categories.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let category_positions = categories;
                
                // Add custom x-axis labels if we have categorical data
                if !category_positions.is_empty() && !is_stacked {
                    // TODO: Add custom axis labels when egui_plot supports it better
                }
            }
            
            // Handle hover tooltips
            if config.show_tooltips {
                self.handle_tooltips(plot_ui, data);
            }
            
            // Render each series
            for series in &series_to_render {
                if !series.visible {
                    continue;
                }
                
                // Create bars for this series
                let bars: Vec<Bar> = series.points.iter()
                    .map(|point| {
                        let mut bar = if is_stacked {
                            // For stacked bars, we need to use the base height stored in z
                            let base_height = point.z.unwrap_or(0.0);
                            
                            // Create a bar with the base height
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                                .base_offset(base_height)
                        } else {
                            // Regular bar
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                        };
                        
                        if let Some(label) = &point.label {
                            bar = bar.name(label);
                        }
                        
                        if let Some(color) = point.color {
                            bar = bar.fill(color);
                        } else {
                            bar = bar.fill(series.color);
                        }
                        
                        bar
                    })
                    .collect();
                
                // Create bar chart
                let chart = BarChart::new(bars)
                    .name(&series.name)
                    .color(series.color);
                    //.horizontal(false); // Vertical bars by default
                
                // Add to plot
                plot_ui.bar_chart(chart);
            }
            
            // Add category labels on X axis for categorical data
            if !data.points.is_empty() {
                // Get unique categories and their x positions
                let mut categories: Vec<(f64, String)> = Vec::new();
                for point in &data.points {
                    if let Some(label) = &point.label {
                        if !categories.iter().any(|(x, _)| (*x - point.x).abs() < 0.001) {
                            categories.push((point.x, label.clone()));
                        }
                    }
                }
                
                // Sort by x position
                categories.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let category_positions = categories;
                
                // Add custom x-axis labels if we have categorical data
                if !category_positions.is_empty() && !is_stacked {
                    // TODO: Add custom axis labels when egui_plot supports it better
                }
            }
            
            // Handle hover tooltips
            if config.show_tooltips {
                self.handle_tooltips(plot_ui, data);
            }
            
            // Render each series
            for series in &series_to_render {
                if !series.visible {
                    continue;
                }
                
                // Create bars for this series
                let bars: Vec<Bar> = series.points.iter()
                    .map(|point| {
                        let mut bar = if is_stacked {
                            // For stacked bars, we need to use the base height stored in z
                            let base_height = point.z.unwrap_or(0.0);
                            
                            // Create a bar with the base height
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                                .base_offset(base_height)
                        } else {
                            // Regular bar
                            Bar::new(point.x, point.y)
                                .width(bar_config.bar_width as f64)
                        };
                        
                        if let Some(label) = &point.label {
                            bar = bar.name(label);
                        }
                        
                        if let Some(color) = point.color {
                            bar = bar.fill(color);
                        } else {
                            bar = bar.fill(series.color);
                        }
                        
                        bar
                    })
                    .collect();
                
                // Create bar chart
                let chart = BarChart::new(bars)
                    .name(&series.name)
                    .color(series.color);
                    //.horizontal(false); // Vertical bars by default
                
                // Add to plot
                plot_ui.bar_chart(chart);
            }
            
            // Add category labels on X axis for categorical data
            if !data.points.is_empty() {
                // Get unique categories and their x positions
                let mut categories: Vec<(f64, String)> = Vec::new();
                for point in &data.points {
                    if let Some(label) = &point.label {
                        if !categories.iter().any(|(x, _)| (*x - point.x).abs() < 0.001) {
                            categories.push((point.x, label.clone()));
                        }
                    }
                }
                
                // Sort by x position
                categories.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let category_positions = categories;
                
                // Add custom x-axis labels if we have categorical data
                if !category_positions.is_empty() && !is_stacked {
                    // TODO: Add custom axis labels when egui_plot supports it better
                }
            }
            
            // Handle hover tooltips
            if config.show_tooltips {
                self.handle_tooltips(plot_ui, data);
            }
        });
    }
    
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, config: &PlotConfiguration) {
        if !data.series.is_empty() {
            ui.group(|ui| {
                // Get bar chart specific config
                let default_config;
                let bar_config = if let PlotSpecificConfig::BarChart(cfg) = &config.plot_specific {
                    cfg
                } else {
                    default_config = self.get_default_config();
                    default_config.plot_specific.as_bar_chart()
                };
                
                // Determine if we're showing series or categories in the legend
                let has_multiple_series = data.series.len() > 1;
                
                if has_multiple_series {
                    // For grouped or stacked bars, show series in legend
                    ui.label(RichText::new("Series:").strong());
                    ui.separator();
                    
                    for series in &data.series {
                        let mut is_checked = series.visible;
                        if ui.checkbox(&mut is_checked, &series.name).changed() {
                            // TODO: Handle toggling series visibility
                            // This would require mutable access to data, which we don't have here
                            // We'll need to return a PlotInteraction to handle this
                        }
                        
                        ui.horizontal(|ui| {
                            ui.colored_label(series.color, "■■■");
                            ui.label(&series.name);
                        });
                    }
                    
                    // Add stacking mode info if applicable
                    match bar_config.stacking_mode {
                        StackingMode::Stacked => {
                            ui.separator();
                            ui.label(RichText::new("Stacked Bar Chart").italics());
                        },
                        StackingMode::Percent => {
                            ui.separator();
                            ui.label(RichText::new("Percentage Stacked Bar Chart").italics());
                        },
                        _ => {}
                    }
                } else {
                    // For simple bars, show categories
                    ui.label(RichText::new("Categories:").strong());
                    ui.separator();
                    
                    // Get unique categories with their colors
                    let mut category_colors = std::collections::HashMap::new();
                    for series in &data.series {
                        if series.visible {
                            for point in &series.points {
                                if let (Some(label), Some(color)) = (&point.label, point.color) {
                                    category_colors.insert(label.clone(), color);
                                }
                            }
                        }
                    }
                    
                    // Sort categories alphabetically for consistent display
                    let mut categories: Vec<(String, Color32)> = category_colors
                        .into_iter()
                        .map(|(k, v)| (k, v))
                        .collect();
                    
                    categories.sort_by(|a, b| a.0.cmp(&b.0));
                    
                    // Display categories
                    for (label, color) in categories {
                        ui.horizontal(|ui| {
                            ui.colored_label(color, "■■■");
                            ui.label(&label);
                        });
                    }
                    
                    // Add sorting info if applicable
                    match bar_config.sort_order {
                        SortOrder::Ascending => {
                            ui.separator();
                            ui.label(RichText::new("Sorted by value (ascending)").italics());
                        },
                        SortOrder::Descending => {
                            ui.separator();
                            ui.label(RichText::new("Sorted by value (descending)").italics());
                        },
                        SortOrder::ByValue => {
                            ui.separator();
                            ui.label(RichText::new("Sorted by category").italics());
                        },
                        _ => {}
                    }
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
        
        // Handle bar selection
        if config.allow_selection {
            // TODO: Implement bar selection interaction
            // This would return PlotInteraction::PointSelected with indices of selected bars
        }
        
        None
    }
}