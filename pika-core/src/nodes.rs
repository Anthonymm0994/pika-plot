//! Node type definitions for the canvas.

use serde::{Deserialize, Serialize};
use crate::types::{NodeId, TableInfo, QueryResult};
use crate::plots::PlotConfig;

/// Node types in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Data source node (CSV, Parquet, etc.)
    DataSource {
        table_info: TableInfo,
    },
    
    /// Query node that transforms data
    Query {
        query: String,
        result: Option<QueryResult>,
    },
    
    /// Plot node for visualization
    Plot {
        config: PlotConfig,
    },
    
    /// Join node for combining datasets
    Join {
        join_type: JoinType,
        left_key: String,
        right_key: String,
    },
    
    /// Filter node
    Filter {
        condition: String,
    },
    
    /// Aggregation node
    Aggregate {
        group_by: Vec<String>,
        aggregations: Vec<AggregateFunction>,
    },
}

/// Join types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// Aggregate functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateFunction {
    pub column: String,
    pub function: AggregateFunctionType,
    pub alias: Option<String>,
}

/// Aggregate function types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunctionType {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    StdDev,
    Variance,
}

/// A node in the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub selected: bool,
} 