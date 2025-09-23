// src/lib/state.rs

// dependencies
use crate::database::UrlDatabase;
use axum_macros::FromRef;
use std::sync::Arc;
use uuid::Uuid;

// struct type to represent the application state
#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: Arc<dyn UrlDatabase>,
    pub api_key: Uuid,
    pub template_dir: String,
}

// methods to build the application state
impl AppState {
    // Create a new application state instance
    pub fn new(database: Arc<dyn UrlDatabase>, api_key: Uuid, template_dir: String) -> Self {
        Self {
            database,
            api_key,
            template_dir,
        }
    }
}
