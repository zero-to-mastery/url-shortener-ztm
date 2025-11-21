use std::net::IpAddr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Type;
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
    pub ip: Option<IpAddr>,
    pub last_rotated_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "authentication_action")]
#[sqlx(rename_all = "snake_case")]
pub enum AuthenticationAction {
    VerifyEmail,
    ResetPassword,
    ChangeEmail,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthenticationChallenge {
    pub id: i64,
    pub user_id: Uuid,
    pub action: AuthenticationAction,
    pub target: Option<String>,
    pub code_hash: Vec<u8>,
    pub attempts: u8,
    pub meta: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthRepoError {
    #[error("cooldown: {0} seconds remaining")]
    Cooldown(i32),
    #[error("already active challenge")]
    AlreadyActive,
    #[error("email already taken")]
    EmailTaken,
    #[error("not found")]
    NotFound,
    #[error("transient error")]
    Transient,
    #[error("internal storage error")]
    Internal,
}

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn upsert_refresh_device(
        &self,
        user_id: Uuid,
        device_id: &str,
        current_hash: &[u8],
        absolute_expires: DateTime<Utc>,
        user_agent: Option<&str>,
        ip: Option<IpAddr>,
    ) -> anyhow::Result<i32>;

    async fn get_refresh_device_by_rt(
        &self,
        device_id: &str,
        rt_hash: &[u8],
    ) -> anyhow::Result<Option<RefreshDevice>>;

    async fn get_refresh_device_by_user_id(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> anyhow::Result<Option<RefreshDevice>>;

    async fn rotate_refresh_hash(
        &self,
        id: i32,
        new_hash: &[u8],
        rotated_at: DateTime<Utc>,
    ) -> anyhow::Result<()>;

    async fn set_previous_hash(&self, id: i32, prev: Option<&[u8]>) -> anyhow::Result<()>;

    async fn revoke_device(&self, id: i32) -> anyhow::Result<()>;
    async fn revoke_all(&self, user_id: Uuid) -> anyhow::Result<()>;

    async fn is_user_ip_blocked(
        &self,
        user_id: &Uuid,
        ip: IpAddr,
        threshold: i32,
        window_mins: i32,
        fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError>;

    async fn should_lock_user_for_failures(
        &self,
        user_id: &Uuid,
        threshold: i32,
        window_mins: i32,
        fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError>;

    async fn add_sign_in_attempt(
        &self,
        user_id: &Uuid,
        ip: IpAddr,
        target: &str,
        success: bool,
        user_agent: Option<&str>,
    ) -> Result<(), AuthRepoError>;

    #[allow(clippy::too_many_arguments)]
    async fn create_or_refresh_auth_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
        target: Option<&str>,
        code_hash: &[u8],
        meta: Option<&Value>,
        expires_at: DateTime<Utc>,
        cooldown_secs: Option<i32>,
    ) -> Result<(), AuthRepoError>;
    async fn get_auth_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
    ) -> Result<Option<AuthenticationChallenge>, AuthRepoError>;
    async fn increase_auth_challenge_attempts(
        &self,
        challenge_id: i64,
    ) -> Result<(), AuthRepoError>;
    async fn confirm_authentication_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
        confirmed_at: DateTime<Utc>,
    ) -> Result<(), AuthRepoError>;
}

// A no-operation implementation of AuthRepository for testing purposes.
#[derive(Clone, Debug)]
pub struct NoopAuthRepo;

#[async_trait]
impl AuthRepository for NoopAuthRepo {
    async fn upsert_refresh_device(
        &self,
        _user_id: Uuid,
        _device_id: &str,
        _current_hash: &[u8],
        _absolute_expires: DateTime<Utc>,
        _user_agent: Option<&str>,
        _ip: Option<IpAddr>,
    ) -> anyhow::Result<i32> {
        anyhow::bail!("NoopAuthRepo: sqlite tests don't support refresh devices")
    }

    async fn get_refresh_device_by_rt(
        &self,
        _device_id: &str,
        _rt_hash: &[u8],
    ) -> anyhow::Result<Option<RefreshDevice>> {
        Ok(None)
    }

    async fn get_refresh_device_by_user_id(
        &self,
        _device_id: &str,
        _user_id: Uuid,
    ) -> anyhow::Result<Option<RefreshDevice>> {
        Ok(None)
    }

    async fn rotate_refresh_hash(
        &self,
        _id: i32,
        _new_hash: &[u8],
        _rotated_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_previous_hash(&self, _id: i32, _prev: Option<&[u8]>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn revoke_device(&self, _id: i32) -> anyhow::Result<()> {
        Ok(())
    }

    async fn revoke_all(&self, _user_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_or_refresh_auth_challenge(
        &self,
        _user_id: Uuid,
        _action: AuthenticationAction,
        _target: Option<&str>,
        _code_hash: &[u8],
        _meta: Option<&Value>,
        _expires_at: DateTime<Utc>,
        _cooldown_secs: Option<i32>,
    ) -> Result<(), AuthRepoError> {
        Ok(())
    }
    async fn get_auth_challenge(
        &self,
        _user_id: Uuid,
        _action: AuthenticationAction,
    ) -> Result<Option<AuthenticationChallenge>, AuthRepoError> {
        Ok(None)
    }
    async fn increase_auth_challenge_attempts(
        &self,
        _challenge_id: i64,
    ) -> Result<(), AuthRepoError> {
        Ok(())
    }
    async fn confirm_authentication_challenge(
        &self,
        _user_id: Uuid,
        _action: AuthenticationAction,
        _confirmed_at: DateTime<Utc>,
    ) -> Result<(), AuthRepoError> {
        Ok(())
    }
    async fn is_user_ip_blocked(
        &self,
        _user_id: &Uuid,
        _ip: IpAddr,
        _ip_max: i32,
        _window_mins: i32,
        _fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError> {
        Ok(false)
    }

    async fn should_lock_user_for_failures(
        &self,
        _user_id: &Uuid,
        _threshold: i32,
        _window_mins: i32,
        _fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError> {
        Ok(false)
    }

    async fn add_sign_in_attempt(
        &self,
        _user_id: &Uuid,
        _ip: IpAddr,
        _target: &str,
        _success: bool,
        _user_agent: Option<&str>,
    ) -> Result<(), AuthRepoError> {
        Ok(())
    }
}
