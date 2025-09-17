// src/lib/middleware.rs

// dependencies
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

// Middleware to check for a valid API key in the request headers
pub async fn check_api_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let api_key: &Uuid = state.api_key.as_ref();

    let provided_api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s.trim()).ok());

    if provided_api_key.as_ref() == Some(api_key) {
        next.run(request).await
    } else {
        ApiResponse::<()>::error("Unauthorized", StatusCode::UNAUTHORIZED).into_response()
    }
}
