//! Application state management.

use pika_core::{
    plots::{PlotConfig, PlotType},
    types::{TableInfo, QueryResult},
    NodeId,
};
use std::collections::HashMap;
use egui::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Canvas,
    Notebook,
    FileConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Pan,
    Rectangle,
    Circle,
    Line,
    Draw,
    Text,
}

#[derive(Debug, Clone)]
pub struct DataNode {
    pub id: NodeId,
    pub table_info: TableInfo,
    pub position: egui::Vec2,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub id: String,
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    DataFlow,
    Transform,
    Join,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

/// Canvas state for zoom and pan
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub zoom: f32,
    pub show_grid: bool,
    pub pan_offset: egui::Vec2,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            show_grid: true,
            pan_offset: egui::Vec2::ZERO,
        }
    }
}

// Canvas node for visualization
#[derive(Debug, Clone)]
pub struct CanvasNode {
    pub id: NodeId,
    pub position: egui::Vec2,
    pub size: egui::Vec2,
    pub node_type: CanvasNodeType,
}

#[derive(Debug, Clone)]
pub enum CanvasNodeType {
    Table { table_info: TableInfo },
    Plot { plot_type: String },
}

/// Query window state similar to Pebble
#[derive(Debug, Clone)]
pub struct QueryWindow {
    pub id: NodeId,
    pub title: String,
    pub query: String,
    pub result: Option<QueryWindowResult>,
    pub error: Option<String>,
    pub page: usize,
    pub page_size: usize,
    pub is_open: bool,
}

/// Query results for window display (different from pika_core::types::QueryResult)
#[derive(Debug, Clone)]
pub struct QueryWindowResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

/// Main application state
#[derive(Debug)]
pub struct AppState {
    pub view_mode: ViewMode,
    pub tool_mode: ToolMode,
    pub selected_node: Option<NodeId>,
    pub data_nodes: Vec<DataNode>,
    pub connections: Vec<NodeConnection>,
    pub query_results: HashMap<NodeId, QueryResult>,
    pub theme: Theme,
    pub show_import_dialog: bool,
    pub show_export_dialog: bool,
    pub notification: Option<String>,
    pub plot_configs: HashMap<NodeId, PlotConfig>,
    pub query_windows: HashMap<NodeId, QueryWindow>,
    pub canvas_state: CanvasState,
    pub tables: Vec<TableInfo>, // Convenience alias for data sources
    pub canvas_nodes: HashMap<NodeId, CanvasNode>,
    /// Query text for each table node
    pub node_queries: HashMap<NodeId, String>,
    /// Data preview for each node
    pub node_data: HashMap<NodeId, NodeDataPreview>,
}

/// Data preview for canvas nodes
#[derive(Debug, Clone)]
pub struct NodeDataPreview {
    pub headers: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<String>>>,
    pub total_rows: Option<usize>,
    pub current_page: usize,
    pub page_size: usize,
}

impl Default for NodeDataPreview {
    fn default() -> Self {
        Self {
            headers: None,
            rows: None,
            total_rows: None,
            current_page: 0,
            page_size: 25,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Canvas,
            tool_mode: ToolMode::Select,
            selected_node: None,
            data_nodes: Vec::new(),
            connections: Vec::new(),
            query_results: HashMap::new(),
            theme: Theme::Dark,
            show_import_dialog: false,
            show_export_dialog: false,
            notification: None,
            plot_configs: HashMap::new(),
            query_windows: HashMap::new(),
            canvas_state: CanvasState::default(),
            tables: vec![],
            canvas_nodes: HashMap::new(),
            node_queries: HashMap::new(),
            node_data: HashMap::new(),
        }
    }
    
    /// Update convenience tables list
    pub fn update_tables(&mut self) {
        self.tables = self.data_nodes.iter()
            .map(|node| node.table_info.clone())
            .collect();
    }
    
    pub fn add_data_node(&mut self, table: TableInfo) -> NodeId {
        let node_id = NodeId(uuid::Uuid::new_v4());
        let node = DataNode {
            id: node_id,
            table_info: table.clone(),
            position: egui::Vec2::new(100.0, 100.0),
            selected: false,
        };
        self.data_nodes.push(node);
        
        // Don't automatically add to canvas - let user do that
        // This fixes the issue where first node always gets added
        
        self.update_tables();
        node_id
    }
    
