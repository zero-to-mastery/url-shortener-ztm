// tests/api/rate_limiting.rs

// tests for rate limiting functionality

use axum::http::StatusCode;
use url_shortener_ztm_lib::get_configuration;

use crate::helpers::spawn_app;

#[tokio::test]
async fn rate_limiting_blocks_excess_requests() {
    // Arrange
    let app = spawn_app().await;
    let test_url = "https://www.example.com";

    // Act - Make requests up to the burst limit (2 requests in test config)
    let mut responses = Vec::new();
    
    for i in 0..2 {
        let response = app
            .client
            .post(&app.url("/api/public/shorten"))
            .header("content-type", "text/plain")
            .body(format!("{}-{}", test_url, i))
            .send()
            .await
            .expect("Failed to execute request.");
        
        responses.push((i, response.status()));
    }

    // Verify first 2 requests succeed
    for (i, status) in &responses {
        assert_eq!(*status, StatusCode::OK, "Request {} should succeed", i);
    }

    // Make the 3rd request (should be rate limited)
    let response = app
        .client
        .post(&app.url("/api/public/shorten"))
        .header("content-type", "text/plain")
        .body(format!("{}-rate-limited", test_url))
        .send()
        .await
        .expect("Failed to execute request.");

        // Assert - The 3rd request should be rate limited even with valid API key
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // Check for rate limiting headers
    let headers = response.headers();
    assert!(headers.contains_key("retry-after"), "Should include retry-after header");
    assert!(headers.contains_key("x-ratelimit-after"), "Should include x-ratelimit-after header");
}

#[tokio::test]
async fn rate_limiting_provides_retry_after_headers() {
    // Arrange
    let app = spawn_app().await;
    let test_url = "https://www.example.com";

    // Act - Make requests up to the burst limit (2 requests)
    for i in 0..2 {
        let response = app
            .client
            .post(&app.url("/api/public/shorten"))
            .header("content-type", "text/plain")
            .body(format!("{}-{}", test_url, i))
            .send()
            .await
            .expect("Failed to execute request.");
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    // The 3rd request should be rate limited
    let response = app
        .client
        .post(&app.url("/api/public/shorten"))
        .header("content-type", "text/plain")
        .body(format!("{}-rate-limited", test_url))
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // Verify that rate limiting headers are present and reasonable
    let headers = response.headers();
    assert!(headers.contains_key("retry-after"), "Should include retry-after header");
    assert!(headers.contains_key("x-ratelimit-after"), "Should include x-ratelimit-after header");
    
    let retry_after = headers
        .get("retry-after")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .expect("retry-after should be a valid number");
    
    // The retry after should be reasonable (less than 2 minutes for this test config)
    assert!(retry_after > 0.0, "retry-after should be positive");
    assert!(retry_after < 120.0, "retry-after should be less than 2 minutes for test config");
}

#[tokio::test]
async fn rate_limiting_works_per_ip_address() {
    // This test would ideally test different IP addresses, but since we're running
    // locally, we'll just verify that rate limiting is applied consistently
    let app = spawn_app().await;
    let test_url = "https://www.example.com";

    // Multiple clients from the same IP should share the rate limit
    let client1 = &app.client;
    let client2 = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build second client");

    // Use up the rate limit with client1
    for i in 0..2 {
        let response = client1
            .post(&app.url("/api/public/shorten"))
            .header("content-type", "text/plain")
            .body(format!("{}-{}", test_url, i))
            .send()
            .await
            .expect("Failed to execute request.");
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    // client2 should also be rate limited (same IP)
    let response = client2
        .post(&app.url("/api/public/shorten"))
        .header("content-type", "text/plain")
        .body(test_url)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn health_check_is_not_rate_limited() {
    // Arrange
    let app = spawn_app().await;

    // Use up the rate limit with URL shortening requests
    for i in 0..2 {
        let response = app
            .client
            .post(&app.url("/api/public/shorten"))
            .header("content-type", "text/plain")
            .body(format!("https://www.example.com/{}", i))
            .send()
            .await
            .expect("Failed to execute request.");
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Verify the next URL shortening request is rate limited
    let response = app
        .client
        .post(&app.url("/api/public/shorten"))
        .header("content-type", "text/plain")
        .body("https://www.example.com")
        .send()
        .await
        .expect("Failed to execute request.");
    
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    // Health check should still work
    let response = app
        .client
        .get(&app.url("/api/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn secure_api_is_rate_limited() {
    // Arrange
    let app = spawn_app().await;
    let test_url = "https://www.example.com";

    // Use up the rate limit with secure API requests
    for i in 0..2 {
        let response = app
            .client
            .post(&app.url("/api/shorten"))
            .header("content-type", "text/plain")
            .header("x-api-key", app.api_key.to_string())
            .body(format!("{}-{}", test_url, i))
            .send()
            .await
            .expect("Failed to execute request.");
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    // The 3rd request should be rate limited even with valid API key
    let response = app
        .client
        .post(&app.url("/api/shorten"))
        .header("content-type", "text/plain")
        .header("x-api-key", app.api_key.to_string())
        .body(test_url)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limiting_configuration_is_loaded() {
    // Test that the configuration structure is loaded correctly
    let config = get_configuration().expect("Failed to read configuration");
    
    // The configuration should have rate limiting settings
    assert!(config.rate_limiting.enabled, "Rate limiting should be enabled");
    assert!(config.rate_limiting.requests_per_second > 0, "Rate should be positive");
    assert!(config.rate_limiting.burst_size > 0, "Burst size should be positive");
    
    // Test that the values are reasonable (either from base.yml or local.yml depending on environment)
    assert!(config.rate_limiting.requests_per_second >= 10, "Rate should be at least 10 req/sec");
    assert!(config.rate_limiting.burst_size >= 5, "Burst size should be at least 5");
}