//! # Middleware
//!
//! This module provides middleware functions for the URL shortener service.
//! Middleware functions are used to process requests before they reach the
//! main handler, enabling cross-cutting concerns like authentication.
//!
//! ## Available Middleware
//!
//! - [`check_api_key`] - Validates API key authentication for protected endpoints
//!
//! ## Usage
//!
//! Middleware is applied to routes using Axum's middleware system:
//!
//! ```rust,ignore
//! use axum::{Router, middleware::from_fn_with_state};
//! use url_shortener_ztm_lib::middleware::check_api_key;
//!
//! // Apply middleware to specific routes
//! let protected_routes = Router::new()
//!     .route("/api/shorten", post(shorten_handler))
//!     .route_layer(from_fn_with_state(state, check_api_key));
//! ```
use crate::response::ApiResponse;
use crate::state::AppState;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
use uuid::Uuid;

/// Middleware function that validates API key authentication.
///
/// This middleware checks for a valid API key in the `x-api-key` header of incoming requests.
/// If a valid API key is provided, the request is allowed to proceed to the handler.
/// If no key or an invalid key is provided, an unauthorized error response is returned.
///
/// # Authentication Process
///
/// 1. Extracts the `x-api-key` header from the request
/// 2. Parses the header value as a UUID
/// 3. Compares the provided key with the configured API key
/// 4. Allows the request to proceed if keys match, otherwise returns 401 Unauthorized
///
/// # Arguments
///
/// * `State(state)` - Application state containing the configured API key
/// * `request` - The incoming HTTP request
/// * `next` - The next middleware or handler in the chain
///
/// # Returns
///
/// Returns either:
/// - The response from the next handler (if authentication succeeds)
/// - An unauthorized error response (if authentication fails)
///
/// # Error Response
///
/// When authentication fails, returns a JSON error response:
///
/// ```json
/// {
///   "success": false,
///   "message": "Unauthorized",
///   "status": 401,
///   "time": "2025-01-18T12:00:00Z",
///   "data": null
/// }
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use axum::{Router, middleware::from_fn_with_state, routing::post};
/// use url_shortener_ztm_lib::middleware::check_api_key;
/// use url_shortener_ztm_lib::state::AppState;
///
/// // Apply to protected routes
/// let app = Router::new()
///     .route("/api/shorten", post(post_shorten))
///     .route_layer(from_fn_with_state(app_state, check_api_key));
/// ```
///
/// # Security Notes
///
/// - API keys should be kept secure and not logged
/// - Consider using HTTPS in production to prevent key interception
/// - The key comparison is done using constant-time comparison for security
pub async fn check_api_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let api_key: &Uuid = state.api_key.as_ref();

    let provided_api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s.trim()).ok());

    if provided_api_key.as_ref() == Some(api_key) {
        next.run(request).await
    } else {
        ApiResponse::<()>::error("Unauthorized", StatusCode::UNAUTHORIZED).into_response()
    }
}

// src/middleware/client_meta.rs

#[derive(Clone, Debug)]
pub struct ClientMeta {
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

pub async fn capture_client_meta(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request,
    next: Next,
) -> Response {
    let xff = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next().map(str::trim))
        .and_then(|s| IpAddr::from_str(s).ok());

    // 2) X-Real-IP
    let xri = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| IpAddr::from_str(s).ok());

    // 3) Socket IP
    let from_socket = Some(addr.ip());

    let ip = xff.or(xri).or(from_socket);

    let ua = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    req.extensions_mut()
        .insert(ClientMeta { ip, user_agent: ua });
    next.run(req).await
}
