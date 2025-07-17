use egui::{Ui, Color32};
use egui_plot::{Bar, BarChart, Plot, Legend};
use datafusion::arrow::datatypes::DataType;

use super::{
    Plot as PlotTrait, 
    PlotData, 
    PlotConfiguration, 
    PlotSpecificConfig, 
    HistogramConfig, 
    PlotInteraction,
    DataSeries,
    SeriesStyle,
    data_processor::DataProcessor,
    DataStatistics
};

pub struct HistogramPlot;

impl HistogramPlot {
    /// Handle tooltips for histogram
    fn handle_tooltips(&self, plot_ui: &egui_plot::PlotUi, data: &PlotData, bin_edges: &[(f64, f64, usize)]) {
        if let Some(pointer_coord) = plot_ui.pointer_coordinate() {
            // Find the bin under the cursor
            for &(bin_start, bin_end, count) in bin_edges {
                if pointer_coord.x >= bin_start && pointer_coord.x < bin_end {
                    // Show tooltip with bin data
                    let tooltip_text = format!(
                        "Range: {:.2} - {:.2}\nCount: {}\nFrequency: {:.2}%",
                        bin_start, bin_end, count,
                        100.0 * count as f64 / data.points.len() as f64
                    );
                    
                    // Note: show_tooltip is not available in the current egui_plot version
                    // We'll need to use a different approach for tooltips
                    
                    // Highlight the bar
                    let highlight_color = Color32::from_rgb(180, 180, 250);
                    let bin_center = (bin_start + bin_end) / 2.0;
                    let bin_width = bin_end - bin_start;
                    
                    let highlight_bar = Bar::new(bin_center, count as f64)
                        .width(bin_width)
                        .fill(highlight_color);
                    
                    // Note: bar method is not available in the current egui_plot version
                    // We'll need to use a different approach for highlighting
                    
                    break;
                }
            }
        }
    }
    
    /// Calculate optimal bin count using Freedman-Diaconis rule
    fn calculate_optimal_bin_count(&self, values: &[f64]) -> usize {
        if values.len() < 2 {
            return 1;
        }
        
        // Sort values for quartile calculation
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let n = sorted_values.len();
        let q1_idx = n / 4;
        let q3_idx = 3 * n / 4;
        let q1 = sorted_values[q1_idx];
        let q3 = sorted_values[q3_idx];
        let iqr = q3 - q1;
        
        if iqr <= 0.0 {
            return 10; // Default if IQR is too small
        }
        
        let h = 2.0 * iqr / (n as f64).powf(1.0/3.0);
        let range = sorted_values[n-1] - sorted_values[0];
        
        let bin_count = (range / h).ceil() as usize;
        bin_count.max(5).min(100) // Reasonable limits
    }
    
