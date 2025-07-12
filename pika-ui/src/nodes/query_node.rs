//! Query node implementation for SQL queries

use pika_core::{Node, NodeId, Point2, Size2, Port, PortDirection, PortType, NodeContext, Result, PikaError};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
// use arrow::datatypes::SchemaRef;
// use arrow::record_batch::RecordBatch;
use super::QUERY_NODE_SIZE;

/// A node representing a SQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryNode {
    id: NodeId,
    name: String,
    position: Point2,
    size: Size2,
    
    // Query-specific data
    sql: String,
    input_tables: Vec<String>,
    output_schema: Option<SchemaRef>,
    cached_result: Option<Arc<RecordBatch>>,
    
    // UI state
    is_executing: bool,
    error_message: Option<String>,
    execution_time: Option<std::time::Duration>,
}

impl QueryNode {
    /// Create a new query node
    pub fn new(position: Point2) -> Self {
        Self {
            id: NodeId::new(),
            name: "Query".to_string(),
            position,
            size: QUERY_NODE_SIZE,
            sql: String::new(),
            input_tables: Vec::new(),
            output_schema: None,
            cached_result: None,
            is_executing: false,
            error_message: None,
            execution_time: None,
        }
    }
    
    /// Set the SQL query
    pub fn set_sql(&mut self, sql: String) {
        // Extract a name from the query if possible
        if let Some(first_line) = sql.lines().next() {
            let truncated = if first_line.len() > 30 {
                format!("{}...", &first_line[..27])
            } else {
                first_line.to_string()
            };
            self.name = format!("Query: {}", truncated);
        }
    }
    
    /// Add an input table connection
    pub fn add_input_table(&mut self, table_name: String) {
        if !self.input_tables.contains(&table_name) {
            self.input_tables.push(table_name);
        }
    }
    
    /// Remove an input table connection
    pub fn remove_input_table(&mut self, table_name: &str) {
        self.input_tables.retain(|t| t != table_name);
    }
    
    /// Set the query result
    pub fn set_result(&mut self, result: Arc<RecordBatch>, execution_time: std::time::Duration) {
        self.output_schema = Some(result.schema());
        self.cached_result = Some(result);
        self.execution_time = Some(execution_time);
        self.is_executing = false;
        self.error_message = None;
    }
    
    /// Set executing state
    pub fn set_executing(&mut self, executing: bool) {
        self.is_executing = executing;
        if executing {
            self.error_message = None;
        }
    }
    
    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_executing = false;
    }
}

