//! Query execution engine.

use std::sync::Arc;
use std::time::Instant;
use parking_lot::Mutex;
use pika_core::{
    error::{PikaError, Result},
    types::QueryResult,
};
use crate::database::Database;

/// Query execution engine with caching support
pub struct QueryEngine {
    database: Arc<Mutex<Database>>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(database: Arc<Mutex<Database>>) -> Self {
        QueryEngine { database }
    }
    
    /// Execute a SQL query
    pub async fn execute(&self, sql: &str) -> Result<QueryResult> {
        let start = Instant::now();
        
        // Get column information from a simple query
        let columns = self.get_column_names(sql).await?;
        
        // Count rows (this is inefficient for large results, but works for now)
        let count_sql = format!("SELECT COUNT(*) FROM ({})", sql);
        let row_count: i64 = {
            let db = self.database.lock();
            db.query_scalar(&count_sql).await?
        };
        
        let execution_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(QueryResult {
            columns,
            row_count: row_count as usize,
            execution_time_ms,
            memory_used_bytes: None,
        })
    }
    
    /// Get column names from a query
    async fn get_column_names(&self, sql: &str) -> Result<Vec<String>> {
        // Execute with LIMIT 0 to get schema without data
        let schema_sql = format!("{} LIMIT 0", sql);
        let db = self.database.lock();
        let batches = db.query(&schema_sql).await?;
        
        if let Some(batch) = batches.first() {
            let schema = batch.schema();
            Ok(schema.fields().iter()
                .map(|f| f.name().clone())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Execute a query and return Arrow RecordBatches
    pub async fn execute_arrow(&self, sql: &str) -> Result<Vec<duckdb::arrow::record_batch::RecordBatch>> {
        let db = self.database.lock();
        db.query(sql).await
    }
    
    /// Validate a SQL query without executing it
    pub async fn validate(&self, sql: &str) -> Result<()> {
        // Try to get schema with LIMIT 0
        let schema_sql = format!("{} LIMIT 0", sql);
        let db = self.database.lock();
        db.execute(&schema_sql).await?;
        Ok(())
    }
    
    /// Get query execution plan
    pub async fn explain(&self, sql: &str) -> Result<String> {
        let explain_sql = format!("EXPLAIN {}", sql);
        let db = self.database.lock();
        
        let mut explanation = String::new();
        let results = db.query_map(&explain_sql, |row| {
            Ok(row.get::<_, String>(0)?)
        }).await?;
        
        for line in results {
            explanation.push_str(&line);
            explanation.push('\n');
        }
        
        Ok(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_query_execution() {
        let db = Arc::new(Mutex::new(Database::new().await.unwrap()));
        let engine = QueryEngine::new(db.clone());
        
        // Create test table
        {
            let database = db.lock();
            database.execute("CREATE TABLE test (id INTEGER, name VARCHAR)").await.unwrap();
            database.execute("INSERT INTO test VALUES (1, 'Alice'), (2, 'Bob')").await.unwrap();
        }
        
        // Execute query
        let result = engine.execute("SELECT * FROM test").await.unwrap();
        
        assert_eq!(result.row_count, 2);
        assert_eq!(result.columns.len(), 2);
        assert!(result.execution_time_ms >= 0);
    }
    
    #[tokio::test]
    async fn test_query_validation() {
        let db = Arc::new(Mutex::new(Database::new().await.unwrap()));
        let engine = QueryEngine::new(db);
        
        // Valid query
        assert!(engine.validate("SELECT 1").await.is_ok());
        
        // Invalid query
        assert!(engine.validate("INVALID SQL").await.is_err());
    }
} 