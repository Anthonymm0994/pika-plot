//! Integration tests for pika-engine.

use pika_engine::{Engine, import, query};
use pika_core::{
    types::{ImportOptions, NodeId},
    error::Result,
};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Create a test CSV file with sample data.
fn create_test_csv(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("test_data.csv");
    std::fs::write(&path, 
        "id,name,value,timestamp\n\
         1,Alice,100.5,2024-01-01\n\
         2,Bob,200.7,2024-01-02\n\
         3,Charlie,150.3,2024-01-03\n"
    ).unwrap();
    path
}

#[tokio::test]
async fn test_csv_import_and_query() -> Result<()> {
    let engine = Engine::new(None).await?;
    let temp_dir = TempDir::new()?;
    let csv_path = create_test_csv(&temp_dir);
    
    // Import CSV
    let options = ImportOptions {
        has_header: true,
        delimiter: Some(','),
        sample_size: None,
    };
    
    let node_id = engine.import_file(&csv_path, options).await?;
    assert_ne!(node_id.0, uuid::Uuid::nil());
    
    // Query the imported data
    let result = engine.execute_query("SELECT COUNT(*) as count FROM data_*").await?;
    assert_eq!(result.row_count, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_queries() -> Result<()> {
    let engine = Engine::new(None).await?;
    
    // Run multiple queries concurrently
    let handles: Vec<_> = (0..5).map(|i| {
        let engine_clone = engine.clone();
        tokio::spawn(async move {
            engine_clone.execute_query(&format!("SELECT {} as value", i)).await
        })
    }).collect();
    
    // Wait for all queries to complete
    for handle in handles {
        let result = handle.await.unwrap()?;
        assert_eq!(result.row_count, 1);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_memory_limit_enforcement() -> Result<()> {
    // Create engine with small memory limit
    let engine = Engine::new(Some(100 * 1024 * 1024)).await?; // 100MB
    
    // This should work within limits
    let result = engine.execute_query("SELECT 1").await;
    assert!(result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_snapshot_save_load() -> Result<()> {
    let engine = Engine::new(None).await?;
    let temp_dir = TempDir::new()?;
    let snapshot_path = temp_dir.path().join("test_snapshot.pika");
    
    // Save snapshot
    engine.save_snapshot(&snapshot_path).await?;
    assert!(snapshot_path.exists());
    
    // Load snapshot in new engine
    let mut engine2 = Engine::new(None).await?;
    engine2.load_snapshot(&snapshot_path).await?;
    
    Ok(())
}

#[cfg(test)]
mod test_utils {
    use super::*;
    
    /// Create test parquet file for benchmarking.
    pub fn create_large_test_dataset(dir: &TempDir, rows: usize) -> PathBuf {
        let path = dir.path().join("large_test.csv");
        let mut content = String::from("id,value,category\n");
        
        for i in 0..rows {
            content.push_str(&format!("{},{},{}\n", 
                i, 
                i as f64 * 1.5, 
                if i % 3 == 0 { "A" } else if i % 3 == 1 { "B" } else { "C" }
            ));
        }
        
        std::fs::write(&path, content).unwrap();
        path
    }
} 