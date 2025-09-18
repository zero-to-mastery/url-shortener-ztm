// src/lib/routes/shorten.rs

// endpoint handler which takes a URL to shorten, shortens it, and inserts it into the database

// dependencies
use crate::database::DatabaseError;
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{TypedHeader, headers::Host};
use axum_macros::debug_handler;
use tracing::instrument;
use url::Url;

// shorten endpoint handler
#[debug_handler]
#[instrument(name = "shorten" skip(state))]
pub async fn post_shorten(
    State(state): State<AppState>,
    TypedHeader(header): TypedHeader<Host>,
    url: String,
) -> Result<impl IntoResponse, ApiError> {
    let id = &nanoid::nanoid!(6);
    let p_url = Url::parse(&url).map_err(|e| {
        tracing::error!("Unable to parse URL");
        ApiError::Unprocessable(e.to_string())
    })?;
    let host = header.hostname();

    match state.database.insert_url(id, p_url.as_str()).await {
        Ok(()) => {
            let response_body = format!("https://{}/{}\n", host, id);
            tracing::info!("URL shortened and saved successfully...");
            Ok((StatusCode::OK, response_body).into_response())
        }
        Err(DatabaseError::Duplicate) => {
            tracing::error!("Duplicate ID generated");
            Err(ApiError::Internal("ID collision occurred".to_string()))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(ApiError::Internal(e.to_string()))
        }
    }
}
