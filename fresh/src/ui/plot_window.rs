use egui::{Color32, RichText, Ui, ScrollArea};
use crate::core::QueryResult;
use crate::ui::plots::{self, PlotType, PlotData, PlotPoint, Plot as PlotTrait};
use crate::ui::gpu_renderer::{GpuPlotRenderer, RenderMode};
use datafusion::arrow::datatypes::DataType;

#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub title: String,
    pub plot_type: Option<PlotType>,
    pub x_column: String,
    pub y_column: String,
    pub show_legend: bool,
    pub show_grid: bool,
    // GPU rendering options
    pub use_gpu_rendering: bool,
    pub render_mode: RenderMode,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            plot_type: None,
            x_column: String::new(),
            y_column: String::new(),
            show_legend: true,
            show_grid: true,
            use_gpu_rendering: true,
            render_mode: RenderMode::Auto,
        }
    }
}

pub struct PlotWindow<'a> {
    pub id: String,
    pub title: String,
    pub config: PlotConfig,
    pub is_config_open: bool,
    pub open: bool,
    data: Option<QueryResult>,
    source_query_id: Option<String>, // ID of the query window this plot is connected to
    // GPU renderer
    gpu_renderer: Option<GpuPlotRenderer<'a>>,
}

impl<'a> PlotWindow<'a> {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            config: PlotConfig::default(),
            is_config_open: true,
            open: true,
            data: None,
            source_query_id: None,
            gpu_renderer: None,
        }
    }

    pub fn update_data(&mut self, data: QueryResult) {
        self.data = Some(data);
    }

    pub fn set_source_query(&mut self, query_id: String) {
        self.source_query_id = Some(query_id);
    }

    /// Initialize GPU renderer if available
    pub async fn initialize_gpu_renderer(&mut self) {
        if self.gpu_renderer.is_none() {
            match GpuPlotRenderer::new().await {
                Ok(renderer) => {
                    self.gpu_renderer = Some(renderer);
                    println!("GPU renderer initialized successfully");
                }
                Err(e) => {
                    println!("Failed to initialize GPU renderer: {}", e);
                    self.config.use_gpu_rendering = false;
                    self.config.render_mode = RenderMode::Cpu;
                }
            }
        }
    }

    /// Get GPU capabilities if available
    pub fn get_gpu_capabilities(&self) -> Option<String> {
        self.gpu_renderer.as_ref().and_then(|renderer| {
            renderer.get_capabilities().map(|caps| {
                format!(
                    "Max vertices: {}, Max instances: {}, Instancing: {}, Compute: {}",
                    caps.max_vertices, caps.max_instances, caps.supports_instancing, caps.supports_compute
                )
            })
        })
    }

    /// Returns a Vec of (column name, compatible: bool, reason: Option<String>) for X or Y columns
    fn column_compatibility<'b>(
        &self,
        axis: &'static str, // "x" or "y"
        plot_type: &PlotType,
        data: &'b QueryResult,
    ) -> Vec<(&'b String, bool, Option<String>)> {
        let required_types = if axis == "x" {
            plot_type.required_x_types()
        } else {
            Some(plot_type.required_y_types())
        };
        data.columns.iter().enumerate().map(|(i, col)| {
            let dtype = &data.column_types[i];
            let compatible = if let Some(ref types) = required_types {
                types.iter().any(|t: &DataType| t == dtype)
            } else {
                true
            };
            let reason = if compatible {
                None
            } else {
                Some("incompatible column type".to_string())
            };
            (col, compatible, reason)
        }).collect()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_gray(18);
        
        // Header
        ui.horizontal(|ui| {
            if ui.button(if self.is_config_open { "▼ Plot Configuration" } else { "▶ Plot Configuration" }).clicked() {
                self.is_config_open = !self.is_config_open;
            }
        });

        if self.is_config_open {
            ui.group(|ui| {
                ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_gray(22);
                
                // Title
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut self.config.title);
                });
                
                ui.separator();
                
                // Plot Type Selection with categories
                ui.horizontal(|ui| {
                    ui.label("Plot Type:");
                    
                    egui::ComboBox::new("plot_type", "")
                        .selected_text(self.config.plot_type.as_ref().map(|pt| pt.name()).unwrap_or("Select..."))
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            // Show plots by category
                            for (category, plot_types) in PlotType::categories() {
                                ui.label(RichText::new(category).strong());
                                ui.separator();
                                
                                for plot_type in plot_types {
                                    let is_compatible = self.check_plot_compatibility(&plot_type);
                                    let label = if is_compatible {
                                        RichText::new(plot_type.name())
                                    } else {
                                        RichText::new(format!("{} (incompatible columns)", plot_type.name()))
                                            .color(Color32::from_gray(100))
                                    };
                                    
                                    if ui.selectable_value(&mut self.config.plot_type, Some(plot_type.clone()), label).clicked() {
                                        ui.close_menu();
                                    }
                                }
                                
                                ui.add_space(5.0);
                            }
                        });
                    
                    ui.add_space(20.0);
                    
                    ui.checkbox(&mut self.config.show_legend, "Show Legend");
                    ui.checkbox(&mut self.config.show_grid, "Show Grid");
                });
                
                ui.separator();
                
                // Column selection with type information
                if let Some(data) = &self.data {
                    let plot_type = self.config.plot_type.as_ref();
                    let x_compat = plot_type.map(|pt| self.column_compatibility("x", pt, data));
                    let y_compat = plot_type.map(|pt| self.column_compatibility("y", pt, data));
                    ui.horizontal(|ui| {
                        ui.label("X Column:");
                        egui::ComboBox::new("x_column", "")
                            .selected_text(if self.config.x_column.is_empty() { "Select..." } else { &self.config.x_column })
                            .show_ui(ui, |ui| {
                                if let Some(x_compat) = x_compat.as_ref() {
                                    let mut iter = x_compat.iter();
                                    let response = if let Some((col, compatible, reason)) = iter.next() {
                                        let label = if let Some(reason) = reason {
                                            format!("{} ({})", col, reason)
                                        } else {
                                            col.to_string()
                                        };
                                        let mut response = if *compatible {
                                            ui.selectable_value(&mut self.config.x_column, col.to_string(), &label)
                                        } else {
                                            ui.add_enabled(false, |ui: &mut egui::Ui| {
                                                ui.selectable_value(&mut self.config.x_column, col.to_string(), &label)
                                                    .on_hover_text("Incompatible column type")
                                            })
                                        };
                                        for (col, compatible, reason) in iter {
                                            let label = if let Some(reason) = reason {
                                                format!("{} ({})", col, reason)
                                            } else {
                                                col.to_string()
                                            };
                                            let r = if *compatible {
                                                ui.selectable_value(&mut self.config.x_column, col.to_string(), &label)
                                            } else {
                                                ui.add_enabled(false, |ui: &mut egui::Ui| {
                                                    ui.selectable_value(&mut self.config.x_column, col.to_string(), &label)
                                                        .on_hover_text("Incompatible column type")
                                                })
                                            };
                                            response |= r;
                                        }
                                        response
                                    } else {
                                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                                    };
                                    response
                                } else {
                                    let mut iter = data.columns.iter().enumerate();
                                    let response = if let Some((i, column)) = iter.next() {
                                        let col_type = &data.column_types[i];
                                        let type_str = format_data_type(col_type);
                                        let label = format!("{} ({})", column, type_str);
                                        let mut response = ui.selectable_value(&mut self.config.x_column, column.to_string(), &label);
                                        for (i, column) in iter {
                                            let col_type = &data.column_types[i];
                                            let type_str = format_data_type(col_type);
                                            let label = format!("{} ({})", column, type_str);
                                            response |= ui.selectable_value(&mut self.config.x_column, column.to_string(), &label);
                                        }
                                        response
                                    } else {
                                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                                    };
                                    response
                                }
                            });
                        
                        ui.add_space(20.0);
                        
                        ui.label("Y Column:");
                        egui::ComboBox::new("y_column", "")
                            .selected_text(if self.config.y_column.is_empty() { "Select..." } else { &self.config.y_column })
                            .show_ui(ui, |ui| {
                                if let Some(y_compat) = y_compat.as_ref() {
                                    let mut iter = y_compat.iter();
                                    let response = if let Some((col, compatible, reason)) = iter.next() {
                                        let label = if let Some(reason) = reason {
                                            format!("{} ({})", col, reason)
                                        } else {
                                            col.to_string()
                                        };
                                        let mut response = if *compatible {
                                            ui.selectable_value(&mut self.config.y_column, col.to_string(), &label)
                                        } else {
                                            ui.add_enabled(false, |ui: &mut egui::Ui| {
                                                ui.selectable_value(&mut self.config.y_column, col.to_string(), &label)
                                                    .on_hover_text("Incompatible column type")
                                            })
                                        };
                                        for (col, compatible, reason) in iter {
                                            let label = if let Some(reason) = reason {
                                                format!("{} ({})", col, reason)
                                            } else {
                                                col.to_string()
                                            };
                                            let r = if *compatible {
                                                ui.selectable_value(&mut self.config.y_column, col.to_string(), &label)
                                            } else {
                                                ui.add_enabled(false, |ui: &mut egui::Ui| {
                                                    ui.selectable_value(&mut self.config.y_column, col.to_string(), &label)
                                                        .on_hover_text("Incompatible column type")
                                                })
                                            };
                                            response |= r;
                                        }
                                        response
                                    } else {
                                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                                    };
                                    response
                                } else {
                                    let mut iter = data.columns.iter().enumerate();
                                    let response = if let Some((i, column)) = iter.next() {
                                        let col_type = &data.column_types[i];
                                        let type_str = format_data_type(col_type);
                                        let label = format!("{} ({})", column, type_str);
                                        let mut response = ui.selectable_value(&mut self.config.y_column, column.to_string(), &label);
                                        for (i, column) in iter {
                                            let col_type = &data.column_types[i];
                                            let type_str = format_data_type(col_type);
                                            let label = format!("{} ({})", column, type_str);
                                            response |= ui.selectable_value(&mut self.config.y_column, column.to_string(), &label);
                                        }
                                        response
                                    } else {
                                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                                    };
                                    response
                                }
                            });
                    });
                    
                    // Show validation errors if any
                    if let Some(plot_type) = &self.config.plot_type {
                        if !self.config.x_column.is_empty() || !self.config.y_column.is_empty() {
                            let validation_result = self.validate_columns(plot_type);
                            if let Err(msg) = validation_result {
                                ui.colored_label(Color32::from_rgb(255, 100, 100), format!("⚠ {}", msg));
                            }
                        }
                    }
                } else {
                    ui.label(RichText::new("No data available for column selection").size(12.0).color(Color32::from_gray(120)));
                }

                ui.separator();

                // GPU Rendering Configuration
                ui.horizontal(|ui| {
                    ui.label("GPU Rendering:");
                    ui.checkbox(&mut self.config.use_gpu_rendering, "Enable GPU Acceleration");
                    
                    if self.config.use_gpu_rendering {
                        ui.add_space(20.0);
                        ui.label("Mode:");
                        egui::ComboBox::new("render_mode", "")
                            .selected_text(match self.config.render_mode {
                                RenderMode::Gpu => "GPU Only",
                                RenderMode::Cpu => "CPU Only",
                                RenderMode::Auto => "Auto",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.config.render_mode, RenderMode::Auto, "Auto");
                                ui.selectable_value(&mut self.config.render_mode, RenderMode::Gpu, "GPU Only");
                                ui.selectable_value(&mut self.config.render_mode, RenderMode::Cpu, "CPU Only");
                            });
                    }
                });

                // Show GPU capabilities if available
                if self.config.use_gpu_rendering {
                    if let Some(capabilities) = self.get_gpu_capabilities() {
                        ui.label(RichText::new(format!("GPU: {}", capabilities)).size(10.0).color(Color32::from_gray(120)));
                    } else {
                        ui.label(RichText::new("GPU: Not available").size(10.0).color(Color32::from_gray(120)));
                    }
                }
            });
        }

        // Plot rendering area
        ui.add_space(10.0);
        
        // Prepare all data and check for errors outside of any closures
        let plot_type_opt = self.config.plot_type.clone();
        let data_opt = self.data.as_ref();
        let mut plot_data_to_render = None;
        let mut plot_type_to_render = None;
        let mut render_error = None;
        
        if let Some(data) = data_opt {
            if let Some(plot_type) = plot_type_opt {
                match self.prepare_plot_data(data, &plot_type) {
                    Ok(plot_data) => {
                        plot_data_to_render = Some(plot_data);
                        plot_type_to_render = Some(plot_type);
                    }
                    Err(_) => {
                        render_error = Some("Invalid data for selected plot type");
                    }
                }
            } else {
                render_error = Some("Please select a plot type");
            }
        } else {
            render_error = Some("No data available");
        }
        
        // Render errors outside of scroll area
        if let Some(msg) = render_error {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(msg).size(14.0).color(Color32::from_gray(150)));
            });
        } else if let (Some(plot_type), Some(plot_data)) = (plot_type_to_render, plot_data_to_render) {
            // Only render the plot inside scroll area if we have valid data
            self.render_plot(ui, &plot_type, plot_data);
        }
    }
    
    fn check_plot_compatibility(&self, plot_type: &PlotType) -> bool {
        if let Some(data) = &self.data {
            if !self.config.x_column.is_empty() && !self.config.y_column.is_empty() {
                if let Some(x_idx) = data.columns.iter().position(|c| c == &self.config.x_column) {
                    if let Some(y_idx) = data.columns.iter().position(|c| c == &self.config.y_column) {
                        let x_type = &data.column_types[x_idx];
                        let y_type = &data.column_types[y_idx];
                        return plot_type.supports_column_types(Some(x_type), y_type);
                    }
                }
            }
        }
        true // If no columns selected, assume compatible
    }
    
    fn validate_columns(&self, plot_type: &PlotType) -> Result<(), String> {
        if let Some(data) = &self.data {
            let x_type = if !self.config.x_column.is_empty() {
                data.columns.iter().position(|c| c == &self.config.x_column)
                    .map(|idx| &data.column_types[idx])
            } else {
                None
            };
            
            let y_type = if !self.config.y_column.is_empty() {
                data.columns.iter().position(|c| c == &self.config.y_column)
                    .map(|idx| &data.column_types[idx])
            } else {
                return Err("Y column is required".to_string());
            };
            
            if let Some(y_type) = y_type {
                if !plot_type.supports_column_types(x_type, y_type) {
                    return Err(format!(
                        "{} requires {} X column and {} Y column",
                        plot_type.name(),
                        if x_type.is_some() { "numeric/categorical" } else { "no" },
                        "numeric"
                    ));
                }
            }
        }
        Ok(())
    }
    
    fn prepare_plot_data(&self, data: &QueryResult, plot_type: &PlotType) -> Result<PlotData, String> {
        self.validate_columns(plot_type)?;
        
        // Create a plot configuration from the current settings
        let plot_config = plots::PlotConfiguration {
            title: self.config.title.clone(),
            x_column: self.config.x_column.clone(),
            y_column: self.config.y_column.clone(),
            color_column: None,
            size_column: None,
            group_column: None,
            show_legend: self.config.show_legend,
            show_grid: self.config.show_grid,
            show_axes_labels: true,
            color_scheme: plots::ColorScheme::default(),
            marker_size: 4.0,
            line_width: 2.0,
            allow_zoom: true,
            allow_pan: true,
            allow_selection: true,
            show_tooltips: true,
            plot_specific: match plot_type {
                plots::PlotType::BarChart => plots::PlotSpecificConfig::BarChart(plots::BarChartConfig::default()),
                plots::PlotType::LineChart => plots::PlotSpecificConfig::LineChart(plots::LineChartConfig::default()),
                plots::PlotType::ScatterPlot => plots::PlotSpecificConfig::ScatterPlot(plots::ScatterPlotConfig::default()),
                plots::PlotType::Histogram => plots::PlotSpecificConfig::Histogram(plots::HistogramConfig::default()),
                plots::PlotType::BoxPlot => plots::PlotSpecificConfig::BoxPlot(plots::BoxPlotConfig::default()),
                plots::PlotType::ViolinPlot => plots::PlotSpecificConfig::Violin(plots::ViolinPlotConfig::default()),
                _ => plots::PlotSpecificConfig::None,
            },
        };
        
        // Use the plot-specific prepare_data method
        match plot_type {
            plots::PlotType::BarChart => plots::BarChartPlot.prepare_data(data, &plot_config),
            plots::PlotType::LineChart => plots::LineChartPlot.prepare_data(data, &plot_config),
            plots::PlotType::ScatterPlot => plots::scatter::ScatterPlot.prepare_data(data, &plot_config),
            plots::PlotType::Histogram => plots::HistogramPlot.prepare_data(data, &plot_config),
            plots::PlotType::BoxPlot => plots::BoxPlotImpl.prepare_data(data, &plot_config),
            plots::PlotType::HeatMap => plots::heatmap::HeatmapPlot.prepare_data(data, &plot_config),
            plots::PlotType::ViolinPlot => plots::violin::ViolinPlot::new().prepare_data(data, &plot_config),
            plots::PlotType::AnomalyDetection => plots::anomaly::AnomalyPlot.prepare_data(data, &plot_config),
            plots::PlotType::CorrelationMatrix => plots::correlation::CorrelationPlot.prepare_data(data, &plot_config),
            plots::PlotType::DistributionPlot => plots::distribution::DistributionPlot.prepare_data(data, &plot_config),
            plots::PlotType::Scatter3D => plots::scatter3d::Scatter3dPlot.prepare_data(data, &plot_config),
            plots::PlotType::Surface3D => plots::surface3d::Surface3dPlot.prepare_data(data, &plot_config),
            plots::PlotType::ContourPlot => plots::contour::ContourPlot.prepare_data(data, &plot_config),
            plots::PlotType::ParallelCoordinates => plots::parallel_coordinates::ParallelCoordinatesPlot.prepare_data(data, &plot_config),
            plots::PlotType::RadarChart => plots::radar::RadarPlot.prepare_data(data, &plot_config),
            plots::PlotType::SankeyDiagram => plots::sankey::SankeyPlot.prepare_data(data, &plot_config),
            plots::PlotType::Treemap => plots::treemap::TreemapPlot.prepare_data(data, &plot_config),
            plots::PlotType::SunburstChart => plots::sunburst::SunburstPlot.prepare_data(data, &plot_config),
            plots::PlotType::NetworkGraph => plots::network::NetworkPlot.prepare_data(data, &plot_config),
            plots::PlotType::GeoPlot => plots::geo::GeoPlot.prepare_data(data, &plot_config),
            plots::PlotType::TimeAnalysis => plots::time_analysis::TimeAnalysisPlot.prepare_data(data, &plot_config),
            plots::PlotType::CandlestickChart => plots::candlestick::CandlestickPlot.prepare_data(data, &plot_config),
            plots::PlotType::StreamGraph => plots::stream::StreamPlot.prepare_data(data, &plot_config),
            plots::PlotType::PolarPlot => plots::polar::PolarPlot.prepare_data(data, &plot_config),
        }
    }
    
    fn extract_plot_points(&self, data: &QueryResult) -> Result<Vec<PlotPoint>, String> {
        if self.config.y_column.is_empty() {
            return Err("Y column not selected".to_string());
        }

        let y_idx = data.columns.iter().position(|c| c == &self.config.y_column)
            .ok_or("Y column not found")?;
        
        let x_idx = if !self.config.x_column.is_empty() {
            data.columns.iter().position(|c| c == &self.config.x_column)
        } else {
            None
        };

        let mut points = Vec::new();
        
        for (row_idx, row) in data.rows.iter().enumerate() {
            if row.len() > y_idx {
                let y_val = row[y_idx].parse::<f64>()
                    .map_err(|_| format!("Failed to parse Y value '{}' as number", row[y_idx]))?;
                
                let x_val = if let Some(x_idx) = x_idx {
                    if row.len() > x_idx {
                        row[x_idx].parse::<f64>().unwrap_or(row_idx as f64)
                    } else {
                        row_idx as f64
                    }
                } else {
                    row_idx as f64
                };
                
                // Create tooltip data
                let mut tooltip_data = std::collections::HashMap::new();
                tooltip_data.insert("X".to_string(), x_val.to_string());
                tooltip_data.insert("Y".to_string(), y_val.to_string());
                
                points.push(PlotPoint {
                    x: x_val,
                    y: y_val,
                    z: None,
                    label: None,
                    color: None,
                    size: None,
                    series_id: None,
                    tooltip_data,
                });
            }
        }
        
        Ok(points)
    }
    
    fn render_plot(&mut self, ui: &mut Ui, plot_type: &PlotType, plot_data: PlotData) {
        // Create a plot configuration from the current settings with proper plot-specific config
        let plot_config = plots::PlotConfiguration {
            title: self.config.title.clone(),
            x_column: self.config.x_column.clone(),
            y_column: self.config.y_column.clone(),
            color_column: None,
            size_column: None,
            group_column: None,
            show_legend: self.config.show_legend,
            show_grid: self.config.show_grid,
            show_axes_labels: true,
            color_scheme: plots::ColorScheme::default(),
            marker_size: 4.0,
            line_width: 2.0,
            allow_zoom: true,
            allow_pan: true,
            allow_selection: true,
            show_tooltips: true,
            plot_specific: match plot_type {
                PlotType::BarChart => plots::PlotSpecificConfig::BarChart(plots::BarChartConfig::default()),
                PlotType::LineChart => plots::PlotSpecificConfig::LineChart(plots::LineChartConfig::default()),
                PlotType::ScatterPlot => plots::PlotSpecificConfig::ScatterPlot(plots::ScatterPlotConfig::default()),
                PlotType::Histogram => plots::PlotSpecificConfig::Histogram(plots::HistogramConfig::default()),
                PlotType::BoxPlot => plots::PlotSpecificConfig::BoxPlot(plots::BoxPlotConfig::default()),
                PlotType::ViolinPlot => plots::PlotSpecificConfig::Violin(plots::ViolinPlotConfig::default()),
                _ => plots::PlotSpecificConfig::None,
            },
        };
        
        // Prepare GPU rendering data if needed
        let gpu_points = if self.config.use_gpu_rendering && matches!(plot_type, PlotType::LineChart) {
            if let Some(data_ref) = self.data.as_ref() {
                self.extract_plot_points(data_ref).ok()
            } else {
                None
            }
        } else {
            None
        };
        
        // Try GPU rendering if enabled and available
        if self.config.use_gpu_rendering {
            if let (Some(ref mut gpu_renderer), Some(points)) = (self.gpu_renderer.as_mut(), gpu_points) {
                if let Ok(()) = gpu_renderer.render_line_chart(
                    &points.iter().map(|p| egui::Pos2::new(p.x as f32, p.y as f32)).collect::<Vec<_>>(),
                    egui::Color32::BLUE,
                    2.0
                ) {
                    // GPU rendering successful, return early
                    return;
                }
            }
        }
        
        // Fall back to CPU rendering with scroll area
        ScrollArea::vertical().show(ui, |ui| {
            use PlotType::*;
            
            match plot_type {
                BarChart => plots::BarChartPlot.render(ui, &plot_data, &plot_config),
                LineChart => plots::LineChartPlot.render(ui, &plot_data, &plot_config),
                ScatterPlot => plots::scatter::ScatterPlot.render(ui, &plot_data, &plot_config),
                Histogram => plots::HistogramPlot.render(ui, &plot_data, &plot_config),
                BoxPlot => plots::BoxPlotImpl.render(ui, &plot_data, &plot_config),
                
                // These will show "coming soon" messages for now
                HeatMap => plots::heatmap::HeatmapPlot.render(ui, &plot_data, &plot_config),
                ViolinPlot => plots::violin::ViolinPlot::new().render(ui, &plot_data, &plot_config),
                AnomalyDetection => plots::anomaly::AnomalyPlot.render(ui, &plot_data, &plot_config),
                CorrelationMatrix => plots::correlation::CorrelationPlot.render(ui, &plot_data, &plot_config),
                DistributionPlot => plots::distribution::DistributionPlot.render(ui, &plot_data, &plot_config),
                Scatter3D => plots::scatter3d::Scatter3dPlot.render(ui, &plot_data, &plot_config),
                Surface3D => plots::surface3d::Surface3dPlot.render(ui, &plot_data, &plot_config),
                ContourPlot => plots::contour::ContourPlot.render(ui, &plot_data, &plot_config),
                ParallelCoordinates => plots::parallel_coordinates::ParallelCoordinatesPlot.render(ui, &plot_data, &plot_config),
                RadarChart => plots::radar::RadarPlot.render(ui, &plot_data, &plot_config),
                SankeyDiagram => plots::sankey::SankeyPlot.render(ui, &plot_data, &plot_config),
                Treemap => plots::treemap::TreemapPlot.render(ui, &plot_data, &plot_config),
                SunburstChart => plots::sunburst::SunburstPlot.render(ui, &plot_data, &plot_config),
                NetworkGraph => plots::network::NetworkPlot.render(ui, &plot_data, &plot_config),
                GeoPlot => plots::geo::GeoPlot.render(ui, &plot_data, &plot_config),
                TimeAnalysis => plots::time_analysis::TimeAnalysisPlot.render(ui, &plot_data, &plot_config),
                CandlestickChart => plots::candlestick::CandlestickPlot.render(ui, &plot_data, &plot_config),
                StreamGraph => plots::stream::StreamPlot.render(ui, &plot_data, &plot_config),
                PolarPlot => plots::polar::PolarPlot.render(ui, &plot_data, &plot_config),
            }
        });
    }
}

fn format_data_type(dtype: &DataType) -> &'static str {
    use DataType::*;
    match dtype {
        Int8 | Int16 | Int32 | Int64 => "Integer",
        UInt8 | UInt16 | UInt32 | UInt64 => "Unsigned Int",
        Float16 | Float32 | Float64 => "Float",
        Utf8 | LargeUtf8 => "Text",
        Boolean => "Boolean",
        Date32 | Date64 => "Date",
        Timestamp(_, _) => "Timestamp",
        _ => "Other",
    }
} 
