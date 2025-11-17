use super::controllers as c;
use crate::AppState;

use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sign-up", post(c::sign_up))
        .route("/sign-in", post(c::sign_in))
        .route("/sign-out", post(c::sign_out))
        .route("/sign-out-all", post(c::sign_out_all))
        .route("/refresh", post(c::refresh))
        .route("/change-password", post(c::change_password))
        .route("/verify-email/request", get(c::email_verification_request))
        .route("/verify-email/confirm", post(c::email_verification_confirm))
        .route("/password-reset/request", post(c::pw_reset_request))
        .route("/password-reset/confirm", post(c::pw_reset_confirm))
        .route("/change-email/request", post(c::change_email_request))
        .route("/change-email/confirm", post(c::change_email_confirm))
}
