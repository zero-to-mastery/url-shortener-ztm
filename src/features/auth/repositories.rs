use std::net::IpAddr;

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

#[async_trait]
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
}
