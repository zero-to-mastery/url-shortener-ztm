// tests/api/shorten.rs
// Integration tests for the URL shortening endpoint
//
// Tests cover:
// - Basic URL shortening functionality
// - URL length validation (max 2048 characters)
// - Edge cases (exact limit, exceeding limit)
// - URL normalization and slash validation

use crate::helpers::{assert_json_ok, spawn_app};
use axum::http::StatusCode;
use regex::Regex;
use url_shortener_ztm_lib::routes::shorten::normalize_url;

/// Test that the shorten endpoint successfully shortens a valid URL
#[tokio::test]
async fn shorten_endpoint_returns_the_shortened_url_and_200_ok() {
    // Arrange
    let app = spawn_app().await;
    let url = r#"https://www.google.ca"#;

    // Act
    let response = app.post_api_with_key("/api/shorten", url).await;

    // Assert - Check that we get a valid JSON API response
    let body = assert_json_ok(response).await;

    // Extract and verify the shortened URL from the data field
    let data = body.get("data").expect("Response should have data field");
    let shortened_url = data
        .get("shortened_url")
        .and_then(|v| v.as_str())
        .expect("Response should have shortened_url field");

    // Verify the shortened URL format
    let hostname = "localhost";
    let pattern = format!(r"^https://{}/[A-Za-z0-9]{{7}}$", hostname);
    let regex = Regex::new(&pattern).expect("Failed to compile regex");
    assert!(regex.is_match(shortened_url));
}

/// Helper function to generate a URL of a specific total length.
///
/// Creates a URL with the pattern "https://example.com/<padding>"
/// where <padding> is repeated 'a' characters to reach the desired length.
///
/// # Arguments
/// * `total_len` - The desired total length of the URL in characters
///
/// # Returns
/// A URL string of exactly `total_len` characters
fn make_url_with_total_len(total_len: usize) -> String {
    let base = "https://example.com/";
    assert!(
        total_len >= base.len(),
        "Total length must be at least {} (base URL length)",
        base.len()
    );
    let padding_len = total_len - base.len();
    let padding = "a".repeat(padding_len);
    format!("{}{}", base, padding)
}

/// Test that URLs at exactly the maximum allowed length (2048 characters) are accepted
#[tokio::test]
async fn shorten_accepts_url_at_exact_max_length() {
    // Arrange
    let app = spawn_app().await;
    let url = make_url_with_total_len(2048);

    // Verify our helper created the right length
    assert_eq!(
        url.len(),
        2048,
        "Test URL should be exactly 2048 characters"
    );

    // Act
    let response = app.post_api_with_key("/api/shorten", &url).await;

    // Assert - URL at max length should be accepted
    let body = assert_json_ok(response).await;

    // Verify shortened URL is present in the response
    let data = body.get("data").expect("Response should have data field");
    let shortened_url: &str = data
        .get("shortened_url")
        .and_then(|v| v.as_str())
        .expect("Response should have shortened_url field");

    assert!(
        shortened_url.starts_with("https://localhost/"),
        "Expected shortened URL in response"
    );
}

/// Test that URLs exceeding the maximum allowed length (2049+ characters) are rejected
#[tokio::test]
async fn shorten_rejects_url_exceeding_max_length() {
    // Arrange
    let app = spawn_app().await;
    let url = make_url_with_total_len(2049);

    // Verify our helper created the right length
    assert_eq!(
        url.len(),
        2049,
        "Test URL should be exactly 2049 characters"
    );

    // Act
    let response = app.post_api_with_key("/api/shorten", &url).await;

    // Assert - URL exceeding max length should be rejected with 422
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 Unprocessable Entity for URL exceeding maximum length"
    );

    // Verify the error message is informative
    let body = response.text().await.expect("Failed to read response body");
    assert!(
        body.contains("exceeds maximum allowed length"),
        "Expected error message about exceeding length limit, got: {}",
        body
    );
}

/// Test that significantly oversized URLs are rejected
#[tokio::test]
async fn shorten_rejects_very_long_url() {
    // Arrange
    let app = spawn_app().await;
    // Test with a URL that's way over the limit
    let url = make_url_with_total_len(5000);

    assert_eq!(
        url.len(),
        5000,
        "Test URL should be exactly 5000 characters"
    );

    // Act
    let response = app.post_api_with_key("/api/shorten", &url).await;

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for very long URL"
    );
}

/// Unit tests for the normalize_url function
/// Tests the slash validation functionality specifically
#[cfg(test)]
mod normalize_url_tests {
    use super::*;
    use url_shortener_ztm_lib::errors::ApiError;

