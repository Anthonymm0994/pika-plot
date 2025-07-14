//! Plot node implementation.

use crate::{state::AppState, canvas::CanvasElement};
use pika_core::{
    Node, NodeId, Point2, Size2, Port, PortDirection, PortType, Result,
    DataNode, PikaError, plots::PlotConfig, RenderData,
};
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
                    Err(PikaError::Validation(
                        format!("Invalid data type for port {}: expected table data", port_id)
                    ))
                }
            }
            _ => Err(PikaError::Validation(format!("Unknown port: {}", port_id))),
        }
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        // Plot nodes typically don't have outputs
        Err(PikaError::Validation(format!("Unknown output port: {}", port_id)))
    }
    
    fn is_ready(&self) -> bool {
        self.data.is_some()
    }
    
    fn execute(&mut self) -> Result<()> { Ok(()) }
    fn render_data(&mut self) -> Result<RenderData> { Ok(RenderData { data: vec![] }) }
    
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