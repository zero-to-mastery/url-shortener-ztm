// src/lib/routes/index.rs

// route to serve the home page template

// dependencies
use crate::errors::ApiError;
use crate::state::AppState;
use crate::templates::build_templates;
use axum::{extract::State, response::Html};
use axum_macros::debug_handler;
use tera::Context;

// handler which renders the index page template
#[debug_handler]
pub async fn get_index(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let mut context = Context::new();
    context.insert("title", "URL Shortener");
    context.insert("page", "Home");
    context.insert("message", "Hello, world!");

    let body = build_templates(state)?.render("index.html", &context)?;

    Ok(Html(body))
}