    /// Process data for histogram with proper binning
    async fn process_data(&self, query_result: &crate::core::QueryResult, config: &PlotConfiguration) -> Result<(Vec<DataSeries>, Vec<(f64, f64, usize)>), String> {
        let data_processor = DataProcessor::new();
        
        // Get histogram specific config
        let default_config;
        let hist_config = if let PlotSpecificConfig::Histogram(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_histogram()
        };
        
        // Extract values from query result
        let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
            .ok_or_else(|| format!("Y column '{}' not found", config.y_column))?;
        
        let mut values = Vec::new();
        for row in &query_result.rows {
            if row.len() > y_idx {
                if let Ok(val) = row[y_idx].parse::<f64>() {
                    values.push(val);
                }
            }
        }
        
        if values.is_empty() {
            return Err("No numeric data to create histogram".to_string());
        }
        
        // Determine bin count
        let bin_count = if let Some(count) = hist_config.bin_count {
            count
        } else {
            self.calculate_optimal_bin_count(&values)
        };
        
        // Calculate histogram bins
        let histogram_data = data_processor.compute_histogram_bins(
            query_result,
            &config.y_column,
            bin_count
        ).await?;
        
        // Convert to bin edges format for tooltips
        let mut bin_edges = Vec::new();
        for (i, (bin_start, bin_end, count)) in histogram_data.iter().enumerate() {
            bin_edges.push((*bin_start, *bin_end, *count));
        }
        
        // Create bars for the histogram
        let mut points = Vec::new();
        for (bin_start, bin_end, count) in &histogram_data {
            let bin_center = (bin_start + bin_end) / 2.0;
            let bin_width = bin_end - bin_start;
            
            // Create tooltip data
            let mut tooltip_data = std::collections::HashMap::new();
            tooltip_data.insert("Range Start".to_string(), format!("{:.2}", bin_start));
            tooltip_data.insert("Range End".to_string(), format!("{:.2}", bin_end));
            tooltip_data.insert("Count".to_string(), count.to_string());
            tooltip_data.insert("Frequency".to_string(), 
                format!("{:.2}%", 100.0 * *count as f64 / values.len() as f64));
            
            points.push(super::PlotPoint {
                x: bin_center,
                y: *count as f64,
                z: None,
                label: Some(format!("{:.2} - {:.2}", bin_start, bin_end)),
                color: Some(Color32::from_rgb(100, 100, 250)),
                size: None,
                series_id: Some("histogram".to_string()),
                tooltip_data,
            });
        }
        
        // Create histogram series
        let series = DataSeries {
            id: "histogram".to_string(),
            name: format!("Histogram of {}", config.y_column),
            points,
            color: Color32::from_rgb(100, 100, 250),
            visible: true,
            style: SeriesStyle::Bars { width: 0.9 },
        };
        
        // Add normal distribution curve if requested
        let mut all_series = vec![series];
        
        if hist_config.show_normal_curve && values.len() > 2 {
            // Calculate mean and standard deviation
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();
            
            if std_dev > 0.0 {
                // Create normal distribution curve
                let min_val = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                let max_val = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
                let range = max_val - min_val;
                
                // Create 100 points for the curve
                let mut curve_points = Vec::new();
                let max_count = histogram_data.iter().map(|(_, _, count)| *count).max().unwrap_or(1) as f64;
                
                for i in 0..100 {
                    let x = min_val - range * 0.2 + range * 1.4 * i as f64 / 99.0;
                    let z = (x - mean) / std_dev;
                    let y = (-(z * z) / 2.0).exp() / (std_dev * (2.0 * std::f64::consts::PI).sqrt());
                    
                    // Scale to match histogram height
                    let scaled_y = y * values.len() as f64 * (bin_edges[0].1 - bin_edges[0].0);
                    
                    curve_points.push(super::PlotPoint {
                        x,
                        y: scaled_y,
                        z: None,
                        label: None,
                        color: Some(Color32::from_rgb(255, 100, 100)),
                        size: None,
                        series_id: Some("normal_curve".to_string()),
                        tooltip_data: std::collections::HashMap::new(),
                    });
                }
                
                let normal_series = DataSeries {
                    id: "normal_curve".to_string(),
                    name: "Normal Distribution".to_string(),
                    points: curve_points,
                    color: Color32::from_rgb(255, 100, 100),
                    visible: true,
                    style: SeriesStyle::Line { width: 2.0, dashed: false },
                };
                
                all_series.push(normal_series);
            }
        }
        
        Ok((all_series, bin_edges))
    }
    
    /// Helper method to get histogram specific config
    fn as_histogram_config(config: &PlotConfiguration) -> &HistogramConfig {
        if let PlotSpecificConfig::Histogram(cfg) = &config.plot_specific {
            cfg
        } else {
            panic!("Expected HistogramConfig")
        }
    }
}

