//! Query building utilities for safe SQL query construction.
//!
//! This module provides a builder pattern for constructing SQL queries
//! in a type-safe manner. Note that this builder does NOT provide SQL
//! injection protection - use parameterized queries for user input.

use std::fmt;

/// Query builder for constructing SQL queries
/// 
/// # Example
/// ```ignore
/// let query = QueryBuilder::new()
///     .from("users")
///     .select(vec!["id", "name"])
///     .where_clause("age > 18")
///     .order_by("name", false)
///     .limit(10)
///     .build();
/// ```
/// 
/// # Security Warning
/// This builder does NOT sanitize inputs. Never use user-provided
/// data directly in queries. Use parameterized queries instead.
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    table: Option<String>,
    columns: Vec<String>,
    filters: Vec<String>,
    order_by: Vec<OrderByClause>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Clone)]
struct OrderByClause {
    column: String,
    descending: bool,
}

/// Errors that can occur during query building
#[derive(Debug, Clone, PartialEq)]
pub enum QueryBuildError {
    /// No table specified for the query
    MissingTable,
    /// Invalid column name
    InvalidColumn(String),
    /// Invalid limit value
    InvalidLimit,
}

impl fmt::Display for QueryBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryBuildError::MissingTable => write!(f, "No table specified for query"),
            QueryBuildError::InvalidColumn(col) => write!(f, "Invalid column name: {}", col),
            QueryBuildError::InvalidLimit => write!(f, "Invalid limit value"),
        }
    }
}

impl std::error::Error for QueryBuildError {}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the table to query from
    /// 
    /// # Security Warning
    /// Table names should be validated or come from trusted sources.
    pub fn from(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }
    
    /// Select specific columns
    /// 
    /// If no columns are specified, SELECT * will be used.
    /// 
    /// # Security Warning
    /// Column names should be validated or come from trusted sources.
    pub fn select(mut self, columns: Vec<&str>) -> Self {
        self.columns = columns.iter().map(|&c| c.to_string()).collect();
        self
    }
    
    /// Add a WHERE clause condition
    /// 
    /// Multiple calls will be combined with AND.
    /// 
    /// # Security Warning
    /// NEVER use user input directly in conditions. Use parameterized queries.
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.filters.push(condition.to_string());
        self
    }
    
    /// Add ORDER BY clause
    /// 
    /// # Arguments
    /// * `column` - Column to order by
    /// * `desc` - If true, order descending; if false, order ascending
    pub fn order_by(mut self, column: &str, desc: bool) -> Self {
        self.order_by.push(OrderByClause {
            column: column.to_string(),
            descending: desc,
        });
        self
    }
    
    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set result offset (for pagination)
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    
    /// Build the SQL query string
    /// 
    /// Returns a Result to handle validation errors.
    pub fn build(&self) -> Result<String, QueryBuildError> {
        // Validate we have a table
        let table = self.table.as_ref()
            .ok_or(QueryBuildError::MissingTable)?;
        
        let mut query = String::new();
        
        // SELECT clause
        if self.columns.is_empty() {
            query.push_str("SELECT *");
        } else {
            query.push_str("SELECT ");
            query.push_str(&self.columns.join(", "));
        }
        
        // FROM clause
        query.push_str(" FROM ");
        query.push_str(table);
        
        // WHERE clause
        if !self.filters.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.filters.join(" AND "));
        }
        
        // ORDER BY clause
        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.order_by.iter()
                .map(|clause| {
                    format!("{} {}", 
                        clause.column, 
                        if clause.descending { "DESC" } else { "ASC" }
                    )
                })
                .collect();
            query.push_str(&order_parts.join(", "));
        }
        
        // LIMIT clause
        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err(QueryBuildError::InvalidLimit);
            }
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        // OFFSET clause
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
        
        Ok(query)
    }
    
    /// Reset the query builder to initial state
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Check if the query builder has any conditions set
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }
    
    /// Get the number of WHERE conditions
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_builder_basic() {
        let query = QueryBuilder::new()
            .from("users")
            .select(vec!["id", "name", "email"])
            .where_clause("age > 18")
            .order_by("name", false)
            .limit(10)
            .build()
            .unwrap();
            
        assert_eq!(
            query,
            "SELECT id, name, email FROM users WHERE age > 18 ORDER BY name ASC LIMIT 10"
        );
    }
    
    #[test]
    fn test_query_builder_minimal() {
        let query = QueryBuilder::new()
            .from("products")
            .build()
            .unwrap();
            
        assert_eq!(query, "SELECT * FROM products");
    }
    
    #[test]
    fn test_query_builder_multiple_where() {
        let query = QueryBuilder::new()
            .from("orders")
            .where_clause("status = 'active'")
            .where_clause("created_at > '2024-01-01'")
            .build()
            .unwrap();
            
        assert_eq!(
            query,
            "SELECT * FROM orders WHERE status = 'active' AND created_at > '2024-01-01'"
        );
    }
    
    #[test]
    fn test_query_builder_multiple_order_by() {
        let query = QueryBuilder::new()
            .from("products")
            .order_by("category", false)
            .order_by("price", true)
            .build()
            .unwrap();
            
        assert_eq!(
            query,
            "SELECT * FROM products ORDER BY category ASC, price DESC"
        );
    }
    
    #[test]
    fn test_query_builder_with_offset() {
        let query = QueryBuilder::new()
            .from("users")
            .limit(10)
            .offset(20)
            .build()
            .unwrap();
            
        assert_eq!(query, "SELECT * FROM users LIMIT 10 OFFSET 20");
    }
    
    #[test]
    fn test_query_builder_no_table_error() {
        let result = QueryBuilder::new()
            .select(vec!["id", "name"])
            .build();
            
        assert_eq!(result.unwrap_err(), QueryBuildError::MissingTable);
    }
    
    #[test]
    fn test_query_builder_invalid_limit() {
        let result = QueryBuilder::new()
            .from("users")
            .limit(0)
            .build();
            
        assert_eq!(result.unwrap_err(), QueryBuildError::InvalidLimit);
    }
    
    #[test]
    fn test_query_builder_reset() {
        let mut builder = QueryBuilder::new()
            .from("users")
            .where_clause("id = 1");
            
        assert!(builder.has_filters());
        assert_eq!(builder.filter_count(), 1);
        
        builder.reset();
        
        assert!(!builder.has_filters());
        assert_eq!(builder.filter_count(), 0);
        assert!(builder.build().is_err()); // No table set
    }
    
    #[test]
    fn test_query_builder_complex() {
        let query = QueryBuilder::new()
            .from("sales")
            .select(vec!["product_id", "SUM(amount) as total"])
            .where_clause("date >= '2024-01-01'")
            .where_clause("date < '2024-02-01'")
            .where_clause("status = 'completed'")
            .order_by("total", true)
            .limit(100)
            .build()
            .unwrap();
            
        assert_eq!(
            query,
            "SELECT product_id, SUM(amount) as total FROM sales WHERE date >= '2024-01-01' AND date < '2024-02-01' AND status = 'completed' ORDER BY total DESC LIMIT 100"
        );
    }
    
    #[test]
    fn test_error_display() {
        let err = QueryBuildError::MissingTable;
        assert_eq!(err.to_string(), "No table specified for query");
        
        let err = QueryBuildError::InvalidColumn("test@column".to_string());
        assert_eq!(err.to_string(), "Invalid column name: test@column");
        
        let err = QueryBuildError::InvalidLimit;
        assert_eq!(err.to_string(), "Invalid limit value");
    }
} 