use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::users::repositories::{User, UserRepository};

#[derive(Clone)]
pub struct PgUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(
        &self,
        email: &str,
        password_hash: &[u8],
        display: Option<String>,
    ) -> anyhow::Result<User> {
        let r = sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, display_name)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING id, email, display_name, is_email_verified,
                     created_at, last_login_at, jwt_token_version
            "#,
            email,
            password_hash,
            display
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: r.id,
            email: r.email,
            password_hash: None,
            display_name: r.display_name,
            is_email_verified: r.is_email_verified,
            created_at: r.created_at,
            last_login_at: r.last_login_at,
            jwt_token_version: r.jwt_token_version as u32,
        })
    }

    async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let r = sqlx::query!(
            r#"
            SELECT id, email, password_hash, display_name, is_email_verified,
                   created_at, last_login_at, jwt_token_version
            FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| User {
            id: r.id,
            email: r.email,
            password_hash: Some(r.password_hash),
            display_name: r.display_name,
            is_email_verified: r.is_email_verified,
            created_at: r.created_at,
            last_login_at: r.last_login_at,
            jwt_token_version: r.jwt_token_version as u32,
        }))
    }

    async fn email_exists(&self, email: &str) -> anyhow::Result<bool> {
        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)", email)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists.unwrap_or(false))
    }

    async fn find_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        let r = sqlx::query!(
            r#"
            SELECT id, email, password_hash, display_name, is_email_verified,
                   created_at, last_login_at, jwt_token_version
            FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| User {
            id: r.id,
            email: r.email,
            password_hash: Some(r.password_hash),
            display_name: r.display_name,
            is_email_verified: r.is_email_verified,
            created_at: r.created_at,
            last_login_at: r.last_login_at,
            jwt_token_version: r.jwt_token_version as u32,
        }))
    }

    async fn get_password_hash_by_id(&self, id: Uuid) -> anyhow::Result<Vec<u8>> {
        let r = sqlx::query!("SELECT password_hash FROM users WHERE id = $1", id)
            .fetch_one(&self.pool)
            .await?;
        Ok(r.password_hash)
    }

    async fn set_last_login(&self, id: Uuid, at: DateTime<Utc>) -> anyhow::Result<()> {
        sqlx::query!("UPDATE users SET last_login_at = $1 WHERE id = $2", at, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn bump_jwt_version(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE users SET jwt_token_version = jwt_token_version + 1 WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_password(&self, id: Uuid, new_hash: &[u8]) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            new_hash,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
