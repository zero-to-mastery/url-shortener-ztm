// features/users/repositories.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<Vec<u8>>,
    pub display_name: Option<String>,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub jwt_token_version: u32,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        email: &str,
        password_hash: &[u8],
        display: Option<String>,
    ) -> anyhow::Result<User>;
    async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>>;
    async fn find_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>>;
    async fn email_exists(&self, email: &str) -> anyhow::Result<bool>;
    async fn get_password_hash_by_id(&self, id: Uuid) -> anyhow::Result<Vec<u8>>;

    async fn confirm_email(&self, id: Uuid) -> anyhow::Result<()>;

    async fn set_last_login(&self, id: Uuid, at: DateTime<Utc>) -> anyhow::Result<()>;

    async fn bump_jwt_version(&self, id: Uuid) -> anyhow::Result<()>;

    async fn update_password(&self, id: Uuid, new_hash: &[u8]) -> anyhow::Result<()>;
    async fn update_email(&self, id: Uuid, new_email: &str) -> anyhow::Result<()>;
}

// A no-operation implementation of UserRepository for testing purposes.
#[derive(Clone, Debug)]
pub struct NoopUserRepo;

#[async_trait]
impl UserRepository for NoopUserRepo {
    async fn create(&self, _e: &str, _p: &[u8], _d: Option<String>) -> anyhow::Result<User> {
        anyhow::bail!("NoopUserRepo: create not supported in sqlite tests")
    }
    async fn confirm_email(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
    async fn find_user_by_email(&self, _email: &str) -> anyhow::Result<Option<User>> {
        Ok(None)
    }
    async fn find_user_by_id(&self, _id: Uuid) -> anyhow::Result<Option<User>> {
        Ok(None)
    }
    async fn email_exists(&self, _email: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn set_last_login(&self, _id: Uuid, _at: DateTime<Utc>) -> anyhow::Result<()> {
        Ok(())
    }
    async fn bump_jwt_version(&self, _id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update_password(&self, _id: Uuid, _new: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
    async fn update_email(&self, _id: Uuid, _new_email: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn get_password_hash_by_id(&self, _id: Uuid) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("NoopUserRepo: get_password_hash_by_id not supported")
    }
}
