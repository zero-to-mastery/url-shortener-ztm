// tests/api/health_check.rs

// dependencies
use crate::helpers::spawn_app;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;

#[tokio::test]
async fn health_check_returns_200_ok_and_json_envelope() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .client
        .get(format!("{}/api/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert: HTTP status
    assert!(response.status().is_success());
    assert_eq!(response.status().as_u16(), 200);

    // Assert: content type is JSON
    let ct = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.starts_with("application/json"));

    // Assert: JSON envelope fields
    let body: Value = response.json().await.expect("Response was not valid JSON");

    // success: true
    assert_eq!(body.get("success").and_then(Value::as_bool), Some(true));

    // message: "ok"
    assert_eq!(body.get("message").and_then(Value::as_str), Some("ok"));

    // status: 200
    assert_eq!(body.get("status").and_then(Value::as_u64), Some(200));

    // time: present and is a string (RFC3339 from chrono)
    assert!(body.get("time").and_then(Value::as_str).is_some());

    // data: present and null (because `()` serializes to JSON null)
    assert!(body.get("data").is_some());
    assert!(body.get("data").unwrap().is_null());
}
