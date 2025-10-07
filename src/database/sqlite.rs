//! # SQLite Database Implementation
//!
//! This module provides the SQLite implementation of the [`UrlDatabase`] trait.
//! It uses the `sqlx` crate for type-safe database operations and automatic migrations.
//!
//! ## Features
//!
//! - **Type-safe queries** - Compile-time SQL query validation
//! - **Automatic migrations** - Database schema management
//! - **Connection pooling** - Efficient database connection management
//! - **In-memory support** - For testing and development
//!
//! ## Database Schema
//!
//! The SQLite database uses a simple schema with a single `urls` table:
//!
//! ```sql
//! CREATE TABLE urls (
//!   id TEXT PRIMARY KEY,              -- Short identifier (nanoid, 6 characters)
//!   url TEXT NOT NULL                 -- Original URL
//! );
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::DatabaseType;
//! use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
//! use url_shortener_ztm_lib::configuration::DatabaseSettings;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create database from configuration
//! let config = DatabaseSettings {
//!    r#type: DatabaseType::Sqlite,
//!     url: "database.db".to_string(),
//!     create_if_missing: true,
//! };
//! let db = SqliteUrlDatabase::from_config(&config).await?;
//!
//! // Run migrations to set up the schema
//! db.migrate().await?;
//!
//! // Use the database
//! db.insert_url("abc123", "https://example.com").await?;
//! let url = db.get_url("abc123").await?;
//! # Ok(())
//! # }
//! ```

use super::{DatabaseError, UrlDatabase};
use crate::configuration::DatabaseSettings;
use async_trait::async_trait;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

/// SQLite implementation of the [`UrlDatabase`] trait.
///
/// This struct wraps a SQLite connection pool and provides methods for
/// storing and retrieving URLs. It handles connection management and
/// provides automatic migration capabilities.
///
/// # Thread Safety
///
/// This struct is `Send + Sync` and can be safely used across thread boundaries.
/// The underlying `SqlitePool` is designed for concurrent access.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
/// use url_shortener_ztm_lib::configuration::DatabaseSettings;
/// use url_shortener_ztm_lib::DatabaseType;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseSettings {
///     r#type: DatabaseType::Sqlite,
///     url: "database.db".to_string(),
///     create_if_missing: true,
/// };
/// let db = SqliteUrlDatabase::from_config(&config).await?;
/// # Ok(())
/// # }
/// ```
pub struct SqliteUrlDatabase {
    /// SQLite connection pool for database operations
    pool: SqlitePool,
}

impl SqliteUrlDatabase {
    /// Creates a new `SqliteUrlDatabase` with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - A configured SQLite connection pool
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use sqlx::SqlitePool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = SqlitePool::connect("sqlite:database.db").await?;
    /// let db = SqliteUrlDatabase::new(pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new `SqliteUrlDatabase` from configuration settings.
    ///
    /// This method sets up the database connection using the provided configuration,
    /// including creating the database file if specified and the file doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `config` - Database configuration settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(SqliteUrlDatabase)` if the connection is successfully established,
    /// or `Err(DatabaseError::ConnectionError)` if connection fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings {
    ///     r#type: DatabaseType::Sqlite,
    ///     url: "database.db".to_string(),
    ///     create_if_missing: true,
    /// };
    /// let db = SqliteUrlDatabase::from_config(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_config(config: &DatabaseSettings) -> Result<Self, DatabaseError> {
        let pool = get_connection_pool(config)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        Ok(Self::new(pool))
    }

    /// Runs database migrations to set up the schema.
    ///
    /// This method applies all migration files found in the `./migrations` directory.
    /// Migrations are run automatically and idempotently - running them multiple times
    /// is safe and will not cause errors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if migrations complete successfully, or
    /// `Err(DatabaseError::MigrationError)` if migration fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings { r#type: DatabaseType::Sqlite, url: "database.db".to_string(), create_if_missing: true, }; let db = SqliteUrlDatabase::from_config(&config).await?;
    /// db.migrate().await?; // Set up the database schema
    /// # Ok(())
    /// # }
    /// ```
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

        Ok(())
    }

}

#[async_trait]
impl UrlDatabase for SqliteUrlDatabase {
    /// Stores a URL with the given ID in the SQLite database.
    ///
    /// This implementation uses a prepared statement for type safety and
    /// handles duplicate key constraints by returning a `DatabaseError::Duplicate`.
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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings { r#type: DatabaseType::Sqlite, url: "database.db".to_string(), create_if_missing: true, }; let db = SqliteUrlDatabase::from_config(&config).await?;
    /// db.insert_url("abc123", "https://example.com").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO urls (id, url) VALUES (?, ?)")
            .bind(id)
            .bind(url)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DatabaseError::Duplicate
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        Ok(())
    }

    /// Retrieves a URL by its short ID from the SQLite database.
    ///
    /// This implementation uses a prepared statement with optional result handling
    /// to safely retrieve URLs and handle the case where no matching record exists.
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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings { r#type: DatabaseType::Sqlite, url: "database.db".to_string(), create_if_missing: true, }; let db = SqliteUrlDatabase::from_config(&config).await?;
    /// let url = db.get_url("abc123").await?;
    /// println!("Original URL: {}", url);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_url(&self, id: &str) -> Result<String, DatabaseError> {
        let row = sqlx::query_as::<_, (String,)>("SELECT url FROM urls WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(record) => Ok(record.0),
            None => Err(DatabaseError::NotFound),
        }
    }

    /// Checks if an alias/ID already exists in the SQLite database.
    ///
    /// This implementation uses a simple SELECT query with COUNT to efficiently
    /// check for the presence of an alias without retrieving the actual data.
    ///
    /// # Arguments
    ///
    /// * `id` - The alias/ID to check for existence
    ///
    /// # Returns
    ///
    /// Returns `Ok(bool)` where `true` means the alias exists and `false` means it's available.
    /// Returns an error if a database error occurred.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::database::{SqliteUrlDatabase, UrlDatabase};
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    /// use url_shortener_ztm_lib::DatabaseType;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings {
    ///     r#type: DatabaseType::Sqlite,
    ///     url: "database.db".to_string(),
    ///     create_if_missing: true,
    /// };
    /// let db = SqliteUrlDatabase::from_config(&config).await?;
    /// let exists = db.alias_exists("my-alias").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn alias_exists(&self, id: &str) -> Result<bool, DatabaseError> {
        let row = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM urls WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(row.0 > 0)
    }
}

/// Creates a SQLite connection pool from configuration settings.
///
/// This function sets up the SQLite connection with the appropriate options,
/// including creating the database file if specified in the configuration.
///
/// # Arguments
///
/// * `config` - Database configuration settings
///
/// # Returns
///
/// Returns `Ok(SqlitePool)` if the connection pool is successfully created,
/// or `Err(sqlx::Error)` if connection setup fails.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::DatabaseType;
/// use url_shortener_ztm_lib::database::sqlite::get_connection_pool;
/// use url_shortener_ztm_lib::configuration::DatabaseSettings;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseSettings {
///     r#type: DatabaseType::Sqlite,
///     url: "database.db".to_string(),
///     create_if_missing: true,
/// };
/// let pool = get_connection_pool(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&config.connection_string())?
        .create_if_missing(config.create_if_missing);

    SqlitePool::connect_with(options).await
}
