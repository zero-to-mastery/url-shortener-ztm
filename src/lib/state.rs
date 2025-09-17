// src/lib/state.rs

// dependencies
use sqlx::SqlitePool;
use uuid::Uuid;

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub api_key: Uuid,
}

// methods to build the application state
impl AppState {
    // Create a new application state instance
    pub fn new(pool: SqlitePool, api_key: Uuid) -> Self {
        Self { pool, api_key }
    }
}
