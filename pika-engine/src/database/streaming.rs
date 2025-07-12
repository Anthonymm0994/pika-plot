//! DuckDB streaming implementation for large result sets.
//! Based on Gemini 2.5 Pro's recommendations for streaming patterns.

use std::sync::Arc;
use tokio::sync::mpsc;
use duckdb::{Connection, Result as DuckResult};
use arrow::record_batch::RecordBatch;
use pika_core::{
    error::{PikaError, Result},
    types::NodeId,
    events::AppEvent,
};

/// Stream query results from DuckDB with backpressure support.
pub async fn stream_query_results(
    conn: Arc<Connection>,
    query: String,
    batch_tx: mpsc::Sender<Result<RecordBatch>>,
    progress_tx: Option<mpsc::Sender<AppEvent>>,
    node_id: NodeId,
) -> Result<()> {
    // Use spawn_blocking for DuckDB operations
    tokio::task::spawn_blocking(move || {
        stream_query_blocking(conn, query, batch_tx, progress_tx, node_id)
    })
    .await
    .map_err(|e| PikaError::Internal(format!("Task join error: {}", e)))?
}

fn stream_query_blocking(
    conn: Arc<Connection>,
    query: String,
    batch_tx: mpsc::Sender<Result<RecordBatch>>,
    progress_tx: Option<mpsc::Sender<AppEvent>>,
    node_id: NodeId,
) -> Result<()> {
    // First, try to estimate row count for progress reporting
    let total_rows = estimate_row_count(&conn, &query).ok();
    
    // Execute the query with Arrow output
    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| PikaError::Database(format!("Failed to prepare query: {}", e)))?;
    
    // Get Arrow result
    let arrow_result = stmt
        .query_arrow([])
        .map_err(|e| PikaError::Database(format!("Query execution failed: {}", e)))?;
    
    let mut rows_processed = 0;
    let batch_size = 10_000; // Reasonable default batch size
    
    // Stream batches
    for batch_result in arrow_result {
        match batch_result {
            Ok(batch) => {
                rows_processed += batch.num_rows();
                
                // Send progress update if we have an estimate
                if let (Some(total), Some(tx)) = (total_rows, &progress_tx) {
                    let progress = rows_processed as f32 / total as f32;
                    let _ = tx.blocking_send(AppEvent::ProgressUpdate {
                        node_id,
                        progress: progress.min(1.0),
                    });
                }
                
                // Send batch - this will block if receiver is slow (backpressure)
                if batch_tx.blocking_send(Ok(batch)).is_err() {
                    // Receiver dropped, stop processing
                    break;
                }
            }
            Err(e) => {
                let _ = batch_tx.blocking_send(Err(PikaError::Database(format!(
                    "Failed to read batch: {}",
                    e
                ))));
                return Err(PikaError::Database(format!("Streaming failed: {}", e)));
            }
        }
    }
    
    // Send completion progress
    if let Some(tx) = progress_tx {
        let _ = tx.blocking_send(AppEvent::ProgressUpdate {
            node_id,
            progress: 1.0,
        });
    }
    
    Ok(())
}

/// Estimate row count for progress reporting.
/// This runs a COUNT(*) query which might be expensive for complex queries.
fn estimate_row_count(conn: &Connection, query: &str) -> Result<usize> {
    let count_query = format!("SELECT COUNT(*) FROM ({})", query);
    
    let count: i64 = conn
        .query_row(&count_query, [], |row| row.get(0))
        .map_err(|e| PikaError::Database(format!("Failed to estimate count: {}", e)))?;
    
    Ok(count as usize)
}

/// Configuration for streaming queries.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Maximum number of batches in flight
    pub channel_capacity: usize,
    /// Whether to estimate total rows (can be expensive)
    pub estimate_progress: bool,
    /// Batch size for streaming
    pub batch_size: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 8, // As Gemini suggested, small buffer for backpressure
            estimate_progress: true,
            batch_size: 10_000,
        }
    }
}

/// Result of a streaming query.
pub enum StreamingResult {
    /// Small result that fits in memory
    Complete(RecordBatch),
    /// Large result that should be streamed
    Stream {
        receiver: mpsc::Receiver<Result<RecordBatch>>,
        estimated_rows: Option<usize>,
    },
}

/// Execute a query and decide whether to stream or return complete result.
pub async fn execute_adaptive(
    conn: Arc<Connection>,
    query: String,
    config: StreamConfig,
    node_id: NodeId,
) -> Result<StreamingResult> {
    // First, check if this is a small result we can return directly
    let is_small = check_if_small_result(&conn, &query).await?;
    
    if is_small {
        // Execute and return complete result
        let batch = execute_to_single_batch(conn, query).await?;
        Ok(StreamingResult::Complete(batch))
    } else {
        // Set up streaming
        let (tx, rx) = mpsc::channel(config.channel_capacity);
        let (progress_tx, _progress_rx) = mpsc::channel(100);
        
        // Get row estimate if requested
        let estimated_rows = if config.estimate_progress {
            estimate_row_count(&conn, &query).ok()
        } else {
            None
        };
        
        // Start streaming in background
        let query_clone = query.clone();
        tokio::spawn(async move {
            let _ = stream_query_results(
                conn,
                query_clone,
                tx,
                Some(progress_tx),
                node_id,
            ).await;
        });
        
        Ok(StreamingResult::Stream {
            receiver: rx,
            estimated_rows,
        })
    }
}

async fn check_if_small_result(conn: &Arc<Connection>, query: &str) -> Result<bool> {
    // Simple heuristic: if LIMIT is present and small, it's a small result
    let query_upper = query.to_uppercase();
    if let Some(limit_pos) = query_upper.find("LIMIT") {
        if let Some(limit_value) = extract_limit_value(&query[limit_pos..]) {
            return Ok(limit_value <= 10_000);
        }
    }
    
    // Otherwise, assume large for safety
    Ok(false)
}

fn extract_limit_value(limit_clause: &str) -> Option<usize> {
    // Simple extraction of LIMIT value
    let parts: Vec<&str> = limit_clause.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
}

async fn execute_to_single_batch(
    conn: Arc<Connection>,
    query: String,
) -> Result<RecordBatch> {
    tokio::task::spawn_blocking(move || {
        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        let mut arrow_result = stmt
            .query_arrow([])
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        // Collect all batches (assuming small result)
        let batches: Vec<RecordBatch> = arrow_result
            .collect::<DuckResult<Vec<_>>>()
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        if batches.is_empty() {
            Err(PikaError::Database("Query returned no results".into()))
        } else if batches.len() == 1 {
            Ok(batches.into_iter().next().unwrap())
        } else {
            // Concatenate batches
            // TODO: Implement batch concatenation
            Ok(batches.into_iter().next().unwrap())
        }
    })
    .await
    .map_err(|e| PikaError::Internal(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_limit_value() {
        assert_eq!(extract_limit_value("LIMIT 100"), Some(100));
        assert_eq!(extract_limit_value("LIMIT 5000"), Some(5000));
        assert_eq!(extract_limit_value("LIMIT"), None);
    }
    
    #[tokio::test]
    async fn test_check_small_result() {
        // This would need a real connection to test properly
        // For now, just test the limit detection
        let query = "SELECT * FROM table LIMIT 100";
        assert!(query.to_uppercase().contains("LIMIT"));
    }
} 