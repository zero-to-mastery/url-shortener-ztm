// tests/api/admin.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn admin_dashboard_requires_api_key() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request without API key
    let response = app.get_admin_dashboard().await;

    // Assert
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn admin_dashboard_works_with_valid_api_key() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request with valid API key
    let response = app.get_admin_dashboard_with_api_key().await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    assert!(response.text().await.unwrap().contains("Admin Dashboard"));
}

#[tokio::test]
async fn admin_login_page_requires_api_key() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request without API key
    let response = app.get_admin_login().await;

    // Assert
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn admin_register_page_requires_api_key() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request without API key
    let response = app.get_admin_register().await;

    // Assert
    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn admin_profile_requires_api_key() {
    // Arrange
    let app = spawn_app().await;

    // Act - Request without API key
    let response = app.get_admin_profile().await;

    // Assert
    assert_eq!(401, response.status().as_u16());
}