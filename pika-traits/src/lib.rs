pub trait NodeCore {
    fn id(&self) -> NodeId;
    fn execute(&self) -> Result<()>;
    // Add other pure methods
}

pub trait DataProcessor {
    fn process(&self, data: &[u8]) -> Result<ProcessedData>;
} 