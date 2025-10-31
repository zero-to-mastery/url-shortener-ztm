use super::controllers as c;
use crate::AppState;

use axum::{Router, routing::post};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sign-up", post(c::sign_up))
        .route("/sign-in", post(c::sign_in))
    // .route("/refresh", post(c::refresh))
    // .route("/verify-email", post(c::verify_email))
    // .route("/password-reset/request", post(c::pw_reset_request))
    // .route("/password-reset/confirm", post(c::pw_reset_confirm))
    // .route("/sign-out", post(c::sign_out))
    // .route("/sign-out-all", post(c::sign_out_all))
}
