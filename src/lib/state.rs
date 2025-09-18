// src/lib/state.rs

// dependencies
use sqlx::SqlitePool;
use uuid::Uuid;

// struct type to represent the application state
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub api_key: Uuid,
    pub template_dir: String,
}

// methods to build the application state
impl AppState {
    // Create a new application state instance
    pub fn new(pool: SqlitePool, api_key: Uuid, template_dir: String) -> Self {
        Self {
            pool,
            api_key,
            template_dir,
        }
    }
}
