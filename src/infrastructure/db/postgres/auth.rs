use std::net::IpAddr;

use crate::features::auth::repositories::{
    AuthRepoError, AuthRepository, AuthenticationAction, AuthenticationChallenge, RefreshDevice,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row, Type, types::ipnetwork::IpNetwork};
use uuid::Uuid;

#[derive(Clone)]
pub struct PgAuthRepository {
    pub pool: PgPool,
}

#[derive(Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "challenge_upsert_status")]
#[sqlx(rename_all = "snake_case")]
enum UpsertStatus {
    Inserted,
    Updated,
    Cooldown,
}

#[derive(sqlx::FromRow)]
struct UpsertRow {
    status: UpsertStatus,
    #[sqlx(rename = "challenge_id")]
    _challenge_id: i64,
    seconds_remaining: i32,
}

#[async_trait]
impl AuthRepository for PgAuthRepository {
    async fn upsert_refresh_device(
        &self,
        user_id: Uuid,
        device_id: &str,
        current_hash: &[u8],
        absolute_expires: DateTime<Utc>,
        user_agent: Option<&str>,
        ip: Option<IpAddr>,
    ) -> anyhow::Result<i32> {
        let row = sqlx::query(
            r#"
            INSERT INTO refresh_token_devices (user_id, device_id, current_hash, absolute_expires, user_agent, ip)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, device_id)
            DO UPDATE SET
                current_hash = EXCLUDED.current_hash,
                previous_hash = refresh_token_devices.current_hash,
                last_rotated_at = NOW(),
                absolute_expires = EXCLUDED.absolute_expires,
                user_agent = EXCLUDED.user_agent,
                ip = EXCLUDED.ip,
                revoked_at = NULL
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(current_hash)
        .bind(absolute_expires)
        .bind(user_agent)
        .bind(ip.map(IpNetwork::from))
        .fetch_one(&self.pool)
        .await?;
        let id = row.get::<i32, _>("id");
        Ok(id)
    }

