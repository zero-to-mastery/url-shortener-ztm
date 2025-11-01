use crate::core::security::{
    jwt::{Claims, JwtKeys, gen_refresh_token, hash_refresh_token},
    password::{hash_password, verify_password},
};
use crate::features::auth::dto::{AuthBundle, SignInReq, SignUpReq};
use crate::features::auth::repositories::AuthRepository;
use crate::features::users::repositories::UserRepository;
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use std::{net::IpAddr, sync::Arc};
use uuid::Uuid;

const MAX_USER_NAME_LENGTH: usize = 30;
const GRACE_SECONDS: i64 = 120;
const REFRESH_TTL_DAYS: i64 = 30;

pub struct AuthService {
    users_repo: Arc<dyn UserRepository>,
    auth_repo: Arc<dyn AuthRepository>,
    jwt: JwtKeys,
    access_ttl: Duration,
    pub_key: String,
}

impl AuthService {
    pub fn new(
        users_repo: Arc<dyn UserRepository>,
        auth_repo: Arc<dyn AuthRepository>,
        jwt: JwtKeys,
        access_ttl: Duration,
        pub_key: String,
    ) -> Self {
        Self {
            users_repo,
            auth_repo,
            jwt,
            access_ttl,
            pub_key,
        }
    }

    pub async fn sign_in(&self, req: SignInReq, ip: Option<IpAddr>) -> anyhow::Result<AuthBundle> {
        let usr = self
            .users_repo
            .find_user_by_email(&req.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("email not found"))?;

        if !verify_password(
            &req.password,
            &usr.password_hash.clone().unwrap(),
            &self.pub_key,
        )? {
            return Err(anyhow::anyhow!("invalid credentials"));
        }

        self.issue_bundle(
            usr.id,
            usr.jwt_token_version,
            req.device_id.as_deref(),
            None,
            ip,
        )
        .await
    }

    pub async fn sign_up(&self, req: SignUpReq, ip: Option<IpAddr>) -> anyhow::Result<AuthBundle> {
        if !EmailAddress::is_valid(&req.email) {
            return Err(anyhow::anyhow!("invalid email"));
        }
        if let Some(ref n) = req.display_name
            && n.len() > MAX_USER_NAME_LENGTH
        {
            return Err(anyhow::anyhow!("name too long"));
        }

        let ph = hash_password(&req.password, &self.pub_key)?;
        let usr = self
            .users_repo
            .create(&req.email, &ph, req.display_name)
            .await?;
        self.issue_bundle(
            usr.id,
            usr.jwt_token_version,
            req.device_id.as_deref(),
            None,
            ip,
        )
        .await
    }

    async fn issue_bundle(
        &self,
        user_id: Uuid,
        ver: u32,
        device_id_opt: Option<&str>,
        ua: Option<&str>,
        ip: Option<IpAddr>,
    ) -> anyhow::Result<AuthBundle> {
        let device_id = device_id_opt.unwrap_or("default");
        let access_token = self.jwt.sign(user_id, ver, self.access_ttl)?;

        let refresh_token = gen_refresh_token();
        let refresh_hash = hash_refresh_token(&refresh_token, &self.pub_key)?;
        let absolute_expires = Utc::now() + Duration::days(REFRESH_TTL_DAYS);

        let _id = self
            .auth_repo
            .upsert_refresh_device(user_id, device_id, &refresh_hash, absolute_expires, ua, ip)
            .await?;

        Ok(AuthBundle {
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> anyhow::Result<AuthBundle> {
        let rt_hash = hash_refresh_token(refresh_token, &self.pub_key)?;

        let Some(dev) = self
            .auth_repo
            .get_refresh_device_by_rt(device_id, &rt_hash)
            .await?
        else {
            return Err(anyhow::anyhow!("invalid refresh token"));
        };

        if dev.revoked_at.is_some() {
            return Err(anyhow::anyhow!("device revoked"));
        }
        if Utc::now() > dev.absolute_expires {
            return Err(anyhow::anyhow!("refresh expired"));
        }

        let matches_current = rt_hash == dev.current_hash;
        let matches_previous = dev
            .previous_hash
            .as_ref()
            .map(|p| *p == rt_hash)
            .unwrap_or(false);

        if !matches_current && matches_previous {
            if let Some(rot) = dev.last_rotated_at {
                if (Utc::now() - rot).num_seconds() > GRACE_SECONDS {
                    let _ = self.auth_repo.revoke_device(dev.id).await;
                    return Err(anyhow::anyhow!("stale refresh token"));
                }
            } else {
                let _ = self.auth_repo.revoke_device(dev.id).await;
                return Err(anyhow::anyhow!("stale refresh token"));
            }
        } else if !matches_current {
            let _ = self.auth_repo.revoke_device(dev.id).await;
            return Err(anyhow::anyhow!("invalid refresh token"));
        }

        let user = self
            .users_repo
            .find_user_by_id(dev.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found"))?;
        let access_token = self
            .jwt
            .sign(user.id, user.jwt_token_version, self.access_ttl)?;

        let new_rt = gen_refresh_token();
        let new_hash = hash_refresh_token(&new_rt, &self.pub_key)?;

        self.auth_repo
            .rotate_refresh_hash(dev.id, &new_hash, Utc::now())
            .await?;

        Ok(AuthBundle {
            access_token,
            refresh_token: new_rt,
        })
    }

    pub async fn sign_out(&self, user_id: Uuid, device_id: &str) -> anyhow::Result<()> {
        if let Some(dev) = self
            .auth_repo
            .get_refresh_device_by_user_id(device_id, user_id)
            .await?
        {
            self.auth_repo.revoke_device(dev.id).await?;
        }
        Ok(())
    }

    pub async fn sign_out_all(&self, user_id: Uuid) -> anyhow::Result<()> {
        self.auth_repo.revoke_all(user_id).await?;
        self.users_repo.bump_jwt_version(user_id).await?;
        Ok(())
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_pwd: String,
        new_pwd: String,
    ) -> anyhow::Result<()> {
        let stored = self.users_repo.get_password_hash_by_id(user_id).await?;
        if !verify_password(&old_pwd, &stored, &self.pub_key)? {
            return Err(anyhow::anyhow!("invalid old password"));
        }
        let new_hash = hash_password(&new_pwd, &self.pub_key)?;
        self.users_repo.update_password(user_id, &new_hash).await?;

        self.sign_out_all(user_id).await
    }

    pub async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let claims = self
            .jwt
            .verify(token)
            .map_err(|_| anyhow::anyhow!("Invalid token"))?;

        let user = self
            .users_repo
            .find_user_by_id(claims.sub)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        if user.jwt_token_version != claims.ver {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }
        Ok(claims)
    }
}
