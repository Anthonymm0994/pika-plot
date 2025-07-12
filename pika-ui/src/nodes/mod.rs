//! Node implementations for the canvas

pub mod table_node;
pub mod query_node;
pub mod plot_node;

pub use table_node::TableNode;
pub use query_node::QueryNode;
pub use plot_node::PlotNode;

use pika_core::{Node, NodeId, Point2, Size2};
use std::sync::Arc;

/// Helper to create a node from its type name
pub fn create_node(type_name: &str, position: Point2) -> Option<Box<dyn Node>> {
    match type_name {
        "TableNode" => Some(Box::new(TableNode::new(position))),
        "QueryNode" => Some(Box::new(QueryNode::new(position))),
        "PlotNode" => Some(Box::new(PlotNode::new(position))),
        _ => None,
    }
}

/// Standard node sizes
pub const TABLE_NODE_SIZE: Size2 = Size2 { width: 300.0, height: 200.0 };
pub const QUERY_NODE_SIZE: Size2 = Size2 { width: 350.0, height: 250.0 };
pub const PLOT_NODE_SIZE: Size2 = Size2 { width: 400.0, height: 400.0 }; 