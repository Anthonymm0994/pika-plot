//! Table node implementation for CSV data sources

use pika_core::{Node, NodeId, Point2, Size2, Port, PortDirection, PortType, NodeContext, Result, DataNode, PikaError, types::TableInfo};
use egui::{Ui, Color32, Stroke, vec2, Align2, FontId};
use std::sync::Arc;
use std::any::Any;

/// A node that represents a data table
#[derive(Debug, Clone)]
pub struct TableNode {
    pub id: NodeId,
    pub name: String,
    pub position: Point2,
    pub size: Size2,
    pub table_info: TableInfo,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
}

impl TableNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            name: "Table".to_string(),
            position: Point2::new(0.0, 0.0),
            size: Size2::new(200.0, 100.0),
            table_info: TableInfo {
                name: "table".to_string(),
                source_path: None,
                row_count: None,
                columns: vec![],
            },
            input_ports: vec![],
            output_ports: vec![
                Port::new("output", "Data", PortDirection::Output, PortType::RecordBatch),
            ],
        }
    }
    
    pub fn set_source_path(&mut self, path: std::path::PathBuf) {
        self.table_info.source_path = Some(path);
    }
    
    pub fn set_table_name(&mut self, name: String) {
        self.table_info.name = name;
    }
    
    pub fn set_row_count(&mut self, row_count: Option<usize>) {
        self.table_info.row_count = row_count;
    }
}

impl Node for TableNode {
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
    
    fn accept_input(&mut self, _port_id: &str, _data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        // Table nodes typically don't accept input
        Err(PikaError::InvalidPort("Table nodes don't accept input".to_string()))
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        match port_id {
            "output" => {
                // Would return table data here
                Ok(None)
            }
            _ => Err(PikaError::InvalidPort(format!("Unknown output port: {}", port_id))),
        }
    }
    
    fn is_ready(&self) -> bool {
        !self.table_info.name.is_empty()
    }
    
    fn execute(&mut self) -> Result<()> {
        // Table execution would happen here
        Ok(())
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
    
    fn render(&mut self, ui: &mut Ui, _context: &NodeContext) {
        let rect = egui::Rect::from_min_size(
            egui::pos2(self.position.x, self.position.y),
            egui::vec2(self.size.width, self.size.height),
        );
        
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        // Draw node background
        ui.painter().rect_filled(
            rect,
            4.0,
            Color32::from_rgb(70, 70, 80),
        );
        
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::WHITE),
        );
        
        // Draw node title
        ui.painter().text(
            rect.center_top() + vec2(0.0, 10.0),
            Align2::CENTER_TOP,
            &self.name,
            FontId::default(),
            Color32::WHITE,
        );
        
        // Draw table info
        let table_name = &self.table_info.name;
        ui.painter().text(
            rect.center() + vec2(0.0, 10.0),
            Align2::CENTER_CENTER,
            table_name,
            FontId::default(),
            Color32::WHITE,
        );
        
        if let Some(row_count) = self.table_info.row_count {
            ui.painter().text(
                rect.center_bottom() + vec2(0.0, -10.0),
                Align2::CENTER_BOTTOM,
                format!("{} rows", row_count),
                FontId::default(),
                Color32::LIGHT_GRAY,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_node_creation() {
        let node = TableNode::new(NodeId::new());
        assert_eq!(node.name(), "Table");
        assert_eq!(node.position(), Point2::new(0.0, 0.0));
        assert!(!node.is_ready());
    }
    
    #[test]
    fn test_table_node_source() {
        let mut node = TableNode::new(NodeId::new());
        node.set_source_path(std::path::PathBuf::from("test_data.csv"));
        assert_eq!(node.name(), "Table");
        assert_eq!(node.table_info.source_path, Some(std::path::PathBuf::from("test_data.csv")));
    }
    
    #[test]
    fn test_table_node_ready_state() {
        let mut node = TableNode::new(NodeId::new());
        assert!(!node.is_ready());
        
        node.set_table_name("my_table".to_string());
        assert!(node.is_ready());
    }
} 