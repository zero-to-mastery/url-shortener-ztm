use crate::core::extractors::auth_user::AuthenticatedUser;
use crate::features::users::dto::MeResp;
use crate::features::users::services::UserService;
use crate::{ApiError, ApiResponse, AppState};
use axum::extract::{FromRef, State};
use axum::response::IntoResponse;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserController {
    pub svc: Arc<UserService>,
}

impl FromRef<AppState> for UserController {
    fn from_ref(app: &AppState) -> Self {
        Self {
            svc: app.user_service.clone(),
        }
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
