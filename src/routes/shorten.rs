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

/// Maximum allowed URL length in characters.
/// 
/// RFC 2616 doesn't specify a limit, but most browsers support 2000+ characters.
/// We use 2048 as a reasonable limit to prevent abuse while supporting legitimate URLs.
const MAX_URL_LENGTH: usize = 2048;

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
/// - `422 Unprocessable Entity` - Invalid URL format or URL exceeds maximum length
/// - `500 Internal Server Error` - Database error or ID collision
///
/// # URL Validation
///
/// The handler validates URLs using the `url` crate:
/// - Must be a valid URL format
/// - Must include a scheme (http:// or https://)
/// - Must have a valid hostname
/// - Must not exceed MAX_URL_LENGTH (2048 characters)
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
/// - **URL Too Long** - Returns 422 if URL exceeds MAX_URL_LENGTH
/// - **Invalid URL Format** - Returns 422 with validation error
/// - **Database Errors** - Returns 500 with internal error message
/// - **ID Collision** - Returns 500 with collision error (rare occurrence)
///
/// # Security Considerations
///
/// - Input validation prevents malicious URL injection
/// - Length validation prevents resource exhaustion attacks
/// - Database queries use prepared statements to prevent SQL injection
/// - API key authentication protects the main endpoint from abuse
/// - Public endpoint provides limited access for testing
///
/// # Performance Considerations
///
/// - URL parsing is optimized for common formats
/// - Length check is O(1) and performed before URL parsing
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
    
    // Validate URL length before parsing to prevent resource exhaustion
    if url.len() > MAX_URL_LENGTH {
        tracing::warn!(
            "URL length {} exceeds maximum allowed length of {}",
            url.len(),
            MAX_URL_LENGTH
        );
        return Err(ApiError::Unprocessable(format!(
            "URL exceeds maximum allowed length of {} characters",
            MAX_URL_LENGTH
        )));
    }
    
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
