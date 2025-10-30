use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct MeResp {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct ChangeEmailReq {
    pub new_email: String,
}

#[derive(Deserialize)]
pub struct ConfirmEmailChangeReq {
    pub code: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
}
