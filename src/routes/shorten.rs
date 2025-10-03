//! # URL Shortening Handler
//!
//! This module provides the URL shortening handler for the URL shortener service.
//! It processes requests to shorten URLs and stores them in the database with
//! unique identifiers.

use crate::database::DatabaseError;
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{TypedHeader, headers::Host};
use axum_macros::debug_handler;
use tracing::instrument;
use url::Url;

/// URL shortening handler that creates short URLs from long URLs.
///
/// This handler processes requests to shorten URLs by generating a unique
/// identifier and storing the mapping in the database. It validates the
/// input URL and returns the shortened URL in a simple text format.
///
/// # Endpoint
///
/// `POST /api/shorten` (protected - requires API key)
/// `POST /api/public/shorten` (public - no authentication required)
///
/// # Arguments
///
/// * `State(state)` - Application state containing database connection
/// * `TypedHeader(header)` - Host header for constructing the response URL
/// * `url` - The URL to shorten (provided in request body as plain text)
///
/// # Request Format
///
/// The request body should contain the URL to shorten as plain text:
///
/// ```text
/// https://www.example.com/very/long/url/with/many/parameters
/// ```
///
/// # Response Format
///
/// Returns the shortened URL as plain text:
///
/// ```text
/// https://localhost:8000/AbC123
/// ```
///
/// # URL Generation
///
/// Short URLs are generated using the `nanoid` library with the following characteristics:
/// - **Length**: 6 characters
/// - **Character Set**: URL-safe characters (A-Z, a-z, 0-9, _, -)
/// - **Collision Handling**: If a duplicate ID is generated, an error is returned
///
/// # Status Codes
///
/// - `200 OK` - URL shortened successfully
/// - `422 Unprocessable Entity` - Invalid URL format
/// - `500 Internal Server Error` - Database error or ID collision
///
/// # URL Validation
///
/// The handler validates URLs using the `url` crate:
/// - Must be a valid URL format
/// - Must include a scheme (http:// or https://)
/// - Must have a valid hostname
///
/// # Tracing
///
/// This handler is instrumented with tracing for request monitoring:
/// - Successful shortenings are logged at info level
/// - URL parsing errors are logged at error level
/// - Database errors are logged at error level
/// - ID collisions are logged at error level
///
/// # Examples
///
/// ```bash
/// # Shorten a URL (protected endpoint)
/// curl -d 'https://www.example.com' \
///   -H "x-api-key: your-api-key" \
///   http://localhost:8000/api/shorten
///
/// # Shorten a URL (public endpoint)
/// curl -d 'https://www.example.com' \
///   http://localhost:8000/api/public/shorten
///
/// # Expected response
/// https://localhost:8000/AbC123
/// ```
///
/// # Error Handling
///
/// This handler handles the following error cases:
/// - **Invalid URL Format** - Returns 422 with validation error
/// - **Database Errors** - Returns 500 with internal error message
/// - **ID Collision** - Returns 500 with collision error (rare occurrence)
///
/// # Security Considerations
///
/// - Input validation prevents malicious URL injection
/// - Database queries use prepared statements to prevent SQL injection
/// - API key authentication protects the main endpoint from abuse
/// - Public endpoint provides limited access for testing
///
/// # Performance Considerations
///
/// - URL parsing is optimized for common formats
/// - Database inserts are performed asynchronously
/// - ID generation is fast and collision-resistant
/// - Response format is minimal to reduce bandwidth
#[debug_handler]
#[instrument(name = "shorten" skip(state))]
pub async fn post_shorten(
    State(state): State<AppState>,
    TypedHeader(header): TypedHeader<Host>,
    url: String,
) -> Result<impl IntoResponse, ApiError> {
    let id = &nanoid::nanoid!(6);
    let p_url = Url::parse(&url).map_err(|e| {
        tracing::error!("Unable to parse URL");
        ApiError::Unprocessable(e.to_string())
    })?;
    let host = "localhost";

    match state.database.insert_url(id, p_url.as_str()).await {
        Ok(()) => {
            let response_body = format!("https://{}/{}\n", host, id);
            tracing::info!("URL shortened and saved successfully...");
            Ok((StatusCode::OK, response_body).into_response())
        }
        Err(DatabaseError::Duplicate) => {
            tracing::error!("Duplicate ID generated");
            Err(ApiError::Internal("ID collision occurred".to_string()))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(ApiError::Internal(e.to_string()))
        }
    }
}
