// src/lib/routes/admin.rs

// Admin panel routes for user management

// dependencies
use crate::errors::ApiError;
use crate::state::AppState;
use crate::templates::build_templates;
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;
use tera::Context;

// handler for the admin dashboard
#[debug_handler]
pub async fn get_admin_dashboard(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Admin Dashboard");
    context.insert("page", "Admin");
    context.insert("message", "Welcome to the admin panel");

    let body = build_templates(state)?.render("admin.html", &context)?;

    Ok(Html(body))
}

// handler for user profile management
#[debug_handler]
pub async fn get_user_profile(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "User Profile");
    context.insert("page", "Profile");
    context.insert("message", "Manage your shortened URLs");

    let body = build_templates(state)?.render("profile.html", &context)?;

    Ok(Html(body))
}

// handler for user login
#[debug_handler]
pub async fn get_login(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Login");
    context.insert("page", "Login");
    context.insert("message", "Please log in to continue");

    let body = build_templates(state)?.render("login.html", &context)?;

    Ok(Html(body))
}

// handler for user registration
#[debug_handler]
pub async fn get_register(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "Register");
    context.insert("page", "Register");
    context.insert("message", "Create a new account");

    let body = build_templates(state)?.render("register.html", &context)?;

    Ok(Html(body))
}
