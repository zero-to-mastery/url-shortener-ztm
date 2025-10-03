//! # API Response Format
//!
//! This module provides a standardized JSON response format for all API endpoints.
//! It ensures consistent response structure across the entire service and simplifies
//! client-side error handling and data processing.
//!
//! ## Response Format
//!
//! All API responses follow a consistent JSON envelope format:
//!
//! ```json
//! {
//!   "success": true,
//!   "message": "Optional message",
//!   "status": 200,
//!   "time": "2025-01-18T12:00:00Z",
//!   "data": { /* Response data */ }
//! }
//! ```
//!
//! ## Success Responses
//!
//! Success responses include the requested data and appropriate status codes:
//!
//! ```json
//! {
//!   "success": true,
//!   "message": "ok",
//!   "status": 200,
//!   "time": "2025-01-18T12:00:00Z",
//!   "data": "https://localhost:8000/AbC123"
//! }
//! ```
//!
//! ## Error Responses
//!
//! Error responses provide detailed error information:
//!
//! ```json
//! {
//!   "success": false,
//!   "message": "URL not found",
//!   "status": 404,
//!   "time": "2025-01-18T12:00:00Z",
//!   "data": null
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use url_shortener_ztm_lib::response::{ApiResponse, ApiResult};
//! use axum::http::StatusCode;
//!
//! // Create success response
//! let response = ApiResponse::success("Hello, world!");
//!
//! // Create error response
//! let error = ApiResponse::<()>::error("Not found", StatusCode::NOT_FOUND);
//!
//! // Use in handlers
//! fn handler() -> ApiResult<String> {
//!     Ok(ApiResponse::success("Data".to_string()))
//! }
//! ```

use crate::errors::ApiError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Convenience type alias for API handler results.
///
/// This type alias simplifies handler function signatures by combining
/// the success and error types commonly used in API handlers.
///
/// # Examples
///
/// ```rust,no_run
/// use url_shortener_ztm_lib::response::{ApiResponse, ApiResult};
///
/// fn handler() -> ApiResult<String> {
///     Ok(ApiResponse::success("Hello, world!".to_string()))
/// }
/// ```
pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;

/// Standardized API response envelope for all endpoints.
///
/// This struct provides a consistent response format across all API endpoints,
/// making it easier for clients to handle responses and errors uniformly.
/// The response includes metadata such as success status, HTTP status code,
/// timestamp, and optional data or error messages.
///
/// # Generic Type Parameter
///
/// * `T` - The type of data to include in the response (must implement `Serialize`)
///
/// # JSON Structure
///
/// ```json
/// {
///   "success": true,
///   "message": "Optional message",
///   "status": 200,
///   "time": "2025-01-18T12:00:00Z",
///   "data": { /* Response data */ }
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// use url_shortener_ztm_lib::response::ApiResponse;
/// use axum::http::StatusCode;
///
/// // Success response with data
/// let success = ApiResponse::success("Hello, world!");
///
/// // Error response
/// let error = ApiResponse::<()>::error("Not found", StatusCode::NOT_FOUND);
///
/// // Custom status response
/// let created = ApiResponse::success_with_status(StatusCode::CREATED, "Created");
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Indicates whether the request was successful
    pub success: bool,
    /// Optional message providing additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// HTTP status code
    pub status: u16,
    /// Timestamp when the response was generated
    pub time: DateTime<Utc>,
    /// Optional response data (omitted for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response with HTTP 200 status.
    ///
    /// This is the most common success response, indicating that the request
    /// was processed successfully and data is being returned.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to include in the response
    ///
    /// # Returns
    ///
    /// Returns an `ApiResponse` with `success: true`, status 200, and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::response::ApiResponse;
    ///
    /// let response = ApiResponse::success("Hello, world!");
    /// assert!(response.success);
    /// assert_eq!(response.status, 200);
    /// ```
    pub fn success(data: T) -> Self {
        Self::success_with_status(StatusCode::OK, data)
    }

    /// Creates a successful response with a custom HTTP status code.
    ///
    /// This method allows for success responses with non-200 status codes,
    /// such as 201 Created, 202 Accepted, etc.
    ///
    /// # Arguments
    ///
    /// * `status` - The HTTP status code to use
    /// * `data` - The data to include in the response
    ///
    /// # Returns
    ///
    /// Returns an `ApiResponse` with `success: true`, the specified status, and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::response::ApiResponse;
    /// use axum::http::StatusCode;
    ///
    /// let response = ApiResponse::success_with_status(StatusCode::CREATED, "Created");
    /// assert!(response.success);
    /// assert_eq!(response.status, 201);
    /// ```
    pub fn success_with_status(status: StatusCode, data: T) -> Self {
        Self {
            success: true,
            message: Some("ok".into()),
            status: status.as_u16(),
            time: Utc::now(),
            data: Some(data),
        }
    }

    /// Creates an error response with the specified message and status code.
    ///
    /// This method creates a standardized error response that will be returned
    /// to clients when an error occurs during request processing.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to include in the response
    /// * `status` - The HTTP status code to use
    ///
    /// # Returns
    ///
    /// Returns an `ApiResponse` with `success: false`, the specified status, and error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::response::ApiResponse;
    /// use axum::http::StatusCode;
    ///
    /// let response = ApiResponse::<()>::error("Not found", StatusCode::NOT_FOUND);
    /// assert!(!response.success);
    /// assert_eq!(response.status, 404);
    /// assert_eq!(response.message, Some("Not found".to_string()));
    /// ```
    pub fn error(message: &str, status: StatusCode) -> Self {
        Self {
            success: false,
            message: Some(message.to_string()),
            status: status.as_u16(),
            time: Utc::now(),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    /// Converts an `ApiResponse` into an HTTP response.
    ///
    /// This implementation automatically converts the response into an HTTP response
    /// with the appropriate status code and JSON body. The response will have
    /// the correct content-type headers for JSON.
    ///
    /// # Returns
    ///
    /// Returns an HTTP response with:
    /// - The status code from the `ApiResponse`
    /// - JSON body containing the response data
    /// - Appropriate content-type headers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::response::ApiResponse;
    /// use axum::response::IntoResponse;
    ///
    /// let response = ApiResponse::success("Hello, world!");
    /// let http_response = response.into_response();
    /// // Can be used directly as an Axum response
    /// ```
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}
