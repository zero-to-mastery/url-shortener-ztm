//! # Admin Interface Handler
//!
//! This module provides the admin interface handler for the URL shortener service.
//! It renders the web-based admin panel using Tera templates.

use crate::errors::ApiError;
use crate::state::AppState;
use crate::templates::build_templates;
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;
use tera::Context;

/// Admin interface handler that renders the home page template.
///
/// This handler serves the main admin interface for the URL shortener service.
/// It renders an HTML page using Tera templates with context data for the
/// web interface.
///
/// # Endpoint
///
/// `GET /admin`
///
/// # Arguments
///
/// * `State(state)` - Application state containing template directory and other dependencies
///
/// # Returns
///
/// Returns `Ok(Html<String>)` with the rendered HTML page, or `Err(ApiError)`
/// if template rendering fails.
///
/// # Template Context
///
/// The handler provides the following context variables to the template:
/// - `title` - Page title ("URL Shortener")
/// - `page` - Current page identifier ("Home")
/// - `message` - Welcome message ("Hello, world!")
///
/// # Template Files
///
/// This handler expects the following template files to exist:
/// - `templates/base.html` - Base template with common layout
/// - `templates/index.html` - Main page template
///
/// # Status Codes
///
/// - `200 OK` - Page rendered successfully
/// - `500 Internal Server Error` - Template rendering failed
///
/// # Examples
///
/// ```bash
/// # Access the admin interface
/// curl http://localhost:8000/admin
///
/// # Expected response: HTML page with admin interface
/// ```
///
/// # Error Handling
///
/// This handler will return an error if:
/// - Template files cannot be loaded
/// - Template rendering fails
/// - Context data is invalid
#[debug_handler]
pub async fn get_index(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "URL Shortener");
    context.insert("page", "Home");
    context.insert("message", "Hello, world!");

    let body = build_templates(state)?.render("index.html", &context)?;

    Ok(Html(body))
}
