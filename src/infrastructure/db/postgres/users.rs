use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
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
        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, display_name)
            VALUES (gen_random_uuid(), $1, $2, $3)
            RETURNING id, email, display_name, is_email_verified,
                     created_at, last_login_at, jwt_token_version
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(display)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: None,
            display_name: row.get("display_name"),
            is_email_verified: row.get("is_email_verified"),
            created_at: row.get("created_at"),
            last_login_at: row.get("last_login_at"),
            jwt_token_version: row.get::<i32, _>("jwt_token_version") as u32,
        })
    }

    async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, display_name, is_email_verified,
                created_at, last_login_at, jwt_token_version
            FROM users WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            password_hash: Some(r.get("password_hash")),
            display_name: r.get("display_name"),
            is_email_verified: r.get("is_email_verified"),
            created_at: r.get("created_at"),
            last_login_at: r.get("last_login_at"),
            jwt_token_version: r.get::<i32, _>("jwt_token_version") as u32,
        }))
    }

    async fn confirm_email(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET is_email_verified = TRUE WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn email_exists(&self, email: &str) -> anyhow::Result<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(email)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn find_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, display_name, is_email_verified,
                created_at, last_login_at, jwt_token_version
            FROM users WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            email: r.get("email"),
            password_hash: Some(r.get("password_hash")),
            display_name: r.get("display_name"),
            is_email_verified: r.get("is_email_verified"),
            created_at: r.get("created_at"),
            last_login_at: r.get("last_login_at"),
            jwt_token_version: r.get::<i32, _>("jwt_token_version") as u32,
        }))
    }

    async fn get_password_hash_by_id(&self, id: Uuid) -> anyhow::Result<Vec<u8>> {
        let row = sqlx::query("SELECT password_hash FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("password_hash"))
    }

    async fn set_last_login(&self, id: Uuid, at: DateTime<Utc>) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
            .bind(at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn bump_jwt_version(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET jwt_token_version = jwt_token_version + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_password(&self, id: Uuid, new_hash: &[u8]) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(new_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_email(&self, id: Uuid, new_email: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
            .bind(new_email)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
