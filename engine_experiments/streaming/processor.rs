use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use arrow::record_batch::RecordBatch;
use crate::{PikaError, Result};
use super::{StreamingBuffer, StreamingConfig};

/// Processes streaming data through various transformations
pub struct StreamingProcessor {
    config: StreamingConfig,
    buffer: Arc<RwLock<StreamingBuffer>>,
    tx: mpsc::Sender<RecordBatch>,
    rx: mpsc::Receiver<RecordBatch>,
}

impl StreamingProcessor {
    pub fn new(config: StreamingConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.batch_size);
        Self {
            config,
            buffer: Arc::new(RwLock::new(StreamingBuffer::new(config.batch_size))),
            tx,
            rx,
        }
    }

    /// Process a batch of data
    pub async fn process_batch(&mut self, batch: RecordBatch) -> Result<()> {
        // Apply transformations if needed
        let processed = self.transform_batch(batch)?;
        
        // Send to buffer
        self.tx.send(processed).await
            .map_err(|_| PikaError::Internal("Failed to send batch".to_string()))?;
        
        Ok(())
    }

    /// Transform a batch (placeholder for future transformations)
    fn transform_batch(&self, batch: RecordBatch) -> Result<RecordBatch> {
        // For now, just pass through
        // In the future, this could apply filters, projections, etc.
        Ok(batch)
    }

    /// Get the next processed batch
    pub async fn next_batch(&mut self) -> Option<RecordBatch> {
        self.rx.recv().await
    }

    /// Get current buffer state
    pub async fn buffer_state(&self) -> StreamingBuffer {
        self.buffer.read().await.clone()
    }

    /// Clear the buffer
    pub async fn clear_buffer(&self) {
        let mut buffer = self.buffer.write().await;
        buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let id_array = Int32Array::from(vec![1, 2, 3]);
        let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie"]);

        RecordBatch::try_new(
            schema,
            vec![Arc::new(id_array), Arc::new(name_array)],
        ).unwrap()
    }

    #[tokio::test]
    async fn test_process_batch() {
        let config = StreamingConfig {
            batch_size: 1000,
            buffer_size_mb: 100,
            flush_interval_ms: 1000,
        };
        
        let mut processor = StreamingProcessor::new(config);
        let batch = create_test_batch();
        
        processor.process_batch(batch.clone()).await.unwrap();
        
        let received = processor.next_batch().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().num_rows(), batch.num_rows());
    }
} 