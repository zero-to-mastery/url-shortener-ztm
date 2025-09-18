// src/database/sqlite.rs

// dependencies
use super::{DatabaseError, UrlDatabase};
use crate::configuration::DatabaseSettings;
use async_trait::async_trait;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

// SqliteUrlDatabase struct
pub struct SqliteUrlDatabase {
    pool: SqlitePool,
}

// implementation methods for SqliteUrlDatabase
impl SqliteUrlDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // Create from config
    pub async fn from_config(config: &DatabaseSettings) -> Result<Self, DatabaseError> {
        let pool = get_connection_pool(config)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        Ok(Self::new(pool))
    }

    // Run migrations
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

        Ok(())
    }
}

// implement UrlDatabase trait for SqliteUrlDatabase
#[async_trait]
impl UrlDatabase for SqliteUrlDatabase {
    // Insert URL with ID
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

    // Fetch URL by ID
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
}

// function to get connection pool
pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&config.connection_string())?
        .create_if_missing(config.create_if_missing);

    SqlitePool::connect_with(options).await
}
