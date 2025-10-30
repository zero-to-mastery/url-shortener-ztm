use std::sync::Arc;

use super::controllers as c;
use crate::core::extractors::auth_user::AuthCtx;

use axum::{Extension, Router, routing::post};

pub fn router(auth_ctrl: c::AuthController, auth_ctx: Arc<AuthCtx>) -> Router {
    Router::new()
        .route("/sign-up", post(c::sign_up))
        .route("/sign-in", post(c::sign_in))
        // .route("/refresh", post(c::refresh))
        // .route("/verify-email", post(c::verify_email))
        // .route("/password-reset/request", post(c::pw_reset_request))
        // .route("/password-reset/confirm", post(c::pw_reset_confirm))
        // .route("/sign-out", post(c::sign_out))
        // .route("/sign-out-all", post(c::sign_out_all))
        .layer(Extension(auth_ctx))
        .with_state(auth_ctrl)
}
