use egui::{Color32, RichText, Ui, ScrollArea};
use crate::core::QueryResult;
use crate::ui::plots::{self, PlotType, PlotData, PlotPoint, Plot as PlotTrait};
use datafusion::arrow::datatypes::DataType;

#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub title: String,
    pub plot_type: Option<PlotType>,
    pub x_column: String,
    pub y_column: String,
    pub show_legend: bool,
    pub show_grid: bool,
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
        }
    }
}

pub struct PlotWindow {
    pub id: String,
    pub title: String,
    pub config: PlotConfig,
    pub is_config_open: bool,
    pub open: bool,
    data: Option<QueryResult>,
    source_query_id: Option<String>, // ID of the query window this plot is connected to
}

impl PlotWindow {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            config: PlotConfig::default(),
            is_config_open: true,
            open: true,
            data: None,
            source_query_id: None,
        }
    }

    pub fn update_data(&mut self, data: QueryResult) {
        self.data = Some(data);
    }

    pub fn set_source_query(&mut self, query_id: String) {
        self.source_query_id = Some(query_id);
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
                    ui.horizontal(|ui| {
                        ui.label("X Column:");
                        egui::ComboBox::new("x_column", "")
                            .selected_text(if self.config.x_column.is_empty() { "Select..." } else { &self.config.x_column })
                            .show_ui(ui, |ui| {
                                for (i, column) in data.columns.iter().enumerate() {
                                    let col_type = &data.column_types[i];
                                    let type_str = format_data_type(col_type);
                                    let label = format!("{} ({})", column, type_str);
                                    
                                    if ui.selectable_value(&mut self.config.x_column, column.clone(), label).clicked() {
                                        ui.close_menu();
                                    }
                                }
                            });
                        
                        ui.add_space(20.0);
                        
                        ui.label("Y Column:");
                        egui::ComboBox::new("y_column", "")
                            .selected_text(if self.config.y_column.is_empty() { "Select..." } else { &self.config.y_column })
                            .show_ui(ui, |ui| {
                                for (i, column) in data.columns.iter().enumerate() {
                                    let col_type = &data.column_types[i];
                                    let type_str = format_data_type(col_type);
                                    let label = format!("{} ({})", column, type_str);
                                    
                                    if ui.selectable_value(&mut self.config.y_column, column.clone(), label).clicked() {
                                        ui.close_menu();
                                    }
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
            });
        }

        // Plot rendering area
        ui.add_space(10.0);
        
        ScrollArea::vertical().show(ui, |ui| {
            if let Some(data) = &self.data {
                if let Some(plot_type) = &self.config.plot_type {
                    if let Ok(plot_data) = self.prepare_plot_data(data, plot_type) {
                        self.render_plot(ui, plot_type, plot_data);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Invalid data for selected plot type").size(14.0).color(Color32::from_gray(150)));
                        });
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Please select a plot type").size(14.0).color(Color32::from_gray(150)));
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("No data available").size(14.0).color(Color32::from_gray(150)));
                });
            }
        });
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
        
        let points = self.extract_plot_points(data)?;
        
        Ok(PlotData {
            points,
            title: self.config.title.clone(),
            x_label: self.config.x_column.clone(),
            y_label: self.config.y_column.clone(),
            show_legend: self.config.show_legend,
            show_grid: self.config.show_grid,
        })
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
                
                points.push(PlotPoint {
                    x: x_val,
                    y: y_val,
                    label: None,
                    color: None,
                    size: None,
                });
            }
        }
        
        Ok(points)
    }
    
    fn render_plot(&self, ui: &mut Ui, plot_type: &PlotType, plot_data: PlotData) {
        use PlotType::*;
        
        match plot_type {
            BarChart => plots::BarChartPlot.render(ui, &plot_data),
            LineChart => plots::LineChartPlot.render(ui, &plot_data),
            ScatterPlot => plots::ScatterPlotImpl.render(ui, &plot_data),
            Histogram => plots::HistogramPlot.render(ui, &plot_data),
            BoxPlot => plots::BoxPlotImpl.render(ui, &plot_data),
            
            // These will show "coming soon" messages for now
            HeatMap => plots::heatmap::HeatmapPlot.render(ui, &plot_data),
            ViolinPlot => plots::violin::ViolinPlot.render(ui, &plot_data),
            AnomalyDetection => plots::anomaly::AnomalyPlot.render(ui, &plot_data),
            CorrelationMatrix => plots::correlation::CorrelationPlot.render(ui, &plot_data),
            DistributionPlot => plots::distribution::DistributionPlot.render(ui, &plot_data),
            Scatter3D => plots::scatter3d::Scatter3dPlot.render(ui, &plot_data),
            Surface3D => plots::surface3d::Surface3dPlot.render(ui, &plot_data),
            ContourPlot => plots::contour::ContourPlot.render(ui, &plot_data),
            ParallelCoordinates => plots::parallel_coordinates::ParallelCoordinatesPlot.render(ui, &plot_data),
            RadarChart => plots::radar::RadarPlot.render(ui, &plot_data),
            SankeyDiagram => plots::sankey::SankeyPlot.render(ui, &plot_data),
            Treemap => plots::treemap::TreemapPlot.render(ui, &plot_data),
            SunburstChart => plots::sunburst::SunburstPlot.render(ui, &plot_data),
            NetworkGraph => plots::network::NetworkPlot.render(ui, &plot_data),
            GeoPlot => plots::geo::GeoPlot.render(ui, &plot_data),
            TimeAnalysis => plots::time_analysis::TimeAnalysisPlot.render(ui, &plot_data),
            CandlestickChart => plots::candlestick::CandlestickPlot.render(ui, &plot_data),
            StreamGraph => plots::stream::StreamPlot.render(ui, &plot_data),
            PolarPlot => plots::polar::PolarPlot.render(ui, &plot_data),
        }
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
