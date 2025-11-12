//! # Error Handling
//!
//! This module provides comprehensive error handling for the URL shortener service.
//! It defines custom error types and implements automatic conversion to HTTP responses
//! with appropriate status codes and error messages.
//!
//! ## Error Types
//!
//! The [`ApiError`] enum represents all possible API errors that can occur during
//! request processing. Each variant maps to a specific HTTP status code and provides
//! detailed error information.
//!
//! ## Error Response Format
//!
//! All errors are automatically converted to standardized JSON responses using the
//! [`ApiResponse`] envelope format:
//!
//! ```json
//! {
//!   "success": false,
//!   "message": "Error description",
//!   "status": 400,
//!   "time": "2025-01-18T12:00:00Z",
//!   "data": null
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::errors::ApiError;
//! use axum::response::IntoResponse;
//!
//! // Return errors from handlers
//! fn handler() -> Result<String, ApiError> {
//!     Err(ApiError::NotFound("Resource not found".to_string()))
//! }
//!
//! // Errors are automatically converted to HTTP responses
//! let response = handler().into_response();
//! ```

use crate::response::ApiResponse;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// API error variants with corresponding HTTP status codes.
///
/// This enum represents all possible errors that can occur during API request processing.
/// Each variant automatically maps to an appropriate HTTP status code and provides
/// detailed error information for debugging and client feedback.
///
/// # Error Variants
///
/// - `BadRequest` - Client sent invalid request data (400)
/// - `NotFound` - Requested resource was not found (404)
/// - `Unauthorized` - Authentication required or failed (401)
/// - `Forbidden` - Access denied (403)
/// - `Conflict` - Resource conflict (409)
/// - `Internal` - Server internal error (500)
/// - `Unprocessable` - Request data is valid but cannot be processed (422)
/// - `Tera` - Template rendering error (500)
///
/// # Examples
///
/// ```rust
/// use url_shortener_ztm_lib::errors::ApiError;
///
/// // Create different types of errors
/// let bad_request = ApiError::BadRequest("Invalid URL format".to_string());
/// let not_found = ApiError::NotFound("URL not found".to_string());
/// let internal = ApiError::Internal("Database connection failed".to_string());
/// ```
#[derive(thiserror::Error)]
pub enum ApiError {
    #[error("cooldown not finished")]
    Cooldown,
    #[error("already have an active challenge")]
    AlreadyActive,
    #[error("email already taken")]
    EmailTaken,
    #[error("challenge expired or invalid")]
    InvalidOrExpired,

    /// Bad request error - client sent invalid data
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Not found error - requested resource doesn't exist
    #[error("Not found: {0}")]
    NotFound(String),

    /// Unauthorized error - authentication required or failed
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden error - access denied
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Conflict error - resource conflict
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Internal server error - unexpected server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Unprocessable entity error - valid data that cannot be processed
    #[error("Unprocessable entity: {0}")]
    Unprocessable(String),

    /// Template rendering error from Tera
    #[error(transparent)]
    Tera(#[from] tera::Error),
}

impl IntoResponse for ApiError {
    /// Converts an `ApiError` into an HTTP response with appropriate status code.
    ///
    /// This implementation automatically maps each error variant to its corresponding
    /// HTTP status code and wraps the error message in a standardized JSON response
    /// using the [`ApiResponse`] envelope format.
    ///
    /// # Returns
    ///
    /// Returns an HTTP response with:
    /// - Appropriate status code based on error type
    /// - JSON body with error details
    /// - Proper content-type headers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::errors::ApiError;
    /// use axum::response::IntoResponse;
    ///
    /// let error = ApiError::NotFound("URL not found".to_string());
    /// let response = error.into_response();
    /// // Response will have 404 status and JSON error body
    /// ```
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Cooldown => (
                StatusCode::TOO_MANY_REQUESTS,
                "Cooldown not finished".into(),
            ),
            ApiError::AlreadyActive => (
                StatusCode::BAD_REQUEST,
                "Already have an active challenge".into(),
            ),
            ApiError::EmailTaken => (StatusCode::BAD_REQUEST, "Email already taken".into()),
            ApiError::InvalidOrExpired => (
                StatusCode::BAD_REQUEST,
                "Challenge expired or invalid".into(),
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Tera(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Tera template rendering error: {msg}"),
            ),
        };

        ApiResponse::<()>::error(&message, status).into_response()
    }
}

impl std::fmt::Debug for ApiError {
    /// Provides detailed debug information including the error chain.
    ///
    /// This implementation uses [`error_chain_fmt`] to display the full error chain,
    /// making it easier to debug complex error scenarios where errors are wrapped
    /// or chained together.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Formats an error with its complete error chain for debugging.
///
/// This utility function displays an error and all its underlying causes in a
/// readable format, making it easier to understand complex error scenarios.
///
/// # Arguments
///
/// * `e` - The error to format
/// * `f` - The formatter to write to
///
/// # Returns
///
/// Returns `fmt::Result` indicating success or failure of the formatting operation.
///
/// # Internal Usage
///
/// This function is used internally by the `Debug` implementation for `ApiError`
/// and is not intended for direct use by library consumers.
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
