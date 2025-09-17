// src/lib/routes/health_check.rs

// dependencies
use axum::{http::StatusCode, response::IntoResponse};

// health_check handler
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
