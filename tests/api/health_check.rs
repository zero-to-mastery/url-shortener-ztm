// tests/api/health_check.rs

// dependencies
use crate::helpers::{assert_json_ok, spawn_app};

#[tokio::test]
async fn health_check_returns_200_ok_and_json_envelope() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_api("/api/health_check").await;

    // Assert standard JSON OK envelope
    let body = assert_json_ok(response).await;
    assert!(body.get("data").unwrap().is_null());
}
