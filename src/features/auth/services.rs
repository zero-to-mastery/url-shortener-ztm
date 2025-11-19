use crate::{
    ApiError, ClientMeta,
    core::security::{
        jwt::{Claims, JwtKeys, gen_refresh_token, hash_refresh_token},
        password::{
            NormalizedPassword, generate_verification_code, hash_password, hash_verification_code,
            validate_policy, verify_password, verify_verification_code,
        },
    },
    features::{
        auth::{
            dto::{AuthBundle, SignInReq, SignUpReq},
            repositories::{
                AuthRepoError, AuthRepository, AuthenticationAction, AuthenticationChallenge,
            },
        },
        users::repositories::UserRepository,
    },
    infrastructure::email::EmailService,
};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use std::{net::IpAddr, sync::Arc};
use uuid::Uuid;
const MAX_USER_NAME_LENGTH: usize = 30;
const GRACE_SECONDS: i64 = 120;
const REFRESH_TTL_DAYS: i64 = 30;
const MAX_ATTEMPTS_ALLOWED: u8 = 5;
const DEFAULT_DEVICE_ID: &str = "default";

pub struct AuthService {
    users_repo: Arc<dyn UserRepository>,
    auth_repo: Arc<dyn AuthRepository>,
    jwt: JwtKeys,
    access_ttl: Duration,
    pwd_pepper: SecretString,
    email_service: EmailService,
}

impl AuthService {
    pub fn new(
        users_repo: Arc<dyn UserRepository>,
        auth_repo: Arc<dyn AuthRepository>,
        jwt: JwtKeys,
        access_ttl: Duration,
        pwd_pepper: SecretString,
        email_service: EmailService,
    ) -> Self {
        Self {
            users_repo,
            auth_repo,
            jwt,
            access_ttl,
            pwd_pepper,
            email_service,
        }
    }

    pub async fn sign_up(&self, req: SignUpReq, ip: Option<IpAddr>) -> anyhow::Result<AuthBundle> {
        if !EmailAddress::is_valid(&req.email) {
            return Err(anyhow::anyhow!("invalid email"));
        } else if self.users_repo.email_exists(&req.email).await? {
            return Err(anyhow::anyhow!("email already registered"));
        }

        if let Some(ref n) = req.display_name
            && n.len() > MAX_USER_NAME_LENGTH
        {
            return Err(anyhow::anyhow!("name too long"));
        }
        let norm_pwd = NormalizedPassword::try_from(&req.password)?;

        validate_policy(&norm_pwd)?;
        let pw_hash = hash_password(&norm_pwd, self.pwd_pepper.expose_secret())?;
        let usr = self
            .users_repo
            .create(&req.email, &pw_hash, req.display_name)
            .await?;
        let code = generate_verification_code();
        let code_hash = hash_verification_code(&code, self.pwd_pepper.expose_secret())?;

        self.auth_repo
            .create_or_refresh_auth_challenge(
                usr.id,
                AuthenticationAction::VerifyEmail,
                None,
                &code_hash,
                None,
                Utc::now() + Duration::hours(1),
                None,
            )
            .await?;

        let bundle_fut = self.issue_bundle(
            usr.id,
            usr.jwt_token_version,
            req.device_id.as_deref(),
            None,
            ip,
        );
        let email_fut = async move {
            if let Err(err) = self
                .email_service
                .send_verification_code(&req.email, &code)
                .await
            {
                tracing::warn!(email=%req.email, error=%err, "send verification code failed");
            }
        };

        let (bundle, _) = tokio::join!(bundle_fut, email_fut);
        bundle
    }

