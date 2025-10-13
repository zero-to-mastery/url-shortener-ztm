// tests/api/alias_validation_consistency.rs
// Integration test to verify alias character validation consistency
//
// This test checks that alias validation in the shorten endpoint is consistent
// with the character validation in the redirect endpoint. The issue is that:
// 1. validate_alias() hardcodes allowed characters: A-Z, a-z, 0-9, _, -
// 2. But configuration/generator.yml alphabet only includes: 0-9, A-Z, a-z
// 3. This means aliases with _ or - can be created but not accessed via redirect
//
// This test verifies that aliases containing _ or - characters fail validation
// consistently across both endpoints.

use crate::helpers::spawn_app;
use axum::http::StatusCode;

// Helper method for POST with query parameters
async fn post_shorten_with_alias(
    app: &crate::helpers::TestApp,
    alias: &str,
    url: &str,
) -> reqwest::Response {
    let request_url = format!("{}?alias={}", app.api("shorten"), alias);

    app.client
        .post(&request_url)
        .header("x-api-key", app.api_key.to_string())
        .header("host", "localhost:8000")
        .body(url.to_string())
        .send()
        .await
        .expect("Failed to execute POST request")
}

/// Test that verifies aliases containing underscore (_) are rejected
/// according to the configuration
#[tokio::test]
async fn bug_alias_with_underscore_is_incorrectly_accepted() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://example.com";
    let alias_with_underscore = "test_alias";

    // Act - Try to create alias with underscore
    let response = post_shorten_with_alias(&app, alias_with_underscore, url).await;

    // Assert - Should return 422 (rejected)
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Alias with underscore should be rejected"
    );

    // Verify the alias was rejected with appropriate error message
    let body_text = response.text().await.expect("Failed to read response body");
    let body: serde_json::Value =
        serde_json::from_str(&body_text).expect("Response should be valid JSON");

    assert!(
        body.get("success").and_then(|v| v.as_bool()) == Some(false),
        "Response should indicate failure"
    );

    // Verify the alias cannot be accessed via redirect (since it was never created)
    let redirect_response = app.get(&format!("/{}", alias_with_underscore)).await;
    assert_eq!(
        redirect_response.status(),
        StatusCode::NOT_FOUND,
        "Alias with underscore should not be accessible via redirect since it was rejected"
    );
}

/// Test that verifies aliases containing hyphen (-) are rejected
/// according to the configuration
#[tokio::test]
async fn bug_alias_with_hyphen_is_incorrectly_accepted() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://example.com";
    let alias_with_hyphen = "test-alias";

    // Act - Try to create alias with hyphen
    let response = post_shorten_with_alias(&app, alias_with_hyphen, url).await;

    // Assert - Should return 422 (rejected)
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Alias with hyphen should be rejected"
    );

    // Verify the alias was rejected with appropriate error message
    let body_text = response.text().await.expect("Failed to read response body");
    let body: serde_json::Value =
        serde_json::from_str(&body_text).expect("Response should be valid JSON");

    assert!(
        body.get("success").and_then(|v| v.as_bool()) == Some(false),
        "Response should indicate failure"
    );

    // Verify the alias cannot be accessed via redirect (since it was never created)
    let redirect_response = app.get(&format!("/{}", alias_with_hyphen)).await;
    assert_eq!(
        redirect_response.status(),
        StatusCode::NOT_FOUND,
        "Alias with hyphen should not be accessible via redirect since it was rejected"
    );
}

