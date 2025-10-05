// tests/api/redirect.rs

// integration test which exercises the /redirect endpoint
// this endpoint should redirect the user to the shortened URL

// dependencies
use crate::helpers::{assert_redirect_to, spawn_app};
use axum::http::StatusCode;

#[tokio::test]
async fn redirect_endpoint_sends_user_to_shortened_destination_url() {
    // Arrange
    let app = spawn_app().await;

    // Insert a test URL into the database
    let test_id = "tstA123";
    let test_url = "https://www.google.com";

    // Insert the test data into the database
    app.database
        .insert_url(test_id, test_url)
        .await
        .expect("Failed to insert test data into database");

    // Act
    let response = app.get_api(&format!("/api/redirect/{}", test_id)).await;

    // Assert
    assert_redirect_to(response, test_url, StatusCode::PERMANENT_REDIRECT).await;
}

#[tokio::test]
async fn redirect_rejects_invalid_characters() {
    let app = spawn_app().await;

    let invalid_id = "tst$12";

    let response = app.get_api(&format!("/api/redirect/{}", invalid_id)).await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn redirect_rejects_invalid_length() {
    // Arrange
    let app = spawn_app().await;

    // Read configured shortener length
    let config = url_shortener_ztm_lib::get_configuration().expect("Failed to read configuration");
    let len = config.shortener.length;

    // Build ids that are one shorter and one longer than configured length.
    // Use 'a' which is within allowed alphabet.
    let shorter = "a".repeat(len.saturating_sub(1));
    let longer = "a".repeat(len + 1);

    // Act
    let resp_short = app.get_api(&format!("/api/redirect/{}", shorter)).await;
    let resp_long = app.get_api(&format!("/api/redirect/{}", longer)).await;

    // Assert - both should be rejected as Not Found (handler validates length before DB lookup)
    assert_eq!(
        resp_short.status().as_u16(),
        404,
        "Expected 404 for id with length -1"
    );
    assert_eq!(
        resp_long.status().as_u16(),
        404,
        "Expected 404 for id with length +1"
    );
}
