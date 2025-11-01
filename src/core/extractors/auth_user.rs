use crate::features::auth::AuthService;
use crate::{
    ApiError, core::security::jwt::JwtKeys, features::users::repositories::UserRepository,
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::TypedHeader;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::{Authorization, Cookie};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub token_version: u32,
}

#[derive(Clone)]
pub struct AuthCtx {
    pub users: Arc<dyn UserRepository>,
    pub jwt: JwtKeys,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<AuthService>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // get AuthService from state

        // try Authorization cookie or bearer token
        let token = extract_token(parts, state).await;

        let auth_svc: Arc<AuthService> = Arc::from_ref(state);

        if let Some(token) = token {
            let claims = auth_svc
                .verify_token(&token)
                .await
                .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

            return Ok(AuthenticatedUser {
                user_id: claims.sub,
                token_version: claims.ver,
            });
        }

        Err(ApiError::Unauthorized("missing token".into()))
    }
}

async fn extract_token<S>(parts: &mut Parts, state: &S) -> Option<String>
where
    S: Send + Sync,
{
    // Try cookie first
    if let Ok(TypedHeader(cookies)) = TypedHeader::<Cookie>::from_request_parts(parts, state).await
        && let Some(token) = cookies.get("access_token")
    {
        return Some(token.to_owned());
    }

    // Fall back to Authorization header
    TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
        .await
        .ok()
        .map(|TypedHeader(Authorization(bearer))| bearer.token().to_owned())
}