impl PlotTrait for HistogramPlot {
    fn name(&self) -> &'static str {
        "Histogram"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Histograms don't require an X column - they compute bins from Y values
        None
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        // Y axis must be numeric for histogram calculation
        vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float16, DataType::Float32, DataType::Float64,
        ]
    }
    
    fn supports_multiple_series(&self) -> bool {
        true // For normal distribution overlay
    }
    
    fn get_default_config(&self) -> PlotConfiguration {
        let mut config = PlotConfiguration::default();
        config.plot_specific = PlotSpecificConfig::Histogram(HistogramConfig {
            bin_count: None, // Auto-calculate using Freedman-Diaconis rule
            bin_width: None,
            show_density: false,
            show_normal_curve: true,
        });
        config
    }
    
    fn validate_columns(&self, _query_result: &crate::core::QueryResult, config: &PlotConfiguration) -> Result<(), String> {
        // Histograms only need a Y column with numeric data
        if config.y_column.is_empty() {
            return Err("Y column is required for histogram".to_string());
        }
        
        Ok(())
    }
    
    fn prepare_data(&self, query_result: &crate::core::QueryResult, config: &PlotConfiguration) -> Result<PlotData, String> {
        // Use tokio runtime to run async data processing
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        let (series, bin_edges) = rt.block_on(self.process_data(query_result, config))?;
        
        // Calculate statistics
        let y_idx = query_result.columns.iter().position(|c| c == &config.y_column)
            .ok_or_else(|| format!("Y column '{}' not found", config.y_column))?;
        
        let mut values = Vec::new();
        for row in &query_result.rows {
            if row.len() > y_idx {
                if let Ok(val) = row[y_idx].parse::<f64>() {
                    values.push(val);
                }
            }
        }
        
        let mean = if !values.is_empty() {
            values.iter().sum::<f64>() / values.len() as f64
        } else {
            0.0
        };
        
        let std_dev = if values.len() > 1 {
            let variance = values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / values.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };
        
        let statistics = Some(DataStatistics {
            mean_x: 0.0, // Not applicable for histogram
            mean_y: mean,
            std_x: 0.0, // Not applicable for histogram
            std_y: std_dev,
            correlation: None, // Not applicable for histogram
            count: values.len(),
        });
        
        // Create plot metadata
        let metadata = super::PlotMetadata {
            title: config.title.clone(),
            x_label: config.y_column.clone(), // X-axis shows the values being binned
            y_label: "Frequency".to_string(),
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
                ui.label("Configure Y column with numeric data");
            });
            return;
        }
        
        // Get histogram specific config
        let default_config;
        let hist_config = if let PlotSpecificConfig::Histogram(cfg) = &config.plot_specific {
            cfg
        } else {
            default_config = self.get_default_config();
            default_config.plot_specific.as_histogram()
        };
        
        // Extract bin edges for tooltips
        let mut bin_edges = Vec::new();
        for point in &data.points {
            if let (Some(series_id), Some(label)) = (&point.series_id, &point.label) {
                if series_id == "histogram" && label.contains('-') {
                    if let Some(tooltip_data) = point.tooltip_data.get("Count") {
                        if let Ok(count) = tooltip_data.parse::<usize>() {
                            if let Some(range_start) = point.tooltip_data.get("Range Start") {
                                if let Some(range_end) = point.tooltip_data.get("Range End") {
                                    if let (Ok(start), Ok(end)) = (range_start.parse::<f64>(), range_end.parse::<f64>()) {
                                        bin_edges.push((start, end, count));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create plot
        let plot = Plot::new("histogram")
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
            // Render each series
            for series in &data.series {
                if !series.visible {
                    continue;
                }
                
                match series.style {
                    SeriesStyle::Bars { width } => {
                        // Create bars for histogram
                        let bars: Vec<Bar> = series.points.iter()
                            .map(|point| {
                                let mut bar = Bar::new(point.x, point.y)
                                    .width(width as f64);
                                
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
                        
                        // Add to plot
                        plot_ui.bar_chart(chart);
                    },
                    SeriesStyle::Line { width, dashed } => {
                        // Create plot points for normal curve
                        let plot_points: Vec<[f64; 2]> = series.points.iter()
                            .map(|p| [p.x, p.y])
                            .collect();
                        
                        // Create line
                        let mut line = egui_plot::Line::new(plot_points)
                            .name(&series.name)
                            .color(series.color)
                            .width(width);
                        
                        // Apply line style
                        if dashed {
                            line = line.style(egui_plot::LineStyle::dashed_dense());
                        }
                        
                        // Add line to plot
                        plot_ui.line(line);
                    },
                    _ => {} // Other styles not applicable for histogram
                }
            }
            
            // Note: Tooltips are not implemented in this version due to API limitations
        });
        
        // Show statistics if available
        if let Some(stats) = &data.statistics {
            ui.horizontal(|ui| {
                ui.label("Mean:");
                ui.label(format!("{:.3}", stats.mean_y));
                
                ui.separator();
                
                ui.label("Std Dev:");
                ui.label(format!("{:.3}", stats.std_y));
                
                ui.separator();
                
                ui.label(format!("Count: {}", stats.count));
            });
        }
    }
    
    fn render_legend(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) {
        if !data.series.is_empty() {
            ui.group(|ui| {
                ui.label("Series:");
                ui.separator();
                
                for series in &data.series {
                    if series.visible {
                        ui.horizontal(|ui| {
                            match series.style {
                                SeriesStyle::Bars { .. } => {
                                    ui.colored_label(series.color, "■");
                                },
                                SeriesStyle::Line { .. } => {
                                    ui.colored_label(series.color, "—");
                                },
                                _ => {
                                    ui.colored_label(series.color, "●");
                                }
                            }
                            ui.label(&series.name);
                        });
                    }
                }
            });
        }
    }
    
    fn handle_interaction(&self, _ui: &mut Ui, _data: &PlotData, _config: &PlotConfiguration) -> Option<PlotInteraction> {
        None
    }
}

// Extension trait for PlotSpecificConfig
trait AsHistogram {
    fn as_histogram(&self) -> &HistogramConfig;
}

impl AsHistogram for PlotSpecificConfig {
    fn as_histogram(&self) -> &HistogramConfig {
        match self {
            PlotSpecificConfig::Histogram(cfg) => cfg,
            _ => panic!("Expected HistogramConfig"),
        }
    }
}