    pub async fn sign_in(&self, req: SignInReq, ip: Option<IpAddr>) -> anyhow::Result<AuthBundle> {
        if !EmailAddress::is_valid(&req.email) {
            return Err(anyhow::anyhow!("invalid email"));
        }

        let usr = self
            .users_repo
            .find_user_by_email(&req.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("email not found"))?;

        if !verify_password(
            &req.password,
            usr.password_hash.as_deref().unwrap(),
            self.pwd_pepper.expose_secret(),
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

    pub async fn request_email_change(
        &self,
        user_id: Uuid,
        new_email: &str,
        current_pwd: &SecretString,
        meta: ClientMeta,
    ) -> anyhow::Result<()> {
        self.verify_password(user_id, current_pwd).await?;

        if !EmailAddress::is_valid(new_email) {
            return Err(anyhow::anyhow!("invalid email"));
        } else if self.users_repo.email_exists(new_email).await? {
            return Err(anyhow::anyhow!("email already registered"));
        }

        let user = self
            .users_repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found"))?;

        let old_email = user.email.clone();
        let new_email_owned = new_email.to_string();

        let meta_payload = json!({
            "old_email": user.email.clone(),
            "new_email": new_email.to_string(),
            "ip": meta.ip.map(|ip| ip.to_string()),
            "user_agent": meta.user_agent.clone(),
        });

        let verification_fut = self.send_verification_code(
            user_id,
            &new_email_owned,
            Some(&new_email_owned),
            AuthenticationAction::ChangeEmail,
            Some(&meta_payload),
        );

        let notification_fut = async {
            let body = format!(
                r#"<h2>Email Change Requested</h2>
            <p>You have requested to change your email to <strong>{}</strong>.</p>
            <p>If you did not make this request, please contact support immediately.</p>"#,
                new_email_owned
            );

            if let Err(e) = self
                .email_service
                .send_email(&old_email, "Email Change Requested", &body)
                .await
            {
                tracing::warn!(email=%old_email, error=%e, "failed to send notification email");
            }
        };

        let (result, _) = tokio::join!(verification_fut, notification_fut);
        result
    }

    pub async fn confirm_email_change(&self, user_id: Uuid, code: &str) -> anyhow::Result<()> {
        let challenge = self
            .verify_code(user_id, AuthenticationAction::ChangeEmail, code)
            .await?;

        if let Some(target_email) = challenge.target {
            self.users_repo.update_email(user_id, &target_email).await?;
        } else {
            return Err(anyhow::anyhow!("no target email in challenge"));
        }

        Ok(())
    }

    pub async fn refresh(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> anyhow::Result<AuthBundle> {
        let rt_hash = hash_refresh_token(refresh_token, self.pwd_pepper.expose_secret())?;

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
        let new_hash = hash_refresh_token(&new_rt, self.pwd_pepper.expose_secret())?;

        self.auth_repo
            .rotate_refresh_hash(dev.id, &new_hash, Utc::now())
            .await?;

        Ok(AuthBundle {
            access_token,
            refresh_token: new_rt,
        })
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_pwd: &SecretString,
        new_pwd: &SecretString,
    ) -> anyhow::Result<()> {
        self.verify_password(user_id, old_pwd).await?;

        self.reset_password(user_id, new_pwd).await
    }

    pub async fn reset_password(
        &self,
        user_id: Uuid,
        new_pwd: &SecretString,
    ) -> anyhow::Result<()> {
        let norm_pwd = NormalizedPassword::try_from(new_pwd)?;
        validate_policy(&norm_pwd)?;
        let new_hash = hash_password(&norm_pwd, self.pwd_pepper.expose_secret())?;
        self.users_repo.update_password(user_id, &new_hash).await?;

        self.sign_out_all(user_id).await
    }

    pub async fn send_verification_code(
        &self,
        user_id: Uuid,
        email: &str,
        target: Option<&str>,
        action: AuthenticationAction,
        meta: Option<&serde_json::Value>,
    ) -> anyhow::Result<()> {
        let code = generate_verification_code();
        let code_hash = hash_verification_code(&code, self.pwd_pepper.expose_secret())?;

        self.auth_repo
            .create_or_refresh_auth_challenge(
                user_id,
                action.clone(),
                target,
                &code_hash,
                meta,
                Utc::now() + Duration::hours(1),
                None,
            )
            .await?;

        match action {
            AuthenticationAction::VerifyEmail | AuthenticationAction::ChangeEmail => {
                self.email_service
                    .send_verification_code(email, &code)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to send email: {}", e))?;
            }
            AuthenticationAction::ResetPassword => {
                self.email_service
                    .send_password_reset_code(email, &code)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to send email: {}", e))?;
            }
        }

        Ok(())
    }

    pub async fn verify_code(
        &self,
        user_id: Uuid,
        action: AuthenticationAction,
        code: &str,
    ) -> anyhow::Result<AuthenticationChallenge> {
        let Some(challenge) = self
            .auth_repo
            .get_auth_challenge(user_id, action.clone())
            .await?
        else {
            return Err(anyhow::anyhow!("challenge not found"));
        };

        if challenge.attempts >= MAX_ATTEMPTS_ALLOWED {
            return Err(anyhow::anyhow!(
                "too many attempts, please request a new code"
            ));
        }

        if Utc::now() > challenge.expires_at {
            return Err(anyhow::anyhow!("challenge expired"));
        }

        if !verify_verification_code(code, &challenge.code_hash, self.pwd_pepper.expose_secret())? {
            self.auth_repo
                .increase_auth_challenge_attempts(challenge.id)
                .await?;
            return Err(anyhow::anyhow!("invalid code"));
        }

        self.auth_repo
            .confirm_authentication_challenge(user_id, action, Utc::now())
            .await?;

        Ok(challenge)
    }

    pub async fn verify_password(&self, uuid: Uuid, password: &SecretString) -> anyhow::Result<()> {
        let stored = self.users_repo.get_password_hash_by_id(uuid).await?;

        if !verify_password(password, &stored, self.pwd_pepper.expose_secret())? {
            return Err(anyhow::anyhow!("invalid password"));
        }

        Ok(())
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

    async fn issue_bundle(
        &self,
        user_id: Uuid,
        ver: u32,
        device_id_opt: Option<&str>,
        ua: Option<&str>,
        ip: Option<IpAddr>,
    ) -> anyhow::Result<AuthBundle> {
        let device_id = device_id_opt.unwrap_or(DEFAULT_DEVICE_ID);
        let access_token = self.jwt.sign(user_id, ver, self.access_ttl)?;

        let refresh_token = gen_refresh_token();
        let refresh_hash = hash_refresh_token(&refresh_token, self.pwd_pepper.expose_secret())?;
        let absolute_expires = Utc::now() + Duration::days(REFRESH_TTL_DAYS);

        let _ = self
            .auth_repo
            .upsert_refresh_device(user_id, device_id, &refresh_hash, absolute_expires, ua, ip)
            .await?;

        Ok(AuthBundle {
            access_token,
            refresh_token,
        })
    }
}

impl From<AuthRepoError> for ApiError {
    fn from(e: AuthRepoError) -> Self {
        match e {
            AuthRepoError::Cooldown(_) => ApiError::Cooldown,
            AuthRepoError::AlreadyActive => ApiError::AlreadyActive,
            AuthRepoError::EmailTaken => ApiError::EmailTaken,
            AuthRepoError::NotFound => ApiError::NotFound("resource not found".into()),
            AuthRepoError::Transient => ApiError::Internal("transient".into()),
            AuthRepoError::Internal => ApiError::Internal("internal".into()),
        }
    }
}
