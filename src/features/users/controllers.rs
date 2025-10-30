use crate::core::extractors::auth_user::AuthenticatedUser;
use crate::features::users::dto::MeResp;
use crate::features::users::services::UserService;
use crate::{ApiError, ApiResponse};
use axum::extract::State;
use axum::response::IntoResponse;

#[derive(Clone)]
pub struct UserController {
    pub svc: std::sync::Arc<UserService>,
}

impl UserController {
    pub fn new(svc: std::sync::Arc<UserService>) -> Self {
        Self { svc }
    }
}

pub async fn me(
    State(ctrl): State<UserController>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let u = ctrl
        .svc
        .me(user.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let response = MeResp {
        id: u.id,
        email: u.email,
        display_name: u.display_name,
        is_email_verified: u.is_email_verified,
        created_at: u.created_at,
        last_login_at: u.last_login_at,
    };

    Ok(ApiResponse::success(response))
}
