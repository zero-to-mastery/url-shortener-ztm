//! RealWorld API minimal scaffold
//!
//! Implements a minimal subset of the RealWorld spec endpoints to integrate
//! alongside the existing URL shortener API. These handlers return the exact
//! response shapes required by the RealWorld spec (no custom envelope).
//!
//! Endpoints:
//! - `GET /api/tags`
//! - `POST /api/users` (register)
//! - `POST /api/users/login`
//! - `GET /api/user` (current user)

use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct TagsResponse {
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    user: RegisterUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RegisterUser {
    email: String,
    password: String,
    username: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    user: LoginUser,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct UserEnvelope {
    user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    email: String,
    token: String,
    username: String,
    bio: Option<String>,
    image: Option<String>,
}

/// GET /api/tags
pub async fn get_tags(_state: State<AppState>) -> impl IntoResponse {
    let resp = TagsResponse {
        tags: vec![
            "rust".to_string(),
            "axum".to_string(),
            "realworld".to_string(),
        ],
    };
    (StatusCode::OK, Json(resp))
}

/// POST /api/users (register)
pub async fn post_users_register(
    _state: State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Stubbed token and echo back provided fields to satisfy RealWorld shape
    let user = UserEnvelope {
        user: UserResponse {
            email: payload.user.email,
            username: payload.user.username,
            token: "stub.jwt.token".to_string(),
            bio: None,
            image: None,
        },
    };
    (StatusCode::CREATED, Json(user))
}

/// POST /api/users/login
pub async fn post_users_login(
    _state: State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Stubbed user; in future, validate credentials and issue JWT
    let username_from_email = payload
        .user
        .email
        .split('@')
        .next()
        .unwrap_or("user")
        .to_string();

    let user = UserEnvelope {
        user: UserResponse {
            email: payload.user.email,
            username: username_from_email,
            token: "stub.jwt.token".to_string(),
            bio: None,
            image: None,
        },
    };
    (StatusCode::OK, Json(user))
}

/// GET /api/user
pub async fn get_current_user(_state: State<AppState>) -> impl IntoResponse {
    // Stub current user until auth is implemented
    let user = UserEnvelope {
        user: UserResponse {
            email: "demo@example.com".to_string(),
            username: "demo".to_string(),
            token: "stub.jwt.token".to_string(),
            bio: None,
            image: None,
        },
    };
    (StatusCode::OK, Json(user))
}
