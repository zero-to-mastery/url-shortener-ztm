use std::net::IpAddr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, types::ipnetwork::IpNetwork};
use uuid::Uuid;

use crate::features::auth::repositories::{AuthRepository, RefreshDevice};

#[derive(Clone)]
pub struct PgAuthRepository {
    pub pool: PgPool,
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
        let rec = sqlx::query!(
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
            user_id,
            device_id,
            current_hash,
            absolute_expires,
            user_agent,
            ip.map(IpNetwork::from)
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.id)
    }

    async fn get_refresh_device_by_rt(
        &self,
        device_id: &str,
        provided_hash: &[u8],
    ) -> anyhow::Result<Option<RefreshDevice>> {
        let r = sqlx::query!(
            r#"
            SELECT id, user_id, device_id, current_hash, previous_hash, absolute_expires,
                    revoked_at, user_agent, ip, last_rotated_at
            FROM refresh_token_devices
            WHERE device_id = $1 AND current_hash = $2
            "#,
            device_id,
            provided_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|x| RefreshDevice {
            id: x.id,
            user_id: x.user_id,
            device_id: x.device_id,
            current_hash: x.current_hash,
            previous_hash: x.previous_hash,
            absolute_expires: x.absolute_expires,
            revoked_at: x.revoked_at,
            user_agent: x.user_agent,
            ip: x.ip.map(|ipn| ipn.ip()),
            last_rotated_at: x.last_rotated_at,
        }))
    }

    async fn get_refresh_device_by_user_id(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> anyhow::Result<Option<RefreshDevice>> {
        let r = sqlx::query!(
            r#"
            SELECT id, user_id, device_id, current_hash, previous_hash, absolute_expires,
                    revoked_at, user_agent, ip, last_rotated_at
            FROM refresh_token_devices
            WHERE device_id = $1 AND user_id = $2
            "#,
            device_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|x| RefreshDevice {
            id: x.id,
            user_id: x.user_id,
            device_id: x.device_id,
            current_hash: x.current_hash,
            previous_hash: x.previous_hash,
            absolute_expires: x.absolute_expires,
            revoked_at: x.revoked_at,
            user_agent: x.user_agent,
            ip: x.ip.map(|ipn| ipn.ip()),
            last_rotated_at: x.last_rotated_at,
        }))
    }

    async fn rotate_refresh_hash(
        &self,
        id: i32,
        new_hash: &[u8],
        rotated_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_token_devices
            SET previous_hash = current_hash,
                current_hash = $1,
                last_rotated_at = $2
            WHERE id = $3
            "#,
            new_hash,
            rotated_at,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_previous_hash(&self, id: i32, prev: Option<&[u8]>) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE refresh_token_devices SET previous_hash = $1 WHERE id = $2",
            prev,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_device(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE refresh_token_devices SET revoked_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_all(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE refresh_token_devices SET revoked_at = NOW() WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
