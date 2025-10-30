use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RefreshDevice {
    pub id: i32,
    pub user_id: Uuid,
    pub device_id: String,
    pub current_hash: Vec<u8>,
    pub previous_hash: Option<Vec<u8>>,
    pub absolute_expires: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailVerification {
    pub id: i32,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PasswordResetCode {
    pub id: i32,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    // // === Refresh Token Devices ===
    // async fn insert_refresh_device(&self, record: RefreshDevice) -> anyhow::Result<()>;
    // async fn find_refresh_device(
    //     &self,
    //     user_id: Uuid,
    //     device_id: &str,
    // ) -> anyhow::Result<Option<RefreshDevice>>;
    // async fn update_refresh_hash(
    //     &self,
    //     user_id: Uuid,
    //     device_id: &str,
    //     new_hash: Vec<u8>,
    //     old_hash: Option<Vec<u8>>,
    // ) -> anyhow::Result<()>;
    // async fn revoke_refresh(&self, user_id: Uuid, device_id: &str) -> anyhow::Result<()>;
    // async fn revoke_all_refresh(&self, user_id: Uuid) -> anyhow::Result<()>;

    // // === Email Verification ===
    // async fn insert_email_code(&self, code: EmailVerification) -> anyhow::Result<()>;
    // async fn find_email_code(
    //     &self,
    //     user_id: Uuid,
    //     code: &str,
    // ) -> anyhow::Result<Option<EmailVerification>>;
    // async fn mark_email_code_used(&self, id: i32) -> anyhow::Result<()>;

    // // === Password Reset ===
    // async fn insert_pw_reset_code(&self, code: PasswordResetCode) -> anyhow::Result<()>;
    // async fn find_pw_reset_code(
    //     &self,
    //     user_id: Uuid,
    //     code: &str,
    // ) -> anyhow::Result<Option<PasswordResetCode>>;
    // async fn mark_pw_reset_used(&self, id: i32) -> anyhow::Result<()>;

    // // === Login attempts / audit ===
    // async fn record_login_attempt(
    //     &self,
    //     user_id: Option<Uuid>,
    //     ip: Option<String>,
    //     success: bool,
    // ) -> anyhow::Result<()>;
}
