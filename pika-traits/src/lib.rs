use std::error::Error as StdError;
use pika_core::types::NodeId;
pub struct ProcessedData { data: Vec<u8> }
use pika_core::error::Result;

pub enum NodeType {
    Plot,
    Table,
    Note,
}

pub trait NodeCore {
    fn id(&self) -> NodeId;
    fn execute(&self) -> Result<()>;
    // Add other pure methods
}

pub trait DataProcessor {
    fn process(&self, data: &[u8]) -> Result<ProcessedData>;
}