//! # Database Abstraction Layer
//!
//! This module provides a trait-based database abstraction for the URL shortener service.
//! The abstraction allows for easy switching between different database backends while
//! maintaining a consistent interface.
//!
//! ## Architecture
//!
//! The database layer consists of:
//! - [`UrlDatabase`] trait - Defines the interface for URL storage operations
//! - [`DatabaseError`] enum - Comprehensive error handling for database operations
//! - Concrete implementations (currently SQLite)
//!
//! ## Supported Databases
//!
//! - **SQLite** - File-based database with automatic migrations (default)
//! - **In-memory SQLite** - For testing and development
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::database::{UrlDatabase, SqliteUrlDatabase};
//! use url_shortener_ztm_lib::configuration::DatabaseSettings;
//! use url_shortener_ztm_lib::DatabaseType;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a database from configuration
//! let db = SqliteUrlDatabase::from_config(&DatabaseSettings {
//!     r#type: DatabaseType::Sqlite,
//!     url: "database.db".to_string(),
//!     create_if_missing: true,
//! }).await?;
//!
//! // Run migrations
//! db.migrate().await?;
//!
//! // Store a URL
//! db.insert_url("abc123", "https://example.com").await?;
//!
//! // Retrieve a URL
//! let url = db.get_url("abc123").await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use std::fmt;

// module declarations
pub mod postgres_sql;
pub mod sqlite;

// Re-exports for convenience
pub use sqlite::*;

/// Database operation errors.
///
/// This enum represents all possible errors that can occur during database operations,
/// providing detailed error information for debugging and error handling.
#[derive(Debug)]
pub enum DatabaseError {
    /// Error establishing or maintaining database connection
    ConnectionError(String),
    /// Error executing database queries
    QueryError(String),
    /// Error running database migrations
    MigrationError(String),
    /// Requested record was not found
    NotFound,
    /// Attempted to insert a duplicate record
    Duplicate,
}

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

impl std::error::Error for DatabaseError {}

/// Trait defining the interface for URL storage operations.
///
/// This trait abstracts database operations for URL storage, allowing different
/// database implementations to be used interchangeably. All implementations must
/// be thread-safe (`Send + Sync`) to work with async Rust.
///
/// ## Thread Safety
///
/// Implementations must be `Send + Sync` to be used across thread boundaries
/// in async contexts.
///
/// ## Error Handling
///
/// All operations return `Result<T, DatabaseError>` for comprehensive error handling.
/// See [`DatabaseError`] for possible error variants.
///
/// ## Examples
///
/// ```rust,no_run
// use url_shortener_ztm_lib::database::UrlDatabase;
//
// # async fn example<DB: UrlDatabase>(db: &DB) -> Result<(), Box<dyn std::error::Error>> {
// // Store a shortened URL
// db.insert_url("abc123", "https://example.com").await?;
//
// // Retrieve the original URL
// let original_url = db.get_url("abc123").await?;
// assert_eq!(original_url, "https://example.com");
// # Ok(())
// # }
/// ```
#[async_trait]
pub trait UrlDatabase: Send + Sync {
    /// Stores a URL with the given ID in the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The short identifier for the URL
    /// * `url` - The original URL to store
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the URL was successfully stored, or an error if:
    /// - The ID already exists (`DatabaseError::Duplicate`)
    /// - A database error occurred (`DatabaseError::QueryError`)
    /// - A connection error occurred (`DatabaseError::ConnectionError`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::database::UrlDatabase;
    ///
    /// # async fn example<DB: UrlDatabase>(db: &DB) -> Result<(), Box<dyn std::error::Error>> {
    /// db.insert_url("abc123", "https://example.com").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError>;

    /// Retrieves a URL by its short ID from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The short identifier to look up
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the original URL if found, or an error if:
    /// - The ID was not found (`DatabaseError::NotFound`)
    /// - A database error occurred (`DatabaseError::QueryError`)
    /// - A connection error occurred (`DatabaseError::ConnectionError`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::database::UrlDatabase;
    ///
    /// # async fn example<DB: UrlDatabase>(db: &DB) -> Result<(), Box<dyn std::error::Error>> {
    /// let url = db.get_url("abc123").await?;
    /// println!("Original URL: {}", url);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_url(&self, id: &str) -> Result<String, DatabaseError>;
}
