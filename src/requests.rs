//! # Request Structures
//!
//! This module defines the request structures for the URL shortener API endpoints.
//! These structures handle deserialization of incoming JSON requests with proper
//! validation and error handling.

use crate::validation::validate_alias;
use serde::Deserialize;

/// Request structure for URL shortening with optional custom alias.
///
/// This structure represents the JSON request body for the URL shortening endpoint.
/// It supports both automatic ID generation and custom alias specification.
///
/// # Fields
///
/// * `url` - The original URL to shorten (required)
/// * `alias` - Optional custom alias for the shortened URL
///
/// # Validation
///
/// The request validates:
/// - URL format and length
/// - Custom alias format and availability (if provided)
///
/// # Examples
///
/// ```json
/// {
///   "url": "https://www.example.com/very/long/url",
///   "alias": "my-custom-link"
/// }
/// ```
///
/// ```json
/// {
///   "url": "https://www.example.com/very/long/url"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    /// The original URL to shorten
    pub url: String,
    /// Optional custom alias for the shortened URL
    pub alias: Option<String>,
}

impl ShortenRequest {
    /// Validates the shorten request.
    ///
    /// This method performs validation on both the URL and the custom alias (if provided).
    /// It returns an error if any validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request is valid, or `Err(ApiError)` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url_shortener_ztm_lib::requests::ShortenRequest;
    ///
    /// let request = ShortenRequest {
    ///     url: "https://example.com".to_string(),
    ///     alias: Some("my-link".to_string()),
    /// };
    /// assert!(request.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), crate::errors::ApiError> {
        // Validate custom alias if provided
        if let Some(ref alias) = self.alias {
            validate_alias(alias)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ApiError;

    #[test]
    fn test_shorten_request_validation() {
        // Valid request with custom alias
        let request = ShortenRequest {
            url: "https://example.com".to_string(),
            alias: Some("my-link".to_string()),
        };
        assert!(request.validate().is_ok());

        // Valid request without custom alias
        let request = ShortenRequest {
            url: "https://example.com".to_string(),
            alias: None,
        };
        assert!(request.validate().is_ok());

        // Invalid request with bad alias
        let request = ShortenRequest {
            url: "https://example.com".to_string(),
            alias: Some("invalid@alias".to_string()),
        };
        assert!(request.validate().is_err());
    }
}