    /// Test that valid HTTP URLs with proper double slashes are accepted
    #[test]
    fn normalize_url_accepts_valid_http_urls() {
        let test_cases = vec![
            "http://example.com",
            "http://example.com/path",
            "http://example.com/path?query=value",
            "http://example.com/path#fragment",
            "http://subdomain.example.com",
            "http://127.0.0.1:8080",
            "http://localhost:3000/api/test",
        ];

        for url in test_cases {
            let result = normalize_url(url);
            assert!(
                result.is_ok(),
                "URL '{}' should be valid, got error: {:?}",
                url,
                result.err()
            );

            let normalized = result.unwrap();
            assert!(
                normalized.starts_with("http://"),
                "Normalized URL should start with http://"
            );
        }
    }

    /// Test that valid HTTPS URLs with proper double slashes are accepted
    #[test]
    fn normalize_url_accepts_valid_https_urls() {
        let test_cases = vec![
            "https://example.com",
            "https://example.com/path",
            "https://example.com/path?query=value",
            "https://example.com/path#fragment",
            "https://subdomain.example.com",
            "https://127.0.0.1:8080",
            "https://localhost:3000/api/test",
        ];

        for url in test_cases {
            let result = normalize_url(url);
            assert!(
                result.is_ok(),
                "URL '{}' should be valid, got error: {:?}",
                url,
                result.err()
            );

            let normalized = result.unwrap();
            assert!(
                normalized.starts_with("https://"),
                "Normalized URL should start with https://"
            );
        }
    }

    /// Test that URLs with missing slashes after scheme are rejected
    #[test]
    fn normalize_url_rejects_urls_with_missing_slashes() {
        let test_cases = vec![
            "http:example.com",   // Missing slashes after http:
            "https:example.com",  // Missing slashes after https:
            "http:/example.com",  // Only one slash
            "https:/example.com", // Only one slash
            "http:",              // Just scheme with no slashes
            "https:",             // Just scheme with no slashes
        ];

        for url in test_cases {
            let result = normalize_url(url);
            assert!(result.is_err(), "URL '{}' should be invalid", url);

            let error = result.unwrap_err();
            assert!(
                matches!(error, ApiError::Unprocessable(_)),
                "Expected ApiError::Unprocessable for URL: '{}'",
                url
            );
        }
    }

    /// Test that URLs with more than 2 slashes after scheme are rejected
    #[test]
    fn normalize_url_rejects_urls_with_too_many_slashes() {
        let test_cases = vec![
            "http:////example.com",   // 4 slashes after http:
            "https://///example.com", // 5 slashes after https:
        ];

        for url in test_cases {
            let result = normalize_url(url);
            assert!(result.is_err(), "URL '{}' should be invalid", url);

            let error = result.unwrap_err();
            assert!(
                matches!(error, ApiError::Unprocessable(_)),
                "Expected ApiError::Unprocessable for URL: '{}'",
                url
            );
        }
    }

    /// Test that non-HTTP/HTTPS schemes are rejected
    #[test]
    fn normalize_url_rejects_other_schemes() {
        let test_cases = vec![
            "ftp://example.com",
            "file:///path/to/file",
            "ws://example.com",
            "wss://example.com",
            "git://example.com",
            "mailto:user@example.com",
            "data:text/plain,Hello",
        ];

        for url in test_cases {
            let result = normalize_url(url);
            assert!(result.is_err(), "URL '{}' should be invalid", url);

            let error = result.unwrap_err();
            assert!(
                matches!(error, ApiError::Unprocessable(_)),
                "Expected ApiError::Unprocessable for URL: '{}'",
                url
            );
        }
    }

    /// Test that URL normalization works correctly (lowercase host, fragment removal)
    #[test]
    fn normalize_url_performs_correct_normalization() {
        // Test lowercase host
        let result = normalize_url("http://Example.COM/path");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://example.com/path");

        // Test fragment removal
        let result = normalize_url("http://example.com/path#fragment");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://example.com/path");

        // Test both lowercase and fragment removal
        let result = normalize_url("http://Example.COM/path#fragment");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://example.com/path");
    }

    /// Test edge cases for URL parsing
    #[test]
    fn normalize_url_handles_edge_cases() {
        // Test with empty path
        let result = normalize_url("http://example.com");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://example.com/");

        // Test with special characters in host
        let result = normalize_url("http://sub-domain.example.com");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://sub-domain.example.com/");

        // Test with port numbers
        let result = normalize_url("http://localhost:8080");
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized, "http://localhost:8080/");
    }
}