impl Node for QueryNode {
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
        // Dynamic ports based on input tables - simplified for now
        static PORTS: &[Port] = &[];
        PORTS
    }
    
    fn output_ports(&self) -> &[Port] {
        // Static port definition
        static PORTS: &[Port] = &[];
        PORTS
    }
    
    fn render(&mut self, ui: &mut egui::Ui, ctx: &NodeContext) {
        use egui::{Color32, Stroke, vec2, Align2, FontId, TextEdit};
        
        let rect = egui::Rect::from_min_size(
            egui::pos2(self.position.x, self.position.y),
            vec2(self.size.width, self.size.height)
        );
        
        // Node background
        let fill_color = if ctx.is_selected {
            Color32::from_rgb(50, 60, 50)
        } else if ctx.is_hovered {
            Color32::from_rgb(40, 50, 40)
        } else {
            Color32::from_rgb(30, 40, 30)
        };
        
        ui.painter().rect_filled(rect, 5.0, fill_color);
        
        // Node border
        let border_color = if ctx.is_selected {
            Color32::from_rgb(100, 255, 150)
        } else if self.is_executing {
            Color32::from_rgb(255, 200, 100)
        } else if self.error_message.is_some() {
            Color32::from_rgb(255, 100, 100)
        } else {
            Color32::from_rgb(60, 80, 60)
        };
        ui.painter().rect_stroke(rect, 5.0, Stroke::new(2.0, border_color));
        
        // Header
        let header_rect = egui::Rect::from_min_size(rect.min, vec2(rect.width(), 30.0));
        ui.painter().rect_filled(header_rect, egui::Rounding::same(5.0), Color32::from_rgb(40, 50, 40));
        
        // Title
        ui.painter().text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            &self.name,
            FontId::proportional(14.0),
            Color32::WHITE,
        );
        
        // SQL editor area
        let editor_rect = egui::Rect::from_min_max(
            rect.min + vec2(10.0, 40.0),
            rect.max - vec2(10.0, 50.0),
        );
        
        // Create a child UI for the SQL editor
        ui.allocate_ui_at_rect(editor_rect, |ui| {
            egui::ScrollArea::vertical()
                .max_height(editor_rect.height())
                .show(ui, |ui| {
                    let response = ui.add(
                        TextEdit::multiline(&mut self.sql)
                            .code_editor()
                            .desired_width(editor_rect.width())
                            .desired_rows(6)
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    if response.changed() {
                        // Update node name when SQL changes
                        if let Some(first_line) = self.sql.lines().next() {
                            let truncated = if first_line.len() > 30 {
                                format!("{}...", &first_line[..27])
                            } else {
                                first_line.to_string()
                            };
                            self.name = format!("Query: {}", truncated);
                        }
                    }
                });
        });
        
        // Status bar
        let status_rect = egui::Rect::from_min_size(
            rect.min + vec2(10.0, rect.height() - 40.0),
            vec2(rect.width() - 20.0, 30.0),
        );
        
        if self.is_executing {
            ui.painter().text(
                status_rect.left_center(),
                Align2::LEFT_CENTER,
                "Executing...",
                FontId::proportional(11.0),
                Color32::from_rgb(255, 200, 100),
            );
        } else if let Some(error) = &self.error_message {
            ui.painter().text(
                status_rect.left_top(),
                Align2::LEFT_TOP,
                error,
                FontId::proportional(10.0),
                Color32::from_rgb(255, 100, 100),
            );
        } else if let Some(result) = &self.cached_result {
            let status_text = format!(
                "{} rows × {} cols",
                result.num_rows(),
                result.num_columns()
            );
            ui.painter().text(
                status_rect.left_center(),
                Align2::LEFT_CENTER,
                status_text,
                FontId::proportional(11.0),
                Color32::from_rgb(150, 255, 150),
            );
            
            if let Some(time) = &self.execution_time {
                ui.painter().text(
                    status_rect.right_center(),
                    Align2::RIGHT_CENTER,
                    format!("{:.2}ms", time.as_secs_f64() * 1000.0),
                    FontId::proportional(11.0),
                    Color32::from_rgb(150, 150, 150),
                );
            }
        }
        
        // Input ports (left side)
        for (i, table) in self.input_tables.iter().enumerate() {
            let port_y = rect.top() + 50.0 + (i as f32 * 30.0);
            let port_pos = egui::pos2(rect.left(), port_y);
            ui.painter().circle_filled(port_pos, 6.0, Color32::from_rgb(100, 150, 200));
            ui.painter().circle_stroke(port_pos, 6.0, Stroke::new(2.0, Color32::from_rgb(150, 200, 255)));
        }
        
        // Output port (right side)
        if self.cached_result.is_some() {
            let port_pos = rect.right_center();
            ui.painter().circle_filled(port_pos, 6.0, Color32::from_rgb(100, 200, 100));
            ui.painter().circle_stroke(port_pos, 6.0, Stroke::new(2.0, Color32::from_rgb(150, 255, 150)));
        }
    }
    
    fn accept_input(&mut self, port_id: &str, data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        // Accept table names as input
        if let Some(table_name) = data.downcast_ref::<String>() {
            self.add_input_table(table_name.clone());
            Ok(())
        } else {
            Err(PikaError::InvalidOperation("Query nodes only accept table names".to_string()))
        }
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        if port_id != "result" {
            return Err(PikaError::InvalidPort(port_id.to_string()));
        }
        
        if let Some(result) = &self.cached_result {
            Ok(Some(result.clone() as Arc<dyn Any + Send + Sync>))
        } else {
            Ok(None)
        }
    }
    
    fn is_ready(&self) -> bool {
        // Query is ready when it has SQL and all required tables
        !self.sql.trim().is_empty()
    }
    
    fn execute(&mut self) -> Result<()> {
        // Execution is triggered via events
        self.set_executing(true);
        Ok(())
    }
    
    fn type_name(&self) -> &'static str {
        "QueryNode"
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
    fn test_query_node_creation() {
        let node = QueryNode::new(Point2::new(100.0, 100.0));
        assert_eq!(node.name(), "Query");
        assert_eq!(node.position(), Point2::new(100.0, 100.0));
        assert!(!node.is_ready());
    }
    
    #[test]
    fn test_query_node_sql() {
        let mut node = QueryNode::new(Point2::new(0.0, 0.0));
        node.set_sql("SELECT * FROM users".to_string());
        assert_eq!(node.name(), "Query: SELECT * FROM users");
        assert!(node.is_ready());
    }
    
    #[test]
    fn test_query_node_inputs() {
        let mut node = QueryNode::new(Point2::new(0.0, 0.0));
        node.add_input_table("users".to_string());
        node.add_input_table("orders".to_string());
        assert_eq!(node.input_tables.len(), 2);
        
        node.remove_input_table("users");
        assert_eq!(node.input_tables.len(), 1);
        assert_eq!(node.input_tables[0], "orders");
    }
} 