    /// Load data preview for a table node
    pub fn load_data_preview(&mut self, node_id: NodeId) {
        // Get table info first to avoid borrow issues
        let table_info_opt = self.get_canvas_node(node_id)
            .and_then(|node| match &node.node_type {
                CanvasNodeType::Table { table_info } => Some(table_info.clone()),
                _ => None,
            });
            
        if let Some(table_info) = table_info_opt {
            // Get the default query
            let query = self.node_queries.get(&node_id)
                .cloned()
                .unwrap_or_else(|| format!("SELECT * FROM {} LIMIT 10", table_info.name));
            
            // Generate mock data based on column types
            let headers: Vec<String> = table_info.columns.iter().map(|c| c.name.clone()).collect();
            let mut rows = Vec::new();
            
            // Generate 10 rows of mock data
            for i in 0..10 {
                let mut row = Vec::new();
                for col in &table_info.columns {
                    let value = match col.data_type.as_str() {
                        "INTEGER" | "integer" => (i + 1).to_string(),
                        "TEXT" | "text" | "VARCHAR" | "varchar" => {
                            match col.name.to_lowercase().as_str() {
                                "first_name" => ["John", "Jane", "Bob", "Alice", "Charlie", "David", "Emma", "Frank", "Grace", "Henry"][i % 10].to_string(),
                                "last_name" => ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"][i % 10].to_string(),
                                "name" => format!("Sample {}", i + 1),
                                "email" => format!("user{}@example.com", i + 1),
                                "gender" => if i % 2 == 0 { "Male" } else { "Female" }.to_string(),
                                "ip_address" => format!("192.168.1.{}", i + 1),
                                _ => format!("{} {}", col.name, i + 1),
                            }
                        }
                        "REAL" | "real" | "FLOAT" | "float" | "DOUBLE" | "double" => {
                            format!("{:.2}", (i + 1) as f64 * 10.5)
                        }
                        "BOOLEAN" | "boolean" | "BOOL" | "bool" => {
                            if i % 2 == 0 { "true" } else { "false" }.to_string()
                        }
                        _ => format!("Data {}", i + 1),
                    };
                    row.push(value);
                }
                rows.push(row);
            }
            
            let preview = NodeDataPreview {
                headers: Some(headers),
                rows: Some(rows),
                total_rows: Some(table_info.row_count.unwrap_or(100)), // Use actual row count if available
                current_page: 0,
                page_size: 25, // Default page size like Pebble
            };
            
            self.node_data.insert(node_id, preview);
        }
    }
    
    /// Execute query for a table node
    pub fn execute_node_query(&mut self, node_id: NodeId) {
        // Get the query and table name
        if let Some(query) = self.node_queries.get(&node_id).cloned() {
            // Execute full query for plots
            self.execute_full_query(node_id, query);
        } else if let Some(canvas_node) = self.get_canvas_node(node_id) {
            if let CanvasNodeType::Table { table_info } = &canvas_node.node_type {
                let default_query = format!("SELECT * FROM '{}'", table_info.name);
                self.execute_full_query(node_id, default_query);
            }
        }
    }
    
    /// Execute query with pagination for table display
    pub fn execute_node_query_with_pagination(&mut self, node_id: NodeId) {
        // Get the query and table name
        let (query, _table_name) = if let Some(query) = self.node_queries.get(&node_id).cloned() {
            let table_name = self.get_canvas_node(node_id)
                .and_then(|n| match &n.node_type {
                    CanvasNodeType::Table { table_info } => Some(table_info.name.clone()),
                    _ => None
                })
                .unwrap_or_default();
            (query, table_name)
        } else if let Some(canvas_node) = self.get_canvas_node(node_id) {
            if let CanvasNodeType::Table { table_info } = &canvas_node.node_type {
                (format!("SELECT * FROM '{}'", table_info.name), table_info.name.clone())
            } else {
                return;
            }
        } else {
            return;
        };
        
        // Get current pagination state
        let (page, page_size) = self.node_data.get(&node_id)
            .map(|d| (d.current_page, d.page_size))
            .unwrap_or((0, 25));
        
        // First, get total count (simulate)
        let total_rows = 200; // TODO: Execute COUNT query
        
        // Add LIMIT and OFFSET for table display
        let _paginated_query = format!("{} LIMIT {} OFFSET {}", query, page_size, page * page_size);
        
        // TODO: Execute the paginated query here
        // For now, generate mock data
        let preview = NodeDataPreview {
            headers: Some(vec!["id".to_string(), "name".to_string(), "value".to_string()]),
            rows: Some((0..page_size.min(total_rows - page * page_size)).map(|i| {
                let row_num = page * page_size + i + 1;
                vec![
                    row_num.to_string(),
                    format!("Row {}", row_num),
                    (row_num * 100).to_string(),
                ]
            }).collect()),
            total_rows: Some(total_rows),
            current_page: page,
            page_size,
        };
        
        self.node_data.insert(node_id, preview);
        
        // Also execute full query for any connected plots
        self.execute_full_query(node_id, query);
    }
    
    /// Execute full query without pagination (for plots)
    fn execute_full_query(&mut self, node_id: NodeId, _query: String) {
        // This would execute the full query and update connected plots
        self.update_connected_plots(node_id);
    }
    
    /// Update plots connected to a data source
    fn update_connected_plots(&mut self, source_node_id: NodeId) {
        let connected_plots: Vec<NodeId> = self.connections.iter()
            .filter(|conn| conn.from == source_node_id)
            .map(|conn| conn.to)
            .collect();
            
        for plot_id in connected_plots {
            // Trigger plot update
            // In real implementation, this would pass the data to the plot
            if let Some(_plot_node) = self.get_canvas_node_mut(plot_id) {
                // Mark plot as having data
                // The plot will read from the source node's data
            }
        }
    }
    
    pub fn remove_data_node(&mut self, id: NodeId) {
        self.data_nodes.retain(|node| node.id != id);
        self.update_tables();
    }
    
    pub fn get_data_node(&self, id: NodeId) -> Option<&DataNode> {
        self.data_nodes.iter().find(|node| node.id == id)
    }
    
    pub fn add_connection(&mut self, from: NodeId, to: NodeId, connection_type: ConnectionType) {
        let connection = NodeConnection {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            connection_type,
        };
        self.connections.push(connection);
        
        // When connecting to a plot, update it with data
        self.update_connected_plots(from);
    }
    
    pub fn get_canvas_node(&self, id: NodeId) -> Option<&CanvasNode> {
        self.canvas_nodes.get(&id)
    }
    
    pub fn get_canvas_node_mut(&mut self, id: NodeId) -> Option<&mut CanvasNode> {
        self.canvas_nodes.get_mut(&id)
    }
} 