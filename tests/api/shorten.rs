// tests/api/shorten.rs
// Integration tests for the URL shortening endpoint
//
// Tests cover:
// - Basic URL shortening functionality
// - URL length validation (max 2048 characters)
// - Edge cases (exact limit, exceeding limit)

use crate::helpers::spawn_app;
use axum::http::StatusCode;
use regex::Regex;

/// Test that the shorten endpoint successfully shortens a valid URL
#[tokio::test]
async fn shorten_endpoint_returns_the_shortened_url_and_200_ok() {
    // Arrange
    let app = spawn_app().await;
    let url = r#"https://www.google.ca"#;

    // Act
    let response = app.post_api_with_key("/api/shorten", url).await;

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("Failed to read response body");
    let hostname = "localhost";
    let pattern = format!(r"^https://{}/[A-Za-z0-9_-]{{6}}\n$", hostname);
    let regex = Regex::new(&pattern).expect("Failed to compile regex");
    assert!(regex.is_match(&body));
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
    assert_eq!(url.len(), 2048, "Test URL should be exactly 2048 characters");

    // Act
    let response = app.post_api_with_key("/api/shorten", &url).await;

    // Assert - URL at max length should be accepted
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected 200 OK for URL at maximum allowed length"
    );
    
    // Verify we got a shortened URL back
    let body = response.text().await.expect("Failed to read response body");
    assert!(
        body.starts_with("https://localhost/"),
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
    assert_eq!(url.len(), 2049, "Test URL should be exactly 2049 characters");

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
    
    assert_eq!(url.len(), 5000, "Test URL should be exactly 5000 characters");

    // Act
    let response = app.post_api_with_key("/api/shorten", &url).await;

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for very long URL"
    );
}
