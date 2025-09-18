// tests/api/redirect.rs

// integration test which exercises the /redirect endpoint
// this endpoint should redirect the user to the shortened URL

// dependencies
use crate::helpers::spawn_app;
use axum::http::StatusCode;

#[tokio::test]
async fn redirect_endpoint_sends_user_to_shortened_destination_url() {
    // Arrange
    let app = spawn_app().await;

    // Insert a test URL into the database
    let test_id = "tst123";
    let test_url = "https://www.google.com";

    sqlx::query("INSERT INTO urls (id, url) VALUES (?, ?);")
        .bind(test_id)
        .bind(test_url)
        .execute(&app.pool)
        .await
        .expect("Failed to insert test data into database");

    // Act
    let response = app
        .client
        .get(format!("{}/api/redirect/{}", &app.address, test_id))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);

    let location_header = response
        .headers()
        .get("location")
        .expect("No location header found in response");

    assert_eq!(location_header, test_url);
}
