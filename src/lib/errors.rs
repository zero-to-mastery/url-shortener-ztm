// src/lib/errors.rs

// dependencies
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

// enum type to represent the possible error variants for the API
#[derive(thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
}

// implement the IntoResponse trait for the ApiError type
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ApiError::BadRequest(err) => (StatusCode::BAD_REQUEST, format!("Bad Request: {}", err)),
            ApiError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", err),
            ),
            ApiError::NotFound(err) => (StatusCode::NOT_FOUND, format!("Not Found: {}", err)),
        };
        tracing::error!("Error occurred: {:?}", self);
        (status, msg).into_response()
    }
}

// implement the Debug trait for ApiError
impl std::fmt::Debug for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// utility function to presereve the error chain in the error message
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
