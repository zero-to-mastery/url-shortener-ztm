// tests/api/shorten.rs

// integration test which exercises the /redirect endpoint
// this endpoint should redirect the user to the shortened URL

// dependencies
use crate::helpers::spawn_app;
use axum::http::StatusCode;
use regex::Regex;

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

// Helper to generate a URL with a path of a specific total length when combined
// with a base like "https://a.com/".
fn make_url_with_total_len(total_len: usize) -> String {
    let base = "https://a.com/"; // length 13
    assert!(total_len >= base.len());
    let remaining = total_len - base.len();
    let path = "a".repeat(remaining);
    format!("{}{}", base, path)
}

#[tokio::test]
async fn shorten_accepts_url_of_exact_max_length() {
    // Arrange
    let app = spawn_app().await;
    // MAX_URL_LENGTH from handler is 2048
    let url = make_url_with_total_len(2048);

    // Act
    let response = app.post_api_with_key("/api/shorten", url).await;

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn shorten_rejects_url_exceeding_max_length() {
    // Arrange
    let app = spawn_app().await;
    // Create a URL of length 2049
    let url = make_url_with_total_len(2049);

    // Act
    let response = app.post_api_with_key("/api/shorten", url).await;

    // Assert
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
