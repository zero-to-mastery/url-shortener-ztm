//! # URL Shortening Handler
//!
//! This module provides the URL shortening handler for the URL shortener service.
//! It processes requests to shorten URLs and stores them in the database with
//! unique identifiers.

use crate::database::DatabaseError;
use crate::errors::ApiError;
use crate::requests::ShortenRequest;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{extract::State, Json};
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
/// This handler processes requests to shorten URLs by either generating a unique
/// identifier or using a custom alias provided by the user. It validates the
/// input URL and alias, then stores the mapping in the database.
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
/// * `Json(request)` - JSON request containing URL and optional custom alias
///
/// # Request Format
///
/// The request body should contain JSON with the URL and optional alias:
///
/// ```json
/// {
///   "url": "https://www.example.com/very/long/url",
///   "alias": "my-custom-link"
/// }
/// ```
///
/// Or without custom alias:
///
/// ```json
/// {
///   "url": "https://www.example.com/very/long/url"
/// }
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
///     "shortened_url": "https://localhost:8000/my-custom-link",
///     "original_url": "https://www.example.com/very/long/url",
///     "id": "my-custom-link"
///   }
/// }
/// ```
///
/// # URL Generation
///
/// - **Custom Alias**: If provided, validates format and availability
/// - **Auto-generated**: Uses nanoid library with 6 characters (A-Z, a-z, 0-9, _, -)
/// - **Collision Handling**: Returns error if alias is already in use
///
/// # Status Codes
///
/// - `200 OK` - URL shortened successfully
/// - `422 Unprocessable Entity` - Invalid URL format, alias format, or alias already exists
/// - `500 Internal Server Error` - Database error or ID collision
///
/// # Validation
///
/// The handler validates:
/// - URL format and length (max 2048 characters)
/// - Custom alias format (A-Z, a-z, 0-9, _, - only)
/// - Custom alias availability (not already in use)
/// - Reserved aliases (admin, api, etc.)
///
/// # Examples
///
/// ```bash
/// # Shorten with custom alias (protected endpoint)
/// curl -X POST http://localhost:8000/api/shorten \
///   -H "x-api-key: your-api-key" \
///   -H "Content-Type: application/json" \
///   -d '{"url": "https://www.example.com", "alias": "my-link"}'
///
/// # Shorten without custom alias (public endpoint)
/// curl -X POST http://localhost:8000/api/public/shorten \
///   -H "Content-Type: application/json" \
///   -d '{"url": "https://www.example.com"}'
/// ```
///
/// # Error Handling
///
/// This handler handles the following error cases:
/// - **URL Too Long** - Returns 422 if URL exceeds MAX_URL_LENGTH
/// - **Invalid URL Format** - Returns 422 with validation error
/// - **Invalid Alias Format** - Returns 422 with alias validation error
/// - **Alias Already Exists** - Returns 422 if custom alias is taken
/// - **Reserved Alias** - Returns 422 if trying to use system-reserved alias
/// - **Database Errors** - Returns 500 with internal error message
/// - **ID Collision** - Returns 500 with collision error (rare occurrence)
#[debug_handler]
#[instrument(name = "shorten" skip(state))]
pub async fn post_shorten(
    State(state): State<AppState>,
    TypedHeader(header): TypedHeader<Host>,
    Json(request): Json<ShortenRequest>,
) -> Result<ApiResponse<ShortenResponse>, ApiError> {
    // Validate the request (including custom alias if provided)
    request.validate()?;

    // Validate URL length before parsing to prevent resource exhaustion
    if request.url.len() > MAX_URL_LENGTH {
        tracing::warn!(
            "URL length {} exceeds maximum allowed length of {}",
            request.url.len(),
            MAX_URL_LENGTH
        );
        return Err(ApiError::Unprocessable(format!(
            "URL exceeds maximum allowed length of {} characters",
            MAX_URL_LENGTH
        )));
    }

    let p_url = Url::parse(&request.url).map_err(|e| {
        tracing::error!("Unable to parse URL: {}", e);
        ApiError::Unprocessable(format!("Invalid URL format: {}", e))
    })?;

    let host = header.hostname();

    // Determine the ID to use
    let id = if let Some(ref alias) = request.alias {
        // Check if custom alias is available
        let exists = state.database.alias_exists(alias).await.map_err(|e| {
            tracing::error!("Database error checking alias availability: {}", e);
            ApiError::Internal("Failed to check alias availability".to_string())
        })?;

        if exists {
            return Err(ApiError::Unprocessable(format!(
                "Alias '{}' is already in use",
                alias
            )));
        }

        alias.clone()
    } else {
        // Generate random ID
        state.code_generator.generate().map_err(|e| {
            tracing::error!("Code generation error: {:?}", e);
            ApiError::Internal("Code generation failed".to_string())
        })?
    };

    // Insert URL into database
    match state.database.insert_url(&id, p_url.as_str()).await {
        Ok(()) => {
            let shortened_url = format!("https://{}/{}", host, id);
            let response_data = ShortenResponse {
                shortened_url,
                original_url: request.url.clone(),
                id,
            };
            tracing::info!("URL shortened and saved successfully with ID: {}", response_data.id);
            Ok(ApiResponse::success(response_data))
        }
        Err(DatabaseError::Duplicate) => {
            tracing::error!("Duplicate ID generated or custom alias already exists");
            Err(ApiError::Internal("ID collision occurred".to_string()))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(ApiError::Internal(e.to_string()))
        }
    }
}
