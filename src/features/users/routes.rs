use crate::AppState;

use super::controllers as c;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(c::me))
}
