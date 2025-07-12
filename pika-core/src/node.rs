//! Node trait and port system for the canvas

use crate::{NodeId, Point2, Size2, Result};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

/// Direction of data flow for a port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortDirection {
    /// Input port - receives data
    Input,
    /// Output port - provides data
    Output,
}

/// Type of data that can flow through a port
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// Table reference in DuckDB
    Table(String),
    /// Arrow RecordBatch data
    RecordBatch,
    /// Plot configuration
    PlotConfig,
    /// Generic data type
    Any,
}

/// A connection point on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Unique identifier for the port
    pub id: String,
    /// Display name
    pub name: String,
    /// Direction of data flow
    pub direction: PortDirection,
    /// Type of data this port handles
    pub data_type: PortType,
    /// Position relative to node center
    pub relative_position: Point2,
}

impl Port {
    /// Create a new port
    pub fn new(id: impl Into<String>, name: impl Into<String>, direction: PortDirection, data_type: PortType) -> Self {
        let relative_position = match direction {
            PortDirection::Input => Point2::new(-50.0, 0.0),
            PortDirection::Output => Point2::new(50.0, 0.0),
        };
        
        Self {
            id: id.into(),
            name: name.into(),
            direction,
            data_type,
            relative_position,
        }
    }
    
    /// Get the absolute position of the port given the node position
    pub fn absolute_position(&self, node_position: Point2) -> Point2 {
        Point2::new(
            node_position.x + self.relative_position.x,
            node_position.y + self.relative_position.y,
        )
    }
}

/// Context provided to nodes for rendering and computation
pub struct NodeContext {
    /// Current frame time
    pub frame_time: f32,
    /// Whether the node is selected
    pub is_selected: bool,
    /// Whether the node is being hovered
    pub is_hovered: bool,
    /// Scale factor for high DPI displays
    pub scale_factor: f32,
}

/// Base trait for all canvas nodes
pub trait Node: Send + Sync {
    /// Get the unique identifier for this node
    fn id(&self) -> NodeId;
    
    /// Get the display name of the node
    fn name(&self) -> &str;
    
    /// Get the current position on the canvas
    fn position(&self) -> Point2;
    
    /// Set the position on the canvas
    fn set_position(&mut self, position: Point2);
    
    /// Get the size of the node
    fn size(&self) -> Size2;
    
    /// Get all input ports
    fn input_ports(&self) -> &[Port];
    
    /// Get all output ports  
    fn output_ports(&self) -> &[Port];
    
    /// Render the node UI
    fn render(&mut self, ui: &mut egui::Ui, ctx: &NodeContext);
    
    /// Handle incoming data on an input port
    fn accept_input(&mut self, port_id: &str, data: Arc<dyn Any + Send + Sync>) -> Result<()>;
    
    /// Get output data for a specific port
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>>;
    
    /// Check if the node is ready to compute (all required inputs connected)
    fn is_ready(&self) -> bool;
    
    /// Execute the node's computation
    fn execute(&mut self) -> Result<()>;
    
    /// Get a type identifier for serialization
    fn type_name(&self) -> &'static str;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Helper trait for nodes that produce Arrow data
pub trait DataNode: Node {
    /// Get the output schema as column info
    fn output_columns(&self) -> Option<Vec<(String, String)>>;
    
    /// Get the output data as an opaque handle
    fn output_data(&self) -> Option<Arc<dyn Any + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_port_creation() {
        let port = Port::new("test_port", "Test Port", PortDirection::Input, PortType::RecordBatch);
        assert_eq!(port.id, "test_port");
        assert_eq!(port.name, "Test Port");
        assert_eq!(port.direction, PortDirection::Input);
        assert_eq!(port.relative_position.x, -50.0);
    }
    
    #[test]
    fn test_port_absolute_position() {
        let port = Port::new("out", "Output", PortDirection::Output, PortType::Table("test".to_string()));
        let node_pos = Point2::new(100.0, 200.0);
        let abs_pos = port.absolute_position(node_pos);
        assert_eq!(abs_pos.x, 150.0); // 100 + 50
        assert_eq!(abs_pos.y, 200.0);
    }
} 