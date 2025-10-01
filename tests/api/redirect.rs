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
    let test_id = "tst123";
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
