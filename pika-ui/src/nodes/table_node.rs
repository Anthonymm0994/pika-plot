//! Table node implementation for CSV data sources

use pika_core::{Node, NodeId, Point2, Size2, Port, PortDirection, PortType, NodeContext, Result, PikaError};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;
use super::TABLE_NODE_SIZE;

/// A node representing a table loaded from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNode {
    id: NodeId,
    name: String,
    position: Point2,
    size: Size2,
    
    // Table-specific data
    source_path: Option<PathBuf>,
    table_name: Option<String>,
    schema: Option<Vec<(String, String)>>, // Column name, type pairs
    row_count: usize,
    
    // Import options
    delimiter: char,
    has_header: bool,
    
    // UI state
    is_loading: bool,
    error_message: Option<String>,
}

impl TableNode {
    /// Create a new table node
    pub fn new(position: Point2) -> Self {
        Self {
            id: NodeId::new(),
            name: "Table".to_string(),
            position,
            size: TABLE_NODE_SIZE,
            source_path: None,
            table_name: None,
            schema: None,
            row_count: 0,
            delimiter: ',',
            has_header: true,
            is_loading: false,
            error_message: None,
        }
    }
    
    /// Set the CSV source file
    pub fn set_source(&mut self, path: PathBuf) {
        self.source_path = Some(path.clone());
        if let Some(file_name) = path.file_stem() {
            self.name = format!("Table: {}", file_name.to_string_lossy());
        }
    }
    
    /// Set the table name in DuckDB
    pub fn set_table_name(&mut self, name: String) {
        self.table_name = Some(name);
    }
    
    /// Update schema after import
    pub fn set_schema(&mut self, schema: Vec<(String, String)>, row_count: usize) {
        self.schema = Some(schema);
        self.row_count = row_count;
        self.is_loading = false;
        self.error_message = None;
    }
    
    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }
    
    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_loading = false;
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
        // Table nodes have no inputs - they are data sources
        &[]
    }
    
    fn output_ports(&self) -> &[Port] {
        // Static port definition - in a real implementation this would be stored
        static PORTS: &[Port] = &[];
        PORTS
    }
    
    fn render(&mut self, ui: &mut egui::Ui, ctx: &NodeContext) {
        use egui::{Color32, Stroke, vec2, Align2, FontId, RichText};
        
        let rect = egui::Rect::from_min_size(
            egui::pos2(self.position.x, self.position.y),
            vec2(self.size.width, self.size.height)
        );
        
        // Node background
        let fill_color = if ctx.is_selected {
            Color32::from_rgb(50, 50, 80)
        } else if ctx.is_hovered {
            Color32::from_rgb(40, 40, 60)
        } else {
            Color32::from_rgb(30, 30, 50)
        };
        
        ui.painter().rect_filled(rect, 5.0, fill_color);
        
        // Node border
        let border_color = if ctx.is_selected {
            Color32::from_rgb(100, 150, 255)
        } else {
            Color32::from_rgb(60, 60, 80)
        };
        ui.painter().rect_stroke(rect, 5.0, Stroke::new(2.0, border_color));
        
        // Header
        let header_rect = egui::Rect::from_min_size(rect.min, vec2(rect.width(), 30.0));
        ui.painter().rect_filled(header_rect, egui::Rounding::same(5.0), Color32::from_rgb(40, 40, 60));
        
        // Title
        ui.painter().text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            &self.name,
            FontId::proportional(14.0),
            Color32::WHITE,
        );
        
        // Content area
        let content_rect = egui::Rect::from_min_max(
            rect.min + vec2(10.0, 40.0),
            rect.max - vec2(10.0, 10.0),
        );
        
        // Show status
        if self.is_loading {
            ui.painter().text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                "Loading...",
                FontId::proportional(12.0),
                Color32::from_rgb(150, 150, 150),
            );
        } else if let Some(error) = &self.error_message {
            ui.painter().text(
                content_rect.left_top(),
                Align2::LEFT_TOP,
                error,
                FontId::proportional(11.0),
                Color32::from_rgb(255, 100, 100),
            );
        } else if let Some(path) = &self.source_path {
            // Show file info
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            
            ui.painter().text(
                content_rect.left_top(),
                Align2::LEFT_TOP,
                format!("File: {}", file_name),
                FontId::proportional(11.0),
                Color32::from_rgb(180, 180, 180),
            );
            
            if let Some(table_name) = &self.table_name {
                ui.painter().text(
                    content_rect.left_top() + vec2(0.0, 20.0),
                    Align2::LEFT_TOP,
                    format!("Table: {}", table_name),
                    FontId::proportional(11.0),
                    Color32::from_rgb(180, 180, 180),
                );
            }
            
            if self.row_count > 0 {
                ui.painter().text(
                    content_rect.left_top() + vec2(0.0, 40.0),
                    Align2::LEFT_TOP,
                    format!("{} rows", self.row_count),
                    FontId::proportional(11.0),
                    Color32::from_rgb(150, 150, 150),
                );
            }
            
            if let Some(schema) = &self.schema {
                ui.painter().text(
                    content_rect.left_top() + vec2(0.0, 60.0),
                    Align2::LEFT_TOP,
                    format!("{} columns", schema.len()),
                    FontId::proportional(11.0),
                    Color32::from_rgb(150, 150, 150),
                );
            }
        } else {
            // No file loaded
            ui.painter().text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                "Drop CSV file here",
                FontId::proportional(12.0),
                Color32::from_rgb(100, 100, 100),
            );
        }
        
        // Output port
        if self.table_name.is_some() {
            let port_pos = rect.right_center();
            ui.painter().circle_filled(port_pos, 6.0, Color32::from_rgb(100, 200, 100));
            ui.painter().circle_stroke(port_pos, 6.0, Stroke::new(2.0, Color32::from_rgb(150, 255, 150)));
        }
    }
    
    fn accept_input(&mut self, _port_id: &str, _data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        // Table nodes don't accept inputs
        Err(PikaError::InvalidOperation("Table nodes have no input ports".to_string()))
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        if port_id != "table" {
            return Err(PikaError::InvalidPort(port_id.to_string()));
        }
        
        if let Some(table_name) = &self.table_name {
            Ok(Some(Arc::new(table_name.clone()) as Arc<dyn Any + Send + Sync>))
        } else {
            Ok(None)
        }
    }
    
    fn is_ready(&self) -> bool {
        // Table nodes are ready when they have a table name
        self.table_name.is_some()
    }
    
    fn execute(&mut self) -> Result<()> {
        // Table nodes don't execute - they're loaded via events
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_node_creation() {
        let node = TableNode::new(Point2::new(100.0, 100.0));
        assert_eq!(node.name(), "Table");
        assert_eq!(node.position(), Point2::new(100.0, 100.0));
        assert!(!node.is_ready());
    }
    
    #[test]
    fn test_table_node_source() {
        let mut node = TableNode::new(Point2::new(0.0, 0.0));
        node.set_source(PathBuf::from("test_data.csv"));
        assert_eq!(node.name(), "Table: test_data");
        assert_eq!(node.source_path, Some(PathBuf::from("test_data.csv")));
    }
    
    #[test]
    fn test_table_node_ready_state() {
        let mut node = TableNode::new(Point2::new(0.0, 0.0));
        assert!(!node.is_ready());
        
        node.set_table_name("my_table".to_string());
        assert!(node.is_ready());
    }
} 