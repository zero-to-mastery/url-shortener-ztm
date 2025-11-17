use super::{dto::*, services::AuthService};
use crate::{
    ApiError, ApiResponse, AppState, ClientMeta,
    core::extractors::auth_user::AuthenticatedUser,
    features::{auth::repositories::AuthenticationAction, users::UserService},
};
use axum::{
    Extension, Json,
    extract::{FromRef, State},
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    extract::{
        CookieJar,
        cookie::{Cookie, SameSite},
    },
    headers::{Authorization, Cookie as AxCookie, authorization::Bearer},
};

use std::sync::Arc;

#[derive(Clone)]
pub struct AuthController {
    pub auth_svc: Arc<AuthService>,
    pub user_svc: Arc<UserService>,
}

impl FromRef<AppState> for AuthController {
    fn from_ref(app: &AppState) -> Self {
        Self {
            auth_svc: app.auth_service.clone(),
            user_svc: app.user_service.clone(),
        }
    }
}

pub async fn sign_up(
    State(ctrl): State<AuthController>,
    Extension(meta): Extension<ClientMeta>,
    jar: CookieJar,
    Json(req): Json<SignUpReq>,
) -> Result<impl IntoResponse, ApiError> {
    let bundle = ctrl
        .auth_svc
        .sign_up(req, meta.ip)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;

    let at = make_access_cookie(bundle.access_token, 30);
    let rt = make_refresh_cookie(bundle.refresh_token, 30);
    let jar = jar.add(at).add(rt);

    Ok((jar, Json(ApiResponse::success(()))))
}

pub async fn sign_in(
    State(ctrl): State<AuthController>,
    Extension(meta): Extension<ClientMeta>,
    jar: CookieJar,
    Json(req): Json<SignInReq>,
) -> Result<impl IntoResponse, ApiError> {
    let bundle = ctrl
        .auth_svc
        .sign_in(req, meta.ip)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".into()))?;

    let at = make_access_cookie(bundle.access_token, 30);
    let rt = make_refresh_cookie(bundle.refresh_token, 30);
    let jar = jar.add(at).add(rt);

    Ok((jar, Json(ApiResponse::success(()))))
}

pub async fn refresh(
    State(ctrl): State<AuthController>,
    TypedHeader(cookies): TypedHeader<AxCookie>,
    jar: CookieJar,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    Json(req): Json<RefreshReq>,
) -> Result<impl IntoResponse, ApiError> {
    let rt = cookies
        .get("refresh_token")
        .map(str::to_owned)
        .or_else(|| {
            auth.as_ref()
                .map(|TypedHeader(Authorization(b))| b.token().to_owned())
        })
        .ok_or_else(|| ApiError::Unauthorized("missing refresh_token".into()))?;

    let bundle = ctrl
        .auth_svc
        .refresh(&rt, &req.device_id)
        .await
        .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

    let at = make_access_cookie(bundle.access_token, 30);
    let rt = make_refresh_cookie(bundle.refresh_token, 30);
    let jar = jar.add(at).add(rt);

    Ok((jar, Json(ApiResponse::success(()))))
}

pub async fn sign_out(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
    Json(req): Json<RefreshReq>,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .sign_out(user.user_id, &req.device_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::success(()))
}

pub async fn sign_out_all(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .sign_out_all(user.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(ApiResponse::success(()))
}

pub async fn change_password(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
    Json(req): Json<ChangePasswordReq>,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .change_password(user.user_id, &req.old_password, &req.new_password)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;
    Ok(ApiResponse::success(()))
}

pub async fn email_verification_request(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
) -> Result<ApiResponse<()>, ApiError> {
    let usr = ctrl
        .user_svc
        .me(user.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    ctrl.auth_svc
        .send_verification_code(
            user.user_id,
            &usr.email,
            None,
            AuthenticationAction::VerifyEmail,
            None,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn email_verification_confirm(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
    Json(req): Json<EmailVerificationConfirmReq>,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .verify_code(user.user_id, AuthenticationAction::VerifyEmail, &req.code)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;

    ctrl.user_svc
        .confirm_email(user.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn change_email_request(
    State(ctrl): State<AuthController>,
    Extension(meta): Extension<ClientMeta>,
    user: AuthenticatedUser,
    Json(req): Json<ChangeEmailRequestReq>,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .request_email_change(user.user_id, &req.new_email, &req.current_password, meta)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn change_email_confirm(
    State(ctrl): State<AuthController>,
    user: AuthenticatedUser,
    Json(req): Json<ChangeEmailConfirmReq>,
) -> Result<ApiResponse<()>, ApiError> {
    ctrl.auth_svc
        .confirm_email_change(user.user_id, &req.code)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn pw_reset_request(
    State(ctrl): State<AuthController>,
    Json(req): Json<PwResetRequestReq>,
) -> Result<ApiResponse<()>, ApiError> {
    let usr = ctrl
        .user_svc
        .get_user_by_email(&req.email)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    ctrl.auth_svc
        .send_verification_code(
            usr.id,
            &usr.email,
            None,
            AuthenticationAction::ResetPassword,
            None,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

pub async fn pw_reset_confirm(
    State(ctrl): State<AuthController>,
    Json(req): Json<PwResetConfirmReq>,
) -> Result<ApiResponse<()>, ApiError> {
    let usr = ctrl
        .user_svc
        .get_user_by_email(&req.email)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    ctrl.auth_svc
        .verify_code(usr.id, AuthenticationAction::ResetPassword, &req.code)
        .await
        .map_err(|e| ApiError::Unprocessable(e.to_string()))?;

    ctrl.auth_svc
        .reset_password(usr.id, &req.new_password)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(()))
}

fn is_production() -> bool {
    std::env::var("APP_ENV")
        .map(|v| v == "production")
        .unwrap_or(false)
}

fn make_access_cookie(token: String, max_age_minutes: i64) -> Cookie<'static> {
    Cookie::build(("access_token", token))
        .http_only(true)
        .secure(is_production())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::minutes(max_age_minutes))
        .build()
}

fn make_refresh_cookie(token: String, max_age_days: i64) -> Cookie<'static> {
    Cookie::build(("refresh_token", token))
        .http_only(true)
        .secure(is_production())
        .same_site(SameSite::Strict)
        .path("/api/v1/auth/refresh")
        .max_age(time::Duration::days(max_age_days))
        .build()
}
