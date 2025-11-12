use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignUpReq {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Deserialize)]
pub struct SignInReq {
    pub email: String,
    pub password: String,
    pub device_id: Option<String>,
}

#[derive(Serialize)]
pub struct TokenResp {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct AuthBundle {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailReq {
    pub code: String,
}

#[derive(Deserialize)]
pub struct RefreshReq {
    pub device_id: String,
}

#[derive(Deserialize)]
pub struct PwResetRequestReq {
    pub email: String,
}

#[derive(Deserialize)]
pub struct PwResetConfirmReq {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct EmailVerificationConfirmReq {
    pub code: String,
}
