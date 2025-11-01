// src/lib/routes/admin.rs

// Admin panel routes for user management

// dependencies
use crate::errors::ApiError;
use crate::state::AppState;
use crate::templates::get_templates;
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;
use tera::Context;

// handler for the admin dashboard
#[debug_handler]
pub async fn get_admin_dashboard(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Dashboard");
    let body = get_templates().render("admin.html", &context)?;
    Ok(Html(body))
}

// handler for user profile management
#[debug_handler]
pub async fn get_user_profile(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "User Profile");
    let body = get_templates().render("profile.html", &context)?;
    Ok(Html(body))
}

// handler for user login
#[debug_handler]
pub async fn get_login(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Login");
    let body = get_templates().render("login.html", &context)?;
    Ok(Html(body))
}

// handler for user registration
#[debug_handler]
pub async fn get_register(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Register");
    let body = get_templates().render("register.html", &context)?;
    Ok(Html(body))
}

// handler for manage users
#[debug_handler]
pub async fn get_users(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Manage Users");
    let body = get_templates().render("users.html", &context)?;
    Ok(Html(body))
}

// handler for manage urls
#[debug_handler]
pub async fn get_urls(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Manage URLs");
    let body = get_templates().render("urls.html", &context)?;
    Ok(Html(body))
}

// handler for analytics
#[debug_handler]
pub async fn get_analytics(_state: State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new(); // <-- Make it mutable
    context.insert("title", "Analytics"); // <-- ADD THIS LINE
    let body = get_templates().render("analytics.html", &context)?;
    Ok(Html(body))
}
