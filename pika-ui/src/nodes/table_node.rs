//! Table node implementation.

use pika_core::{Node, NodeId, Port, NodeContext, Result, PikaError, types::TableInfo};
use std::any::Any;
use std::sync::Arc;
use pika_core::{Point2, Size2, RenderData};

pub struct TableNode {
    id: NodeId,
    table_info: TableInfo,
    position: Point2,
    size: Size2,
}

impl TableNode {
    pub fn new(id: NodeId, table_info: TableInfo) -> Self {
        Self {
            id,
            table_info,
            position: Point2::new(0.0, 0.0),
            size: Size2::new(200.0, 150.0),
        }
    }
}

impl Node for TableNode {
    fn id(&self) -> NodeId {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.table_info.name
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
        &[]
    }
    
    fn output_ports(&self) -> &[Port] {
        &[]
    }
    
    fn accept_input(&mut self, _port_id: &str, _data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        Ok(())
    }
    
    fn get_output(&self, _port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        Ok(None)
    }
    
    fn is_ready(&self) -> bool {
        true
    }
    
    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn render_data(&mut self) -> Result<RenderData> {
        Ok(RenderData { data: vec![] })
    }
    
    fn type_name(&self) -> &'static str {
        "TableNode"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
} 