//! Node implementations for the canvas

pub mod plot_node;
pub mod query_node;
pub mod table_node;

pub use plot_node::PlotNode;
pub use query_node::QueryNode;
pub use table_node::TableNode;

use pika_core::{Node, NodeId, Point2, Size2};

/// Factory function to create nodes by type name
pub fn create_node(node_type: &str, id: NodeId) -> Option<Box<dyn Node>> {
    match node_type {
        "PlotNode" => Some(Box::new(PlotNode::new(id))),
        "QueryNode" => Some(Box::new(QueryNode::new(id))),
        "TableNode" => Some(Box::new(TableNode::new(id))),
        _ => None,
    }
}

/// Standard node sizes
pub const TABLE_NODE_SIZE: Size2 = Size2 { width: 300.0, height: 200.0 };
pub const QUERY_NODE_SIZE: Size2 = Size2 { width: 350.0, height: 250.0 };
pub const PLOT_NODE_SIZE: Size2 = Size2 { width: 400.0, height: 400.0 }; 