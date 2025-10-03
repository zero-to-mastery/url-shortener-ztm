// src/database/postgres.rs

use super::{DatabaseError, UrlDatabase};
use crate::configuration::DatabaseSettings;
use async_trait::async_trait;
use sqlx::{
    Error as SqlxError, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::str::FromStr;

// PostgresUrlDatabase struct
pub struct PostgresUrlDatabase {
    pool: PgPool,
}

impl PostgresUrlDatabase {
    pub fn new(pool: PgPool) -> Self {
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
        sqlx::migrate!("./migrations/pg")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl UrlDatabase for PostgresUrlDatabase {
    // Insert URL with ID
    async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2)")
            .bind(id)
            .bind(url)
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

    // Fetch URL by ID
    async fn get_url(&self, id: &str) -> Result<String, DatabaseError> {
        let row = sqlx::query_as::<_, (String,)>("SELECT url FROM urls WHERE id = $1")
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

pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, SqlxError> {
    let options = PgConnectOptions::from_str(&config.connection_string())?;
    PgPoolOptions::new().connect_with(options).await
}

// ---- helpers ----

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
    use std::time::{SystemTime, UNIX_EPOCH};

    // Integration test for PostgresUrlDatabase
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
        let id = format!(
            "test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_nanos()
        );
        let url = "https://example.com/test";

        // Insert and fetch URL
        db.insert_url(&id, url).await.expect("insert failed");
        let fetched = db.get_url(&id).await.expect("get_url failed");
        assert_eq!(fetched, url);

        // Check duplicate insert
        let duplicate = db.insert_url(&id, url).await;
        assert!(matches!(duplicate, Err(DatabaseError::Duplicate)));

        // Check not found
        let missing = db.get_url("this-id-does-not-exist-hopefully").await;
        assert!(matches!(missing, Err(DatabaseError::NotFound)));

        // Cleanup
        sqlx::query("DELETE FROM urls WHERE id = $1")
            .bind(&id)
            .execute(&pool)
            .await
            .expect("cleanup failed");
    }
}
