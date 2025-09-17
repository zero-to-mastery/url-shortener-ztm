// src/lib/routes/health_check.rs

// dependencies
use crate::response::ApiResponse;

// handler function for the health check endpoint
pub async fn health_check() -> ApiResponse<()> {
    ApiResponse::success(())
}
