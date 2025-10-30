// features/auth/dto.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignUpReq {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
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

#[derive(Deserialize)]
pub struct VerifyEmailReq {
    pub code: String,
}

#[derive(Deserialize)]
pub struct RefreshReq {
    pub device_id: Option<String>,
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
