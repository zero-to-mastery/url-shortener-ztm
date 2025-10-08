// tests/api/shorten.rs
// Integration tests for the URL shortening endpoint
//
// Tests cover:
// - Basic URL shortening functionality
// - URL length validation (max 2048 characters)
// - Edge cases (exact limit, exceeding limit)

use crate::helpers::{assert_json_ok, spawn_app};
use axum::http::StatusCode;
use regex::Regex;
use serde_json::json;

/// Test that the shorten endpoint successfully shortens a valid URL
#[tokio::test]
async fn shorten_endpoint_returns_the_shortened_url_and_200_ok() {
    // Arrange
    let app = spawn_app().await;
    let url = r#"https://www.google.ca"#;
    let request_body = json!({
        "url": url
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert - Check that we get a valid JSON API response
    let body = assert_json_ok(response).await;

    // Extract and verify the shortened URL from the data field
    let data = body.get("data").expect("Response should have data field");
    let shortened_url = data
        .get("shortened_url")
        .and_then(|v| v.as_str())
        .expect("Response should have shortened_url field");

    // Verify the shortened URL format (nanoid generates variable length)
    let hostname = "localhost";
    let pattern = format!(r"^https://{}/[A-Za-z0-9_-]+$", hostname);
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
    let request_body = json!({
        "url": url
    });
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

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
    let request_body = json!({
        "url": url
    });
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

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
    let request_body = json!({
        "url": url
    });
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for very long URL"
    );
}

/// Test that custom aliases work correctly
#[tokio::test]
async fn shorten_endpoint_accepts_valid_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "my-custom-link";
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert
    let body = assert_json_ok(response).await;
    let data = body.get("data").expect("Response should have data field");

    let shortened_url = data
        .get("shortened_url")
        .and_then(|v| v.as_str())
        .expect("Response should have shortened_url field");

    let id = data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Response should have id field");

    assert_eq!(shortened_url, format!("https://localhost/{}", alias));
    assert_eq!(id, alias);
}

/// Test that custom aliases are rejected if already in use
#[tokio::test]
async fn shorten_endpoint_rejects_duplicate_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url1 = "https://www.example1.com";
    let url2 = "https://www.example2.com";
    let alias = "duplicate-alias";

    // First request
    let request_body1 = json!({
        "url": url1,
        "alias": alias
    });

    let response1 = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body1)
        .send()
        .await
        .expect("Failed to execute POST request");

    assert_json_ok(response1).await;

    // Second request with same alias
    let request_body2 = json!({
        "url": url2,
        "alias": alias
    });

    let response2 = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body2)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert - Second request should be rejected
    assert_eq!(
        response2.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for duplicate alias"
    );

    let body = response2
        .text()
        .await
        .expect("Failed to read response body");
    assert!(
        body.contains("already in use"),
        "Expected error message about alias being in use, got: {}",
        body
    );
}

/// Test that reserved custom aliases are rejected
#[tokio::test]
async fn shorten_endpoint_rejects_reserved_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "admin"; // reserved word
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert - Should be rejected
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for reserved alias: '{}'",
        alias
    );
}

/// Test that custom aliases with invalid characters are rejected
#[tokio::test]
async fn shorten_endpoint_rejects_invalid_characters_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "invalid@alias"; // invalid character
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert - Should be rejected
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for invalid character alias: '{}'",
        alias
    );
}

/// Test that too long custom aliases are rejected
#[tokio::test]
async fn shorten_endpoint_rejects_too_long_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "a".repeat(51); // too long
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert - Should be rejected
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for too long alias"
    );
}

/// Test that maximum length custom aliases are accepted
#[tokio::test]
async fn shorten_endpoint_accepts_maximum_length_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "a".repeat(50); // maximum length
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert
    let body = assert_json_ok(response).await;
    let data = body.get("data").expect("Response should have data field");
    let id = data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Response should have id field");

    assert_eq!(id, alias);
}

/// Test that minimum length custom aliases are accepted
#[tokio::test]
async fn shorten_endpoint_accepts_minimum_length_custom_alias() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://www.example.com";
    let alias = "a"; // minimum length
    let request_body = json!({
        "url": url,
        "alias": alias
    });

    // Act
    let response = app
        .client
        .post(app.url("/api/shorten"))
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute POST request");

    // Assert
    let body = assert_json_ok(response).await;
    let data = body.get("data").expect("Response should have data field");
    let id = data
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Response should have id field");

    assert_eq!(id, alias);
}
