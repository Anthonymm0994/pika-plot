//! Plot node implementation.

use pika_core::{
    node::{Node, NodeContext, Port, PortType, PortDirection, DataNode},
    types::NodeId,
    plots::{PlotConfig, PlotType, PlotDataConfig},
};
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Vec2};
use std::any::Any;

/// Plot node for data visualization
pub struct PlotNode {
    id: NodeId,
    position: Pos2,
    size: Vec2,
    
    // State
    plot_config: PlotConfig,
    has_data: bool,
    last_data: Option<arrow::record_batch::RecordBatch>,
    is_rendering: bool,
    error: Option<String>,
}

impl PlotNode {
    /// Create a new plot node
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            position: Pos2::new(100.0, 100.0),
            size: Vec2::new(300.0, 400.0),
            plot_config: PlotConfig::scatter("x".to_string(), "y".to_string()),
            has_data: false,
            last_data: None,
            is_rendering: false,
            error: None,
        }
    }
    
    /// Set the plot configuration
    pub fn set_config(&mut self, config: PlotConfig) {
        self.plot_config = config;
    }
    
    /// Set data for rendering
    pub fn set_data(&mut self, data: arrow::record_batch::RecordBatch) {
        self.last_data = Some(data);
        self.has_data = true;
        self.error = None;
    }
    
    /// Clear any error state
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

impl Node for PlotNode {
    fn id(&self) -> NodeId {
        self.id
    }
    
    fn name(&self) -> &str {
        "Plot"
    }
    
    fn node_type(&self) -> &str {
        "Plot"
    }
    
    fn position(&self) -> Pos2 {
        self.position
    }
    
    fn set_position(&mut self, pos: Pos2) {
        self.position = pos;
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
    
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: format!("{}_data", self.id),
                direction: PortDirection::Input,
                port_type: PortType::RecordBatch,
                position: Pos2::new(0.0, 40.0),
            },
            Port {
                id: format!("{}_config", self.id),
                direction: PortDirection::Input,
                port_type: PortType::PlotConfig,
                position: Pos2::new(0.0, 60.0),
            },
        ]
    }
    
    fn render(&self, ui: &mut egui::Ui, ctx: &NodeContext) -> Response {
        let rect = Rect::from_min_size(self.position, self.size);
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            let bg_color = if ctx.is_selected {
                Color32::from_gray(50)
            } else {
                Color32::from_gray(40)
            };
            painter.rect_filled(rect, 4.0, bg_color);
            
            // Border
            let border_color = if ctx.is_selected {
                Color32::from_rgb(100, 150, 255)
            } else if self.error.is_some() {
                Color32::from_rgb(255, 100, 100)
            } else if self.has_data {
                Color32::from_rgb(100, 200, 100)
            } else {
                Color32::from_gray(100)
            };
            painter.rect_stroke(rect, 4.0, Stroke::new(2.0, border_color));
            
            // Header
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 30.0));
            painter.rect_filled(header_rect, 4.0, Color32::from_gray(30));
            
            // Title with plot type
            let title = format!("📊 {} Plot", match self.plot_config.plot_type {
                PlotType::Scatter => "Scatter",
                PlotType::Line => "Line",
                PlotType::Bar => "Bar",
                PlotType::Histogram => "Histogram",
                PlotType::Heatmap => "Heatmap",
                _ => "Plot",
            });
            painter.text(
                header_rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::default(),
                Color32::from_gray(200),
            );
            
            // Content area for plot preview
            let content_rect = Rect::from_min_size(
                rect.min + Vec2::new(10.0, 40.0),
                Vec2::new(rect.width() - 20.0, rect.height() - 100.0),
            );
            
            // Render plot preview if we have data
            if let Some(data) = &self.last_data {
                ui.allocate_ui_at_rect(content_rect, |ui| {
                    crate::plots::render_plot(ui, &self.plot_config, data);
                });
            } else if let Some(error) = &self.error {
                painter.text(
                    content_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    error,
                    egui::FontId::default(),
                    Color32::from_rgb(255, 100, 100),
                );
            } else {
                painter.text(
                    content_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No data",
                    egui::FontId::default(),
                    Color32::from_gray(150),
                );
            }
            
            // Configuration info
            let config_y = rect.max.y - 50.0;
            let config_text = match &self.plot_config.specific {
                PlotDataConfig::ScatterConfig { x_column, y_column, .. } => {
                    format!("X: {}, Y: {}", x_column, y_column)
                }
                PlotDataConfig::LineConfig { x_column, y_column, .. } => {
                    format!("X: {}, Y: {}", x_column, y_column)
                }
                PlotDataConfig::BarConfig { category_column, value_column, .. } => {
                    format!("Cat: {}, Val: {}", category_column, value_column)
                }
                PlotDataConfig::HistogramConfig { column, .. } => {
                    format!("Column: {}", column)
                }
                _ => "Configure in properties".to_string(),
            };
            
            painter.text(
                Pos2::new(rect.center().x, config_y),
                egui::Align2::CENTER_CENTER,
                config_text,
                egui::FontId::default(),
                Color32::from_gray(180),
            );
            
            // Status indicator
            let status_pos = rect.min + Vec2::new(10.0, rect.height() - 20.0);
            let (status_text, status_color) = if self.error.is_some() {
                ("Error", Color32::from_rgb(255, 100, 100))
            } else if self.is_rendering {
                ("Rendering...", Color32::from_rgb(255, 200, 100))
            } else if self.has_data {
                ("Ready", Color32::from_rgb(100, 200, 100))
            } else {
                ("Waiting for data", Color32::from_gray(150))
            };
            
            painter.text(
                status_pos,
                egui::Align2::LEFT_CENTER,
                status_text,
                egui::FontId::default(),
                status_color,
            );
            
            // Port labels
            for port in self.ports() {
                let port_pos = rect.min + port.position;
                let label = match port.port_type {
                    PortType::RecordBatch => "Data",
                    PortType::PlotConfig => "Config",
                    _ => "",
                };
                
                painter.text(
                    port_pos + Vec2::new(15.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::default(),
                    Color32::from_gray(150),
                );
            }
        }
        
        response
    }
    
    fn update(&mut self, _ctx: &NodeContext) {
        // Update logic if needed
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DataNode for PlotNode {
    fn process_data(&mut self, _inputs: Vec<Option<&dyn Any>>) -> Option<Box<dyn Any>> {
        // Plots don't produce data, they consume it for visualization
        None
    }
} 