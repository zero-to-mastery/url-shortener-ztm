//! # PostgreSQL Database Implementation
//!
//! This module provides the PostgreSQL implementation of the [`UrlDatabase`] trait.
//! It uses the `sqlx` crate for type-safe database operations and automatic migrations.
//!
//! ## Features
//!
//! - **Type-safe queries** — Compile-time SQL query validation (when `sqlx` offline is set up)
//! - **Automatic migrations** — Database schema management via `sqlx::migrate!`
//! - **Connection pooling** — Efficient `PgPool` management
//! - **Detailed error mapping** — Duplicate key detection and friendly errors
//!
//! ## Database Schema
//!
//! The PostgreSQL database uses a simple schema with a single `urls` table:
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS urls (
//!   id  TEXT PRIMARY KEY,            -- Short identifier (nanoid, ~6 characters)
//!   url TEXT NOT NULL                -- Original URL
//! );
//! ```
//!
//! > Migrations for Postgres live under `./migrations/pg`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::DatabaseType;
//! use url_shortener_ztm_lib::database::{PostgresUrlDatabase, UrlDatabase};
//! use url_shortener_ztm_lib::configuration::DatabaseSettings;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create database from configuration
//! let config = DatabaseSettings {
//!     r#type: DatabaseType::Postgres,
//!     url: "postgres://app:secret@localhost:5432/urlshortener".to_string(),
//!     create_if_missing: false, // Not used by Postgres connector
//!     max_connections: Some(16),
//!     min_connections: Some(4),
//! };
//! let db = PostgresUrlDatabase::from_config(&config).await?;
//!
//! // Run migrations to set up the schema (reads from ./migrations/pg)
//! db.migrate().await?;
//!
//! // Use the database
//! db.insert_url("abc123", "https://example.com").await?;
//! let url = db.get_url("abc123").await?;
//! println!("Original URL: {}", url);
//! # Ok(())
//! # }
//! ```
//!
//! ## Thread Safety
//!
//! This struct is `Send + Sync` and can be safely used across thread boundaries.
//! The underlying `PgPool` is designed for concurrent access.

use super::{DatabaseError, UrlDatabase};
use crate::configuration::DatabaseSettings;
use crate::models::{UpsertResult, Urls};
use async_trait::async_trait;
use sqlx::{
    Error as SqlxError, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::str::FromStr;

const MAX_CAP: u32 = 96;
const MIN_CAP: u32 = 2;

/// PostgreSQL implementation of the [`UrlDatabase`] trait.
///
/// This struct wraps a PostgreSQL connection pool and provides methods for
/// storing and retrieving URLs. It handles connection management and
/// provides automatic migration capabilities.
pub struct PostgresUrlDatabase {
    /// PostgreSQL connection pool for database operations
    pool: PgPool,
}

impl PostgresUrlDatabase {
    /// Creates a new `PostgresUrlDatabase` with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - A configured PostgreSQL connection pool
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::database::postgres_sql::PostgresUrlDatabase;
    /// use sqlx::PgPool;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPool::connect("postgres://app:secret@localhost:5432/urlshortener").await?;
    /// let db = PostgresUrlDatabase::new(pool);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new `PostgresUrlDatabase` from configuration settings.
    ///
    /// This method sets up the database connection using the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Database configuration settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(PostgresUrlDatabase)` if the connection is successfully established,
    /// or `Err(DatabaseError::ConnectionError)` if connection fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use url_shortener_ztm_lib::DatabaseType;
    /// use url_shortener_ztm_lib::database::postgres_sql::PostgresUrlDatabase;
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings {
    ///     r#type: DatabaseType::Postgres,
    ///     url: "postgres://app:secret@localhost:5432/urlshortener".to_string(),
    ///     create_if_missing: false,
    ///     max_connections: Some(16),
    ///     min_connections: Some(4),
    /// };
    /// let db = PostgresUrlDatabase::from_config(&config).await?;
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
    /// This method applies all migration files found in the `./migrations/pg` directory.
    /// Migrations are run automatically and idempotently—running them multiple times
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
    /// use url_shortener_ztm_lib::database::PostgresUrlDatabase;
    /// use url_shortener_ztm_lib::configuration::DatabaseSettings;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatabaseSettings {
    ///     r#type: DatabaseType::Postgres,
    ///     url: "postgres://app:secret@localhost:5432/urlshortener".to_string(),
    ///     create_if_missing: false,
    ///     max_connections: Some(16),
    ///     min_connections: Some(4),
    /// };
    /// let db = PostgresUrlDatabase::from_config(&config).await?;
    /// db.migrate().await?; // Set up the database schema
    /// # Ok(())
    /// # }
    /// ```
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations/pg")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl UrlDatabase for PostgresUrlDatabase {
    /// Retrieves the short ID by original URL from the PostgreSQL database.
    async fn get_id_by_url(&self, url: &str) -> Result<Urls, DatabaseError> {
        let row = sqlx::query_as::<_, Urls>(
            "SELECT id, code FROM urls WHERE url_hash = digest($1, 'sha256') LIMIT 1",
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(record) => Ok(record),
            None => Err(DatabaseError::NotFound),
        }
    }
    /// Stores a URL with the given ID in the PostgreSQL database.
    ///
    /// This implementation uses a prepared statement for type safety and
    /// handles duplicate key constraints by returning a `DatabaseError::Duplicate`.
    ///
    /// # Arguments
    ///
    /// * `id` - The short identifier for the URL
    /// * `url` - The original URL to store
    async fn insert_url(&self, code: &str, url: &str) -> Result<(UpsertResult, Urls), DatabaseError> {
        // First, call the existing SQL function to either insert the URL or get the ID if it exists.
        let upsert_result: UpsertResult = sqlx::query_as("SELECT * FROM upsert_url($1, $2)")
            .bind(code)
            .bind(url)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    DatabaseError::Duplicate
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;

        // If a new record was created, the code is the one we just generated.
        if upsert_result.created {
            let urls = Urls { id: upsert_result.id, code: code.to_string() };
            return Ok((upsert_result, urls));
        }

        // If the URL already existed, we need to fetch the original code associated with it.
        let existing_urls: Urls = sqlx::query_as("SELECT id, code FROM urls WHERE id = $1")
            .bind(upsert_result.id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        Ok((upsert_result, existing_urls))
    }

    /// Retrieves a URL by its short ID from the PostgreSQL database.
    ///
    /// Uses a prepared statement with optional result handling
    /// to safely retrieve URLs and handle the case where no matching record exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The short identifier to look up
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` with the original URL if found, or
    /// `Err(DatabaseError::NotFound)` if no record exists.
    async fn get_url(&self, code: &str) -> Result<String, DatabaseError> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT url FROM all_short_codes u WHERE u.code = $1 LIMIT 1;",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        match row {
            Some(record) => Ok(record.0),
            None => Err(DatabaseError::NotFound),
        }
    }

    async fn list_short_codes(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<String>, DatabaseError> {
        let codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM all_short_codes LIMIT $1 OFFSET $2")
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(codes)
    }

    async fn insert_alias(&self, alias_code: &str, code_id: i64) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO aliases (alias, target_id) VALUES ($1, $2)")
            .bind(alias_code)
            .bind(code_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    DatabaseError::Duplicate
                } else {
                    DatabaseError::QueryError(e.to_string())
                }
            })?;
        Ok(())
    }

    async fn load_bloom_snapshot(&self, name: &str) -> Result<Option<Vec<u8>>, DatabaseError> {
        let data = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT data FROM bloom_snapshots WHERE name = $1 LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(data)
    }

    async fn save_bloom_snapshot(&self, name: &str, data: &[u8]) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
                INSERT INTO bloom_snapshots (name, data)
                VALUES ($1, $2)
                ON CONFLICT (name)
                DO UPDATE
                SET data = EXCLUDED.data,
                    updated_at = NOW()
            "#,
        )
        .bind(name)
        .bind(data)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        Ok(())
    }
}

/// Creates a PostgreSQL connection pool from configuration settings.
///
/// # Arguments
///
/// * `config` - Database configuration settings
///
/// # Returns
///
/// Returns `Ok(PgPool)` if the connection pool is successfully created,
/// or `Err(sqlx::Error)` if connection setup fails.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::DatabaseType;
/// use url_shortener_ztm_lib::database::postgres_sql::get_connection_pool;
/// use url_shortener_ztm_lib::configuration::DatabaseSettings;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseSettings {
///     r#type: DatabaseType::Postgres,
///     url: "postgres://app:secret@localhost:5432/urlshortener".to_string(),
///     create_if_missing: false,
///     max_connections: Some(16),
///     min_connections: Some(4),
/// };
/// let pool = get_connection_pool(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, SqlxError> {
    let options = PgConnectOptions::from_str(&config.connection_string())?;

    let cores = num_cpus::get().max(1) as u32;

    let default_max = cores.saturating_mul(4);
    let default_min = cores.saturating_mul(2);

    let mut max_conn = config.max_connections.unwrap_or(default_max);
    let mut min_conn = config.min_connections.unwrap_or(default_min);

    max_conn = max_conn.clamp(MIN_CAP, MAX_CAP);
    if min_conn < MIN_CAP {
        min_conn = MIN_CAP;
    }

    if min_conn > max_conn {
        tracing::warn!(
            requested_min = %min_conn,
            requested_max = %max_conn,
            "min_connections > max_connections, adjusting min_connections to max_connections"
        );
        min_conn = max_conn;
    }

    tracing::warn!(cores = %cores, min_connections = %min_conn, max_connections = %max_conn, "Postgres pool sizes");

    PgPoolOptions::new()
        .max_connections(max_conn)
        .min_connections(min_conn)
        .connect_with(options)
        .await
}

// ---- helpers ----

/// Returns true if the provided `sqlx::Error` corresponds to a unique
/// constraint violation (PostgreSQL error code `23505`).
fn is_unique_violation(e: &SqlxError) -> bool {
    if let SqlxError::Database(db_err) = e {
        db_err.code().map(|c| c == "23505").unwrap_or(false)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;

    /// Integration test for `PostgresUrlDatabase`.
    ///
    /// This test is ignored by default; run it explicitly when a Postgres instance is available.
    #[tokio::test]
    #[ignore]
    async fn postgres_database_insert_get() {
        // Use DATABASE_URL env or fallback to default
        let default_url = "postgres://app:secret@localhost:5432/urlshortener";
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| default_url.to_string());

        // Connect to Postgres
        let pool = PgPool::connect(&database_url)
            .await
            .expect("failed to connect to Postgres");

        let db = PostgresUrlDatabase::new(pool.clone());

        // Run migrations
        db.migrate().await.expect("migrations failed");

        // Generate unique test id
        let code = "Abc123";
        let url = "https://example.com/test";

        // Insert and fetch URL
        db.insert_url(code, url).await.expect("insert failed");
        let fetched = db.get_url(code).await.expect("get_url failed");
        assert_eq!(fetched, url);

        // Check duplicate insert
        let (duplicate_result, _) = db.insert_url(code, url).await.unwrap();
        assert!(
            !duplicate_result.created,
            "duplicate insert should not create a new record"
        );

        // Check not found
        let missing = db.get_url("this-id-does-not-exist-hopefully").await;
        assert!(matches!(missing, Err(DatabaseError::NotFound)));

        // Cleanup
        sqlx::query("DELETE FROM urls WHERE code = $1")
            .bind(code)
            .execute(&pool)
            .await
            .expect("cleanup failed");
    }
}
