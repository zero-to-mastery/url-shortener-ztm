//! # URL Shortening Handler
//!
//! This module provides the URL shortening handler for the URL shortener service.
//! It processes requests to shorten URLs and stores them in the database with
//! unique identifiers.

use crate::database::DatabaseError;
use crate::errors::ApiError;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::extract::State;
use axum_extra::{TypedHeader, headers::Host};
use axum_macros::debug_handler;
use serde::Serialize;
use tracing::instrument;
use url::Url;

/// Maximum allowed URL length in characters.
///
/// RFC 2616 doesn't specify a limit, but most browsers support 2000+ characters.
/// We use 2048 as a reasonable limit to prevent abuse while supporting legitimate URLs.
const MAX_URL_LENGTH: usize = 2048;
const MAX_ID_RETRIES: usize = 8;

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    /// The shortened URL
    pub shortened_url: String,
    /// The original URL that was shortened
    pub original_url: String,
    /// The unique identifier used in the shortened URL
    pub id: String,
}

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
/// Returns a JSON response with the shortened URL information:
///
/// ```json
/// {
///   "success": true,
///   "message": "ok",
///   "status": 200,
///   "time": "2025-01-18T12:00:00Z",
///   "data": {
///     "shortened_url": "https://localhost:8000/AbC123",
///     "original_url": "https://www.example.com/very/long/url",
///     "id": "AbC123"
///   }
/// }
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
/// # Expected response (JSON)
/// {
///   "success": true,
///   "message": "ok",
///   "status": 200,
///   "time": "2025-01-18T12:00:00Z",
///   "data": {
///     "shortened_url": "https://localhost:8000/AbC123",
///     "original_url": "https://www.example.com",
///     "id": "AbC123"
///   }
/// }
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
/// - Response format follows consistent JSON schema for better frontend integration
#[debug_handler]
#[instrument(name = "shorten", skip(state))]
pub async fn post_shorten(
    State(state): State<AppState>,
    TypedHeader(header): TypedHeader<Host>,
    url: String,
) -> Result<ApiResponse<ShortenResponse>, ApiError> {
    // 1) Early length validation to prevent resource exhaustion
    if url.len() > MAX_URL_LENGTH {
        tracing::warn!("URL length {} exceeds max {}", url.len(), MAX_URL_LENGTH);
        return Err(ApiError::Unprocessable(format!(
            "URL exceeds maximum allowed length of {} characters",
            MAX_URL_LENGTH
        )));
    }

    // 2) Parse and normalize the URL (lowercase host, remove fragments, etc.)
    let norm = normalize_url(&url).map_err(|e| {
        tracing::error!("Unable to parse URL: {}", e);
        ApiError::Unprocessable(e.to_string())
    })?;

    let hostname = header.hostname();

    // 3) Fast path: check Bloom filter (long → short).
    // If it may exist, verify with the database.
    if state.blooms.l2s.may_contain(norm.as_str()) {
        match state.database.get_id_by_url(&norm).await {
            Ok(existing_id) => {
                tracing::info!("Hit existing mapping via bloom+db");
                return Ok(make_response(hostname, &existing_id, &norm));
            }
            Err(DatabaseError::NotFound) => {
                // False positive; proceed to insertion path.
            }
            Err(e) => {
                tracing::error!("Database error on get_id: {}", e);
                return Err(ApiError::Internal(e.to_string()));
            }
        }
    }

    // 4) Insert path: generate IDs and retry on duplicate collisions
    let id = insert_with_retry(&state, &norm).await?;

    // 5) Optionally update Bloom filters after successful insertion
    state.blooms.s2l.insert(id.as_str());
    state.blooms.l2s.insert(norm.as_str());

    tracing::info!("URL shortened and saved successfully");
    Ok(make_response(hostname, &id, &norm))
}

/// Parses and normalizes a URL:
/// - Enforces http/https schemes
/// - Removes fragments
/// - Lowercases host
fn normalize_url(raw: &str) -> Result<String, url::ParseError> {
    let mut u = Url::parse(raw)?;
    // Only allow http/https schemes
    match u.scheme() {
        "http" | "https" => {}
        _ => return Err(url::ParseError::RelativeUrlWithoutBase),
    }

    // Remove fragment if any
    u.set_fragment(None);

    // Lowercase host (Url usually does this automatically, but we ensure it)
    if let Some(h) = u.host_str() {
        let lower = h.to_ascii_lowercase();
        if lower != h {
            let _ = u.set_host(Some(&lower));
        }
    }

    // Additional normalization (e.g., trailing slash, query sorting) can be added here
    Ok(u.to_string())
}

/// Inserts a new URL, retrying ID generation if duplicates occur.
/// Relies on the database's Duplicate error to ensure atomicity and avoid TOCTOU issues.
async fn insert_with_retry(state: &AppState, norm_url: &str) -> Result<String, ApiError> {
    for attempt in 0..MAX_ID_RETRIES {
        let id = state.code_generator.generate().map_err(|e| {
            tracing::error!("Code generation error: {:?}", e);
            ApiError::Internal("Code generation failed".to_string())
        })?;

        match state.database.insert_url(id.as_str(), norm_url).await {
            Ok(()) => return Ok(id),
            Err(DatabaseError::Duplicate) => {
                tracing::warn!("ID collision on attempt {} — retrying", attempt + 1);
                continue;
            }
            Err(e) => {
                tracing::error!("Database error on insert: {}", e);
                return Err(ApiError::Internal(e.to_string()));
            }
        }
    }

    tracing::error!("Exhausted ID retries ({} attempts)", MAX_ID_RETRIES);
    Err(ApiError::Internal("ID collision occurred".into()))
}

/// Builds a unified response structure for shortened URLs.
fn make_response(hostname: &str, id: &str, original_url: &str) -> ApiResponse<ShortenResponse> {
    let shortened_url = format!("https://{}/{}", hostname, id);
    let response_data = ShortenResponse {
        shortened_url,
        original_url: original_url.to_string(),
        id: id.to_string(),
    };
    ApiResponse::success(response_data)
}
