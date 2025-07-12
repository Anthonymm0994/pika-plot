//! Database layer using DuckDB for data storage and querying.

use duckdb::{Connection, params};
use pika_core::{
    error::{PikaError, Result},
    types::NodeId,
};
use std::path::Path;

/// Database wrapper for DuckDB operations.
pub struct Database {
    conn: Connection,
    memory_limit: Option<u64>,
}

impl Database {
    /// Create a new in-memory database.
    pub async fn new(memory_limit: Option<u64>) -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        // Set memory limit if specified
        if let Some(limit) = memory_limit {
            conn.execute(&format!("SET memory_limit='{}'", format_bytes(limit)), [])
                .map_err(|e| PikaError::Database(e.to_string()))?;
        }
        
        // Enable parallel execution
        conn.execute("SET threads TO 0", [])
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        // Create initial schema
        Self::create_schema(&conn)?;
        
        Ok(Self { conn, memory_limit })
    }
    
    /// Open a database from file.
    pub async fn open(path: &Path, memory_limit: Option<u64>) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        if let Some(limit) = memory_limit {
            conn.execute(&format!("SET memory_limit='{}'", format_bytes(limit)), [])
                .map_err(|e| PikaError::Database(e.to_string()))?;
        }
        
        Ok(Self { conn, memory_limit })
    }
    
    /// Get a reference to the connection.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
    
    /// Create initial database schema.
    fn create_schema(conn: &Connection) -> Result<()> {
        // Metadata table for tracking imported data
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pika_metadata (
                node_id UUID PRIMARY KEY,
                table_name VARCHAR NOT NULL,
                source_path VARCHAR,
                import_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                row_count BIGINT,
                schema_info JSON
            )",
            [],
        ).map_err(|e| PikaError::Database(e.to_string()))?;
        
        // Query cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pika_query_cache (
                query_hash VARCHAR PRIMARY KEY,
                query_text TEXT NOT NULL,
                result_path VARCHAR,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                access_count INTEGER DEFAULT 1,
                last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| PikaError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Execute a raw SQL query.
    pub fn execute_sql(&self, sql: &str) -> Result<usize> {
        self.conn.execute(sql, [])
            .map_err(|e| PikaError::Database(e.to_string()))
    }
    
    /// Prepare a statement.
    pub fn prepare(&self, sql: &str) -> Result<duckdb::Statement> {
        self.conn.prepare(sql)
            .map_err(|e| PikaError::Database(e.to_string()))
    }
    
    /// Begin a transaction.
    pub fn transaction(&mut self) -> Result<duckdb::Transaction> {
        self.conn.transaction()
            .map_err(|e| PikaError::Database(e.to_string()))
    }
    
    /// Query a single scalar value
    pub fn query_scalar<T>(&self, sql: &str) -> Result<T> 
    where 
        T: duckdb::types::FromSql
    {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query([])
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        if let Some(row) = rows.next().map_err(|e| PikaError::Database(e.to_string()))? {
            row.get(0).map_err(|e| PikaError::Database(e.to_string()))
        } else {
            Err(PikaError::Database("No rows returned".to_string()))
        }
    }
    
    /// Query and map rows to a vector
    pub fn query_map<T, F>(&self, sql: &str, mut map_fn: F) -> Result<Vec<T>>
    where
        F: FnMut(&duckdb::Row) -> Result<T>
    {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query([])
            .map_err(|e| PikaError::Database(e.to_string()))?;
        
        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(|e| PikaError::Database(e.to_string()))? {
            results.push(map_fn(&row)?);
        }
        
        Ok(results)
    }
}

/// Format bytes into human-readable format (e.g., "4GB", "512MB").
fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;
    
    if bytes >= GB {
        format!("{}GB", bytes / GB)
    } else if bytes >= MB {
        format!("{}MB", bytes / MB)
    } else if bytes >= KB {
        format!("{}KB", bytes / KB)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new(None).await.unwrap();
        assert!(db.execute_sql("SELECT 1").is_ok());
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1KB");
        assert_eq!(format_bytes(1024 * 1024), "1MB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2GB");
    }
} 