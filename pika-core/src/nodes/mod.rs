use serde::{Serialize, Deserialize};
use crate::types::{TableInfo, QueryResult, NodeId, Point2, Size2};
use crate::plots::PlotConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    DataSource { table_info: TableInfo },
    Query { query: String, result: Option<QueryResult> },
    Plot { config: PlotConfig },
    Join { join_type: JoinType, left_key: String, right_key: String },
    Filter { condition: String },
    Aggregate { group_by: Vec<String>, aggregations: Vec<AggregateFunction> },
    Table { table_info: TableInfo },
    Note { content: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType { Inner, Left, Right, Full, Cross }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateFunction {
    pub column: String,
    pub function: AggregateFunctionType,
    pub alias: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunctionType { Count, Sum, Avg, Min, Max, StdDev, Variance }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub selected: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub title: String,
    pub pos: Point2,
    pub size: Size2,
}
impl Node {
    pub fn new(typ: NodeType) -> Self {
        let typ_clone = typ.clone();
        Self { id: NodeId::new(), node_type: typ, title: format!("{:?} Node", typ_clone), pos: Point2::new(0.0, 0.0), size: Size2::new(200.0, 150.0) }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub id: String,
    pub from_node: NodeId,
    pub from_port: String,
    pub to_node: NodeId,
    pub to_port: String,
}
impl Default for TableInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_path: None,
            row_count: None,
            columns: Vec::new(),
            preview_data: None,
        }
    }
}