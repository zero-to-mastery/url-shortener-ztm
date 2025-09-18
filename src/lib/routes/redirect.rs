// src/lib/routes/redirect.rs

// endpoint handler which provides the shortened URL to redirect to

// dependencies
use crate::database::DatabaseError;
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use axum_macros::debug_handler;
use tracing::instrument;

// redirect endpoint handler
#[debug_handler]
#[instrument(name = "redirect" skip(state))]
pub async fn get_redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    match state.database.get_url(&id).await {
        Ok(url) => {
            tracing::info!("shortened URL retrieved, redirecting...");
            Ok(Redirect::permanent(&url))
        }
        Err(DatabaseError::NotFound) => {
            tracing::error!("shortened URL not found in the database...");
            Err(ApiError::NotFound("URL not found".to_string()))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(ApiError::Internal(e.to_string()))
        }
    }
}
