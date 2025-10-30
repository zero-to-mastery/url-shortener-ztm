use std::sync::Arc;

use super::controllers as c;
use crate::core::extractors::auth_user::AuthCtx;
use axum::{Extension, Router, routing::get};

pub fn router(user_ctrl: c::UserController, auth_ctx: Arc<AuthCtx>) -> Router {
    Router::new()
        .route("/me", get(c::me))
        // Add other user routes here
        .layer(Extension(auth_ctx))
        .with_state(user_ctrl)
}
