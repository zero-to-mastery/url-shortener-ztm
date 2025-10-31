use std::sync::Arc;

// features/auth/controllers.rs
use super::{dto::*, services::AuthService};
use crate::{ApiError, ApiResponse, AppState};
use axum::extract::FromRef;
use axum::{Json, extract::State, response::IntoResponse};

#[derive(Clone)]
pub struct AuthController {
    pub svc: Arc<AuthService>,
}

impl FromRef<AppState> for AuthController {
    fn from_ref(app: &AppState) -> Self {
        Self {
            svc: app.auth_service.clone(),
        }
    }
}

pub async fn sign_up(
    State(ctrl): State<AuthController>,
    Json(req): Json<SignUpReq>,
) -> Result<impl IntoResponse, ApiError> {
    let resp = ctrl
        .svc
        .sign_up(req)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;

    let cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        resp.access_token,
        1800 // 30 minutes
    );

    Ok((
        [(axum::http::header::SET_COOKIE, cookie)],
        axum::Json(ApiResponse::success(())),
    ))
}

pub async fn sign_in(
    State(ctrl): State<AuthController>,
    Json(req): Json<SignInReq>,
) -> Result<impl IntoResponse, ApiError> {
    let resp = ctrl
        .svc
        .sign_in(req)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".into()))?;

    let cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
        resp.access_token, 1800
    );

    Ok((
        [(axum::http::header::SET_COOKIE, cookie)],
        axum::Json(ApiResponse::success(())),
    ))
}

// pub async fn refresh(
//     State(ctrl): State<AuthController>,
//     Json(req): Json<RefreshReq>,
// ) -> Result<ApiResponse<TokenResp>, ApiError> {
//     let resp = ctrl
//         .svc
//         .refresh(req)
//         .await
//         .map_err(|e| ApiError::Unauthorized(e.to_string()))?;
//     Ok(ApiResponse::ok(resp))
// }

// pub async fn sign_out(
//     State(ctrl): State<AuthController>,
//     user: AuthenticatedUser,
//     Json(req): Json<RefreshReq>,
// ) -> Result<ApiResponse<()>, ApiError> {
//     ctrl.svc
//         .sign_out(user.user_id, req.device_id)
//         .await
//         .map_err(|e| ApiError::Internal(e.to_string()))?;
//     Ok(ApiResponse::ok(()))
// }

// pub async fn sign_out_all(
//     State(ctrl): State<AuthController>,
//     user: AuthenticatedUser,
// ) -> Result<ApiResponse<()>, ApiError> {
//     ctrl.svc
//         .sign_out_all(user.user_id)
//         .await
//         .map_err(|e| ApiError::Internal(e.to_string()))?;
//     Ok(ApiResponse::ok(()))
// }
