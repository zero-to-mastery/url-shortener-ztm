// src/lib/routes/redirect.rs

// endpoint handler which provides the shortened URL to redirect to

// dependencies
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;
use sqlx::Error;
use tracing::instrument;

// redirect endpoint handler
#[debug_handler]
#[instrument(name = "redirect" skip(state))]
pub async fn get_redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let url: (String,) = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => {
                tracing::error!("shortened URL not found in the database...");
                ApiError::NotFound(e.to_string())
            }
            _ => ApiError::Internal(e.to_string()),
        })?;
    tracing::info!("shortened URL retreived, redirecting...");
    Ok(Redirect::permanent(&url.0))
}
