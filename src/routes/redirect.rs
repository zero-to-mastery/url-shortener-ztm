//! # URL Redirect Handler
//!
//! This module provides the URL redirect handler for the URL shortener service.
//! It handles requests to shortened URLs and redirects users to the original URLs.

use crate::database::{DatabaseError, MAX_ALIAS_LENGTH};
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;

/// URL redirect handler that redirects users to the original URL.
///
/// This handler processes requests to shortened URLs and redirects users to
/// the original URLs stored in the database. It uses HTTP 308 Permanent Redirect
/// to ensure proper SEO handling and browser caching.
///
/// # Endpoint
///
/// `GET /api/redirect/{id}`
///
/// # Arguments
///
/// * `State(state)` - Application state containing database connection
/// * `Path(id)` - Short URL identifier extracted from the URL path
///
/// # Returns
///
/// Returns `Ok(Redirect)` with a permanent redirect to the original URL, or
/// `Err(ApiError)` if the URL is not found or there's a database error.
///
/// # Redirect Behavior
///
/// - **HTTP 308 Permanent Redirect** - Indicates that the resource has permanently
///   moved to the new location
/// - **SEO Friendly** - Search engines understand that the short URL is an alias
///   for the original URL
/// - **Browser Caching** - Browsers may cache the redirect for performance
///
/// # Status Codes
///
/// - `308 Permanent Redirect` - URL found and redirect successful
/// - `404 Not Found` - Short URL not found in database
/// - `500 Internal Server Error` - Database error occurred
///
/// # Tracing
///
/// This handler is instrumented with tracing for request monitoring:
/// - Successful redirects are logged at info level
/// - Not found errors are logged at error level
/// - Database errors are logged at error level
///
/// # Examples
///
/// ```bash
/// # Redirect to original URL
/// curl -L http://localhost:8000/api/redirect/AbC123
///
/// # Expected behavior: HTTP 308 redirect to original URL
/// ```
///
/// # Error Handling
///
/// This handler handles the following error cases:
/// - **URL Not Found** - Returns 404 with appropriate error message
/// - **Database Errors** - Returns 500 with internal error message
/// - **Invalid ID Format** - Handled by Axum's path extraction
///
/// # Performance Considerations
///
/// - Database queries are optimized for fast lookups
/// - Redirects are processed asynchronously
/// - Error responses are minimal to reduce bandwidth
#[debug_handler]
#[tracing::instrument(name = "redirect" skip(state))]
pub async fn get_redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate against configured length and alphabet before DB lookup
    // check length (use char count to be safe)
    if id.chars().count() > MAX_ALIAS_LENGTH {
        tracing::info!("rejecting redirect: invalid id length");
        return Err(ApiError::NotFound("URL not found".to_string()));
    }

    // Use precomputed allowed_chars from AppState
    if id.chars().any(|c| !state.allowed_chars.contains(&c)) {
        tracing::info!("rejecting redirect: id contains invalid characters");
        return Err(ApiError::NotFound("URL not found".to_string()));
    }

    if !state.blooms.s2l.may_contain(&id) {
        tracing::info!("rejecting redirect: id is not in the short to long filter");
        return Err(ApiError::NotFound("URL not found".to_string()));
    }

    // Proceed with DB lookup
    match state.database.get_url(&id).await {
        Ok(url) => {
            tracing::info!("shortened URL retrieved, redirecting...");
            Ok(Redirect::permanent(&url))
        }
        Err(DatabaseError::NotFound) => {
            tracing::error!("shortened URL not found in the database...");
            Err(ApiError::NotFound("URL not found".to_string()))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(ApiError::Internal(e.to_string()))
        }
    }
}
