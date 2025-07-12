//! Plot node implementation.

use pika_core::{
    Node, NodeId, Point2, Size2, Port, PortDirection, PortType, NodeContext, Result,
    DataNode, PikaError, plots::{PlotConfig, PlotDataConfig},
};
use egui::{Ui, Color32, Stroke, vec2, Align2, FontId};
use std::sync::Arc;
use std::any::Any;

/// A node that renders plots
#[derive(Debug, Clone)]
pub struct PlotNode {
    id: NodeId,
    name: String,
    position: Point2,
    size: Size2,
    config: PlotConfig,
    input_ports: Vec<Port>,
    output_ports: Vec<Port>,
    data: Option<Arc<dyn Any + Send + Sync>>,
}

impl PlotNode {
    pub fn new(id: NodeId) -> Self {
        let input_ports = vec![
            Port::new("data", "Data", PortDirection::Input, PortType::RecordBatch),
            Port::new("config", "Config", PortDirection::Input, PortType::PlotConfig),
        ];
        
        let output_ports = vec![
            // Plot nodes typically don't have outputs, but we could add image output
        ];
        
        PlotNode {
            id,
            name: "Plot".to_string(),
            position: Point2::new(0.0, 0.0),
            size: Size2::new(200.0, 150.0),
            config: PlotConfig::scatter("x".to_string(), "y".to_string()),
            input_ports,
            output_ports,
            data: None,
        }
    }
    
    pub fn with_config(mut self, config: PlotConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn config(&self) -> &PlotConfig {
        &self.config
    }
    
    pub fn set_config(&mut self, config: PlotConfig) {
        self.config = config;
    }
}

impl Node for PlotNode {
    fn id(&self) -> NodeId {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn position(&self) -> Point2 {
        self.position
    }
    
    fn set_position(&mut self, position: Point2) {
        self.position = position;
    }
    
    fn size(&self) -> Size2 {
        self.size
    }
    
    fn input_ports(&self) -> &[Port] {
        &self.input_ports
    }
    
    fn output_ports(&self) -> &[Port] {
        &self.output_ports
    }
    
    fn render(&mut self, ui: &mut Ui, ctx: &NodeContext) {
        // Node background
        let rect = ui.available_rect_before_wrap();
        let color = if ctx.is_selected {
            Color32::from_rgb(100, 150, 200)
        } else if ctx.is_hovered {
            Color32::from_rgb(80, 80, 90)
        } else {
            Color32::from_rgb(60, 60, 70)
        };
        
        ui.painter().rect_filled(rect, 4.0, color);
        ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::WHITE));
        
        // Title
        ui.painter().text(
            rect.center_top() + vec2(0.0, 10.0),
            Align2::CENTER_TOP,
            &self.name,
            FontId::default(),
            Color32::WHITE,
        );
        
        // Plot type
        let plot_type = match &self.config.specific {
            PlotDataConfig::ScatterConfig { .. } => "Scatter Plot",
            PlotDataConfig::LineConfig { .. } => "Line Plot",
            PlotDataConfig::BarConfig { .. } => "Bar Chart",
            PlotDataConfig::HistogramConfig { .. } => "Histogram",
            PlotDataConfig::HeatmapConfig { .. } => "Heatmap",
            PlotDataConfig::BoxPlotConfig { .. } => "Box Plot",
            _ => "Plot",
        };
        
        ui.painter().text(
            rect.center() + vec2(0.0, 10.0),
            Align2::CENTER_CENTER,
            plot_type,
            FontId::default(),
            Color32::LIGHT_GRAY,
        );
        
        // Status indicator
        let status_color = if self.is_ready() {
            Color32::GREEN
        } else {
            Color32::RED
        };
        
        ui.painter().circle_filled(
            rect.right_bottom() + vec2(-10.0, -10.0),
            4.0,
            status_color,
        );
    }
    
    fn accept_input(&mut self, port_id: &str, data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        match port_id {
            "data" => {
                self.data = Some(data);
                Ok(())
            }
            "config" => {
                if let Some(config) = data.downcast_ref::<PlotConfig>() {
                    self.config = config.clone();
                    Ok(())
                } else {
                    Err(PikaError::InvalidDataType {
                        expected: "PlotConfig".to_string(),
                        found: "Unknown".to_string(),
                    })
                }
            }
            _ => Err(PikaError::InvalidPort(format!("Unknown port: {}", port_id))),
        }
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        // Plot nodes typically don't have outputs
        Err(PikaError::InvalidPort(format!("Unknown output port: {}", port_id)))
    }
    
    fn is_ready(&self) -> bool {
        self.data.is_some()
    }
    
    fn execute(&mut self) -> Result<()> {
        // Plot execution would happen here
        // For now, just validate we have data
        if self.data.is_none() {
            return Err(PikaError::MissingField("data".to_string()));
        }
        Ok(())
    }
    
    fn type_name(&self) -> &'static str {
        "PlotNode"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DataNode for PlotNode {
    fn output_columns(&self) -> Option<Vec<(String, String)>> {
        // Plot nodes don't output tabular data
        None
    }
    
    fn output_data(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        // Plot nodes don't output data
        None
    }
} 