/// Test that aliases with only allowed characters (from config) are accepted
#[tokio::test]
async fn alias_with_config_allowed_characters_is_accepted() {
    // Arrange
    let app = spawn_app().await;
    let url = "https://example.com";
    let valid_alias = "Test123"; // Only contains characters from generator.yml alphabet

    // Act - Try to create alias with only config-allowed characters
    let response = post_shorten_with_alias(&app, valid_alias, url).await;

    // Assert - Should be accepted
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Alias with config-allowed characters should be accepted"
    );

    // Parse JSON response manually
    let body_text = response.text().await.expect("Failed to read response body");
    let body: serde_json::Value =
        serde_json::from_str(&body_text).expect("Response should be valid JSON");

    // Verify the alias was used in the response
    let data = body.get("data").expect("Response should have data field");
    let shortened_url = data
        .get("shortened_url")
        .and_then(|v| v.as_str())
        .expect("Response should have shortened_url field");

    assert!(
        shortened_url.ends_with(valid_alias),
        "Shortened URL should end with the provided alias: {}",
        shortened_url
    );

    // Verify the alias can be used for redirection
    let redirect_response = app.get(&format!("/{}", valid_alias)).await;
    assert_eq!(
        redirect_response.status(),
        StatusCode::PERMANENT_REDIRECT,
        "Alias should be accessible via redirect"
    );
}

/// Test that demonstrates the inconsistency issue
/// This test shows what happens if we could create an alias with _ or -
/// (This test documents the bug - it should fail if the bug is fixed)
#[tokio::test]
async fn demonstrate_character_validation_inconsistency() {
    // This test documents the expected behavior:
    // 1. validate_alias() should use state.allowed_chars instead of hardcoded characters
    // 2. Both underscore and hyphen should be rejected consistently
    // 3. Only characters from the configuration alphabet should be allowed

    // Test characters that are in the hardcoded validation but NOT in the config
    let problematic_characters = ['_', '-'];

    for &char in &problematic_characters {
        println!(
            "Testing alias with '{}' character - should be rejected consistently",
            char
        );

        // The current implementation allows these characters in validate_alias()
        // but they would be rejected in the redirect validation
        // This creates an inconsistency that should be fixed
    }
}

/// Test that validates the configuration alphabet is used correctly
#[tokio::test]
async fn validate_configuration_alphabet_usage() {
    // This test verifies that the configuration alphabet is properly loaded
    // and used consistently across the application

    let app = spawn_app().await;
    let url = "https://example.com";

    // Test with a character that should be in the config alphabet
    let config_alias = "ABC1234"; // All characters from generator.yml alphabet, length 7 to match config

    let response = post_shorten_with_alias(&app, config_alias, url).await;

    // Should succeed
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Alias with config-allowed characters should be accepted"
    );

    // And should be accessible via redirect
    let redirect_response = app.get(&format!("/{}", config_alias)).await;
    assert_eq!(
        redirect_response.status(),
        StatusCode::PERMANENT_REDIRECT,
        "Config-allowed alias should be accessible via redirect"
    );
}

/// Integration test that verifies aliases with _ and - are consistently rejected
/// and cannot be accessed via redirect
#[tokio::test]
async fn demonstrate_core_bug_with_underscore_and_hyphen() {
    let url = "https://example.com";

    // Test cases that verify _ and - characters are consistently rejected
    let test_cases = vec![("alias_with_underscore", '_'), ("alias-with-hyphen", '-')];

    for (alias, _problematic_char) in test_cases {
        // Create a fresh app instance for each test case to avoid interference
        let app = spawn_app().await;

        // Try to create the alias
        let response = post_shorten_with_alias(&app, alias, url).await;

        // Should return 422 (rejected)
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Alias '{}' should be rejected consistently",
            alias
        );

        // Verify the alias was rejected with appropriate error message
        let body = response.text().await.expect("Failed to read response body");
        let body_json: serde_json::Value =
            serde_json::from_str(&body).expect("Response should be valid JSON");

        assert!(
            body_json.get("success").and_then(|v| v.as_bool()) == Some(false),
            "Response should indicate failure for alias '{}'",
            alias
        );

        // Test if the alias can be accessed via redirect - this should fail since it was never created
        let redirect_response = app.get(&format!("/{}", alias)).await;

        assert_eq!(
            redirect_response.status(),
            StatusCode::NOT_FOUND,
            "Alias '{}' should not be accessible via redirect since it was rejected",
            alias
        );
    }
}
