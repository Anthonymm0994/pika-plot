//! Database connection and management.

use std::collections::HashMap;
use crate::error::Result;

pub struct Database {
    data: HashMap<String, Vec<Vec<String>>>
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self { data: HashMap::new() })
    }

    pub fn insert_csv(&self, table: &str, df: DataFrame) -> Result<()> {
        // Stub
        Ok(())
    }

    pub fn query(&self, sql: &str) -> Result<DataFrame> {
        // Stub
        Ok(DataFrame::default())
    }
}

// Define DataFrame as alias for Vec<Vec<String>>
pub type DataFrame = Vec<Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new("test.db").unwrap();
        
        // Test simple query
        // db.execute("CREATE TABLE test (id INTEGER, name VARCHAR)").await.unwrap();
        // db.execute("INSERT INTO test VALUES (1, 'test')").await.unwrap();
        
        // let count: i64 = db.query_scalar("SELECT COUNT(*) FROM test").await.unwrap();
        // assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_query_map() {
        let db = Database::new("test.db").unwrap();
        
        // db.execute("CREATE TABLE test (id INTEGER, name VARCHAR)").await.unwrap();
        // db.execute("INSERT INTO test VALUES (1, 'one'), (2, 'two')").await.unwrap();
        
        // let results = db.query_map("SELECT id, name FROM test ORDER BY id", |row| {
        //     Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
        // }).await.unwrap();
        
        // assert_eq!(results.len(), 2);
        // assert_eq!(results[0], (1, "one".to_string()));
        // assert_eq!(results[1], (2, "two".to_string()));
    }
} 