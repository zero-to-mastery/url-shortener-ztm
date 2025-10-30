use crate::{
    ApiError, core::security::jwt::JwtKeys, features::users::repositories::UserRepository,
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
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
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1) Try Bearer first
        tracing::debug!("headers: {:?}", parts.headers);

        let bearer = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                // Be tolerant to case and extra spaces
                s.strip_prefix("Bearer ")
                    .or_else(|| s.strip_prefix("bearer "))
                    .map(str::trim)
            })
            .map(|s| s.to_owned());

        // 2) Then try Cookie: access_token
        let cookie_token = parts
            .headers
            .get(header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|cookie| {
                cookie.split(';').find_map(|kv| {
                    let mut it = kv.trim().splitn(2, '=');
                    match (it.next(), it.next()) {
                        (Some("access_token"), Some(value))
                        | (Some("access_token "), Some(value)) => Some(value.to_string()),
                        _ => None,
                    }
                })
            });

        let token = bearer
            .or(cookie_token)
            .ok_or_else(|| ApiError::Unauthorized("missing token".into()))?;

        // 3) Get AuthCtx from extensions
        let auth = parts
            .extensions
            .get::<Arc<AuthCtx>>()
            .cloned()
            .ok_or_else(|| ApiError::Internal("AuthCtx missing in extensions".into()))?;

        // 4) Validate
        let claims = auth
            .jwt
            .verify(&token)
            .map_err(|_| ApiError::Unauthorized("invalid token".into()))?;

        let user = auth
            .users
            .find_user_by_id(claims.sub)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::Unauthorized("user not found".into()))?;

        if user.jwt_token_version != claims.ver {
            return Err(ApiError::Unauthorized("token revoked".into()));
        }

        Ok(AuthenticatedUser {
            user_id: user.id,
            token_version: user.jwt_token_version,
        })
    }
}
