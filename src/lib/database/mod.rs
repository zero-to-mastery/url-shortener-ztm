// src/lib/database/mod.rs

// dependencies
use async_trait::async_trait;
use std::fmt;

// module declarations
pub mod sqlite;

// re-exports
pub use sqlite::*;

// database error type
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    QueryError(String),
    MigrationError(String),
    NotFound,
    Duplicate,
}

// Implement Display for DatabaseError
impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Database query error: {}", msg),
            DatabaseError::NotFound => write!(f, "Record not found"),
            DatabaseError::Duplicate => write!(f, "Duplicate record"),
            DatabaseError::MigrationError(msg) => write!(f, "Database migration error: {}", msg),
        }
    }
}

// Implement std::error::Error for DatabaseError
impl std::error::Error for DatabaseError {}

// UrlDatabase trait definition
#[async_trait]
pub trait UrlDatabase: Send + Sync {
    async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError>;
    async fn get_url(&self, id: &str) -> Result<String, DatabaseError>;
}