    async fn get_refresh_device_by_rt(
        &self,
        device_id: &str,
        provided_hash: &[u8],
    ) -> anyhow::Result<Option<RefreshDevice>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, device_id, current_hash, previous_hash, absolute_expires,
                    revoked_at, user_agent, ip, last_rotated_at
            FROM refresh_token_devices
            WHERE device_id = $1 AND current_hash = $2
            "#,
        )
        .bind(device_id)
        .bind(provided_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RefreshDevice {
            id: r.get("id"),
            user_id: r.get("user_id"),
            device_id: r.get("device_id"),
            current_hash: r.get("current_hash"),
            previous_hash: r.get("previous_hash"),
            absolute_expires: r.get("absolute_expires"),
            revoked_at: r.get("revoked_at"),
            user_agent: r.get("user_agent"),
            ip: r.get::<Option<IpNetwork>, _>("ip").map(|ipn| ipn.ip()),
            last_rotated_at: r.get("last_rotated_at"),
        }))
    }

    async fn get_refresh_device_by_user_id(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> anyhow::Result<Option<RefreshDevice>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, device_id, current_hash, previous_hash, absolute_expires,
                    revoked_at, user_agent, ip, last_rotated_at
            FROM refresh_token_devices
            WHERE device_id = $1 AND user_id = $2
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| RefreshDevice {
            id: r.get("id"),
            user_id: r.get("user_id"),
            device_id: r.get("device_id"),
            current_hash: r.get("current_hash"),
            previous_hash: r.get("previous_hash"),
            absolute_expires: r.get("absolute_expires"),
            revoked_at: r.get("revoked_at"),
            user_agent: r.get("user_agent"),
            ip: r.get::<Option<IpNetwork>, _>("ip").map(|ipn| ipn.ip()),
            last_rotated_at: r.get("last_rotated_at"),
        }))
    }

    async fn rotate_refresh_hash(
        &self,
        id: i32,
        new_hash: &[u8],
        rotated_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE refresh_token_devices
            SET previous_hash = current_hash,
                current_hash = $1,
                last_rotated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(new_hash)
        .bind(rotated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_previous_hash(&self, id: i32, prev: Option<&[u8]>) -> anyhow::Result<()> {
        sqlx::query("UPDATE refresh_token_devices SET previous_hash = $1 WHERE id = $2")
            .bind(prev)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_device(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query("UPDATE refresh_token_devices SET revoked_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_all(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE refresh_token_devices SET revoked_at = NOW() WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_or_refresh_auth_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
        target: Option<&str>,
        code_hash: &[u8],
        meta: Option<&Value>,
        expires_at: DateTime<Utc>,
        cooldown_secs: Option<i32>,
    ) -> Result<(), AuthRepoError> {
        let row: UpsertRow = sqlx::query_as(
            r#"
            SELECT status, challenge_id, seconds_remaining
            FROM create_or_refresh_auth_challenge($1, $2, $3::citext, $4, $5::jsonb, $6, $7)"#,
        )
        .bind(user_id)
        .bind(action)
        .bind(target)
        .bind(code_hash)
        .bind(meta)
        .bind(expires_at)
        .bind(cooldown_secs.unwrap_or(60))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error upserting auth challenge: {:#?}", e);
            e
        })?;

        match row.status {
            UpsertStatus::Inserted | UpsertStatus::Updated => Ok(()),
            UpsertStatus::Cooldown => {
                tracing::debug!(
                    "Cooldown active for user {} , seconds remaining: {}",
                    user_id,
                    row.seconds_remaining
                );
                Err(AuthRepoError::Cooldown(row.seconds_remaining))
            }
        }
    }

    async fn get_auth_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
    ) -> Result<Option<AuthenticationChallenge>, AuthRepoError> {
        let row = sqlx::query(
            r#"
                SELECT * FROM authentication_challenges
                WHERE user_id = $1 AND action = $2 AND confirmed_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(action)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error upserting auth challenge: {:#?}", e);
            e
        })?;

        Ok(row.map(|row| AuthenticationChallenge {
            id: row.get("id"),
            user_id: row.get("user_id"),
            action: row.get("action"),
            target: row.get("target"),
            code_hash: row.get("code_hash"),
            meta: row.get("meta"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            confirmed_at: row.get("confirmed_at"),
            attempts: row.get::<i32, _>("attempts") as u8,
        }))
    }

    async fn confirm_authentication_challenge(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
        confirmed_at: DateTime<Utc>,
    ) -> Result<(), AuthRepoError> {
        sqlx::query("SELECT confirm_auth_challenge($1, $2, $3)")
            .bind(user_id)
            .bind(action)
            .bind(confirmed_at)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Error upserting auth challenge: {:#?}", e);
                e
            })?;

        Ok(())
    }

    async fn increase_auth_challenge_attempts(
        &self,
        challenge_id: i64,
    ) -> Result<(), AuthRepoError> {
        sqlx::query(
            r#"
            UPDATE authentication_challenges
            SET attempts = attempts + 1
            WHERE id = $1 AND confirmed_at IS NULL
            "#,
        )
        .bind(challenge_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error upserting auth challenge: {:#?}", e);
            e
        })?;

        Ok(())
    }

    async fn add_sign_in_attempt(
        &self,
        user_id: &Uuid,
        ip: IpAddr,
        target: &str,
        success: bool,
        user_agent: Option<&str>,
    ) -> Result<(), AuthRepoError> {
        sqlx::query(
            r#"
            INSERT INTO sign_in_attempts (user_id, ip, target, success, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(ip)
        .bind(target)
        .bind(success)
        .bind(user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error adding sign in attempt: {:#?}", e);
            e
        })?;

        Ok(())
    }

    async fn is_user_ip_blocked(
        &self,
        user_id: &Uuid,
        ip: IpAddr,
        threshold: i32,
        window_mins: i32,
        fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError> {
        let blocked = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM sign_in_attempts
                WHERE user_id = $1
                  AND ip = $2
                  AND success = false
                  AND created_at > GREATEST(
                        now() - make_interval(mins => $3),
                        COALESCE($4, '-infinity'::timestamptz)
                  )
                ORDER BY created_at DESC
                OFFSET GREATEST($5 - 1, 0)
                LIMIT 1
            )
            "#,
        )
        .bind(user_id)
        .bind(ip)
        .bind(window_mins)
        .bind(fail_count_since)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await?;

        Ok(blocked)
    }

    async fn should_lock_user_for_failures(
        &self,
        user_id: &Uuid,
        threshold: i32,
        window_mins: i32,
        fail_count_since: Option<DateTime<Utc>>,
    ) -> Result<bool, AuthRepoError> {
        let should_lock = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM sign_in_attempts
                WHERE user_id = $1
                  AND success = false
                  AND created_at > GREATEST(
                        now() - make_interval(mins => $2),
                        COALESCE($3, '-infinity'::timestamptz)
                  )
                ORDER BY created_at DESC
                OFFSET GREATEST($4 - 1, 0)
                LIMIT 1
            )
            "#,
        )
        .bind(user_id) // $1
        .bind(window_mins) // $2
        .bind(fail_count_since) // $3
        .bind(threshold) // $4
        .fetch_one(&self.pool)
        .await?;

        Ok(should_lock)
    }
}

impl From<sqlx::Error> for AuthRepoError {
    fn from(err: sqlx::Error) -> Self {
        use sqlx::Error::*;
        match &err {
            Database(db) if matches!(db.code().as_deref(), Some("40001" | "55P03" | "57014")) => {
                AuthRepoError::Transient
            }
            PoolTimedOut | Io(_) | Tls(_) | PoolClosed => AuthRepoError::Transient,

            RowNotFound => AuthRepoError::NotFound,

            _ => AuthRepoError::Internal,
        }
    }
}
