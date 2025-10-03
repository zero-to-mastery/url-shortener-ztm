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
