//! Database connection and management.

use duckdb::{Connection, Result as DuckResult};
use parking_lot::Mutex;
use std::sync::Arc;
use pika_core::error::{PikaError, Result};

/// Database wrapper for DuckDB connections
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new in-memory database
    pub async fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| PikaError::Database(e))?;
        
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // Initialize schema
        db.init_schema().await?;
        
        Ok(db)
    }
    
    /// Create a new database with a file path
    pub async fn new_with_path(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| PikaError::Database(e))?;
        
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.init_schema().await?;
        
        Ok(db)
    }
    
    /// Initialize the database schema
    async fn init_schema(&self) -> Result<()> {
        // Create metadata tables
        self.execute(
            "CREATE TABLE IF NOT EXISTS _pika_metadata (
                key VARCHAR PRIMARY KEY,
                value VARCHAR,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).await?;
        
        // Create query cache table
        self.execute(
            "CREATE TABLE IF NOT EXISTS _pika_query_cache (
                query_hash VARCHAR PRIMARY KEY,
                result_path VARCHAR,
                row_count BIGINT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).await?;
        
        Ok(())
    }
    
    /// Execute a SQL statement
    pub async fn execute(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(sql, [])
            .map_err(|e| PikaError::Database(e))?;
        Ok(())
    }
    
    /// Execute a query and return the result as an Arrow RecordBatch
    pub async fn query(&self, sql: &str) -> Result<Vec<duckdb::arrow::record_batch::RecordBatch>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(sql)
            .map_err(|e| PikaError::Database(e))?;
        
        let arrow = stmt.query_arrow([])
            .map_err(|e| PikaError::Database(e))?;
        
        // Collect all batches
        let batches: Vec<_> = arrow.collect();
        Ok(batches)
    }
    
    /// Query a single scalar value
    pub async fn query_scalar<T>(&self, sql: &str) -> Result<T>
    where
        T: duckdb::types::FromSql,
    {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(sql)
            .map_err(|e| PikaError::Database(e))?;
        
        let mut rows = stmt.query([])
            .map_err(|e| PikaError::Database(e))?;
        
        if let Some(row) = rows.next().map_err(|e| PikaError::Database(e))? {
            row.get(0).map_err(|e| PikaError::Database(e))
        } else {
            Err(PikaError::QueryExecution("No rows returned".to_string()))
        }
    }
    
    /// Query and map results
    pub async fn query_map<T, F>(&self, sql: &str, mut f: F) -> Result<Vec<T>>
    where
        F: FnMut(&duckdb::Row) -> DuckResult<T>,
    {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(sql)
            .map_err(|e| PikaError::Database(e))?;
        
        let mut rows = stmt.query([])
            .map_err(|e| PikaError::Database(e))?;
        
        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(|e| PikaError::Database(e))? {
            results.push(f(&row).map_err(|e| PikaError::Database(e))?);
        }
        
        Ok(results)
    }
    
    /// Set memory limit
    pub async fn set_memory_limit(&self, limit_bytes: usize) -> Result<()> {
        let limit_mb = limit_bytes / (1024 * 1024);
        let sql = format!("SET memory_limit='{}MB'", limit_mb);
        self.execute(&sql).await
    }
    
    /// Get current memory usage
    pub async fn get_memory_usage(&self) -> Result<usize> {
        let usage: i64 = self.query_scalar("SELECT current_setting('memory_usage')").await?;
        Ok(usage as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new().await.unwrap();
        
        // Test simple query
        db.execute("CREATE TABLE test (id INTEGER, name VARCHAR)").await.unwrap();
        db.execute("INSERT INTO test VALUES (1, 'test')").await.unwrap();
        
        let count: i64 = db.query_scalar("SELECT COUNT(*) FROM test").await.unwrap();
        assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_query_map() {
        let db = Database::new().await.unwrap();
        
        db.execute("CREATE TABLE test (id INTEGER, name VARCHAR)").await.unwrap();
        db.execute("INSERT INTO test VALUES (1, 'one'), (2, 'two')").await.unwrap();
        
        let results = db.query_map("SELECT id, name FROM test ORDER BY id", |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
        }).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (1, "one".to_string()));
        assert_eq!(results[1], (2, "two".to_string()));
    }
} 