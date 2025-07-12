//! Query execution and result handling.

use crate::database::Database;
use pika_core::{
    error::{PikaError, Result},
    types::{NodeId, QueryResult},
};
use std::sync::Arc;
use std::collections::HashMap;

/// Execute a SQL query and return results.
pub async fn execute(db: &Database, sql: &str) -> Result<QueryResult> {
    let start_time = std::time::Instant::now();
    
    // Prepare statement
    let mut stmt = db.prepare(sql)?;
    
    // Get column information
    let column_count = stmt.column_count();
    let mut column_names = Vec::with_capacity(column_count);
    let mut column_types = Vec::with_capacity(column_count);
    
    for i in 0..column_count {
        let name = stmt.column_name(i)
            .map_err(|e| PikaError::Database(e.to_string()))?
            .to_string();
        column_names.push(name);
        // DuckDB doesn't expose column type info easily through rusqlite-style API
        column_types.push("UNKNOWN".to_string());
    }
    
    // For now, we'll just count rows without materializing results
    // In a real implementation, we'd stream results or cache them
    let row_count = count_query_rows(db, sql)?;
    
    let execution_time_ms = start_time.elapsed().as_millis() as u64;
    let execution_time = std::time::Duration::from_millis(execution_time_ms);
    
    // Create an opaque data handle (empty for now)
    let data_handle = Arc::new(()) as Arc<dyn std::any::Any + Send + Sync>;
    
    Ok(QueryResult {
        data_handle,
        execution_time,
        row_count,
        memory_usage: 0, // TODO: Calculate actual memory usage
        execution_time_ms: execution_time_ms as f64,
        column_names,
        column_types,
    })
}

/// Count the number of rows a query would return.
fn count_query_rows(db: &Database, sql: &str) -> Result<usize> {
    // Wrap the original query in a COUNT(*) query
    let count_sql = format!("SELECT COUNT(*) FROM ({})", sql);
    
    let mut stmt = db.prepare(&count_sql)?;
    let count: i64 = stmt.query_row([], |row| row.get(0))
        .map_err(|e| PikaError::Database(e.to_string()))?;
    
    Ok(count as usize)
}

/// Execute a query and export results to a file.
pub async fn execute_to_file(
    db: &Database,
    sql: &str,
    output_path: &str,
    format: ExportFormat,
) -> Result<()> {
    let export_sql = match format {
        ExportFormat::Csv => {
            format!("COPY ({}) TO '{}' (FORMAT CSV, HEADER)", sql, output_path)
        }
        ExportFormat::Parquet => {
            format!("COPY ({}) TO '{}' (FORMAT PARQUET)", sql, output_path)
        }
        ExportFormat::Json => {
            format!("COPY ({}) TO '{}' (FORMAT JSON)", sql, output_path)
        }
    };
    
    db.execute_sql(&export_sql)?;
    Ok(())
}

/// Supported export formats.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Parquet,
    Json,
}

/// Get a preview of query results (first N rows).
pub async fn preview_query(
    db: &Database,
    sql: &str,
    limit: usize,
) -> Result<Vec<HashMap<String, String>>> {
    let preview_sql = format!("{} LIMIT {}", sql, limit);
    let mut stmt = db.prepare(&preview_sql)?;
    
    let column_count = stmt.column_count();
    let mut column_names = Vec::with_capacity(column_count);
    
    for i in 0..column_count {
        column_names.push(
            stmt.column_name(i)
                .map_err(|e| PikaError::Database(e.to_string()))?
                .to_string()
        );
    }
    
    let rows = stmt.query_map([], |row| {
        let mut row_map = HashMap::new();
        for (i, name) in column_names.iter().enumerate() {
            // Convert all values to strings for preview
            let value: String = row.get(i).unwrap_or_else(|_| "NULL".to_string());
            row_map.insert(name.clone(), value);
        }
        Ok(row_map)
    }).map_err(|e| PikaError::Database(e.to_string()))?;
    
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| PikaError::Database(e.to_string()))?);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_query_execution() {
        let db = Database::new(None).await.unwrap();
        
        // Create a test table
        db.execute_sql("CREATE TABLE test (id INTEGER, name VARCHAR)").unwrap();
        db.execute_sql("INSERT INTO test VALUES (1, 'Alice'), (2, 'Bob')").unwrap();
        
        // Execute query
        let result = execute(&db, "SELECT * FROM test").await.unwrap();
        assert_eq!(result.row_count, 2);
        assert_eq!(result.schema.len(), 2);
    }
} 