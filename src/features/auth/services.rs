use std::sync::Arc;

use chrono::Duration;
use email_address::EmailAddress;

use crate::{
    core::security::{
        jwt::JwtKeys,
        password::{hash_password, verify_password},
    },
    features::{
        auth::{
            dto::{SignInReq, SignUpReq, TokenResp},
            repositories::AuthRepository,
        },
        users::repositories::UserRepository,
    },
};

const MAX_USER_NAME_LENGTH: usize = 30;

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    auth: Arc<dyn AuthRepository>,
    jwt: JwtKeys,
    access_ttl: Duration,
    pub_key: String,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        auth: Arc<dyn AuthRepository>,
        jwt: JwtKeys,
        access_ttl: Duration,
        pub_key: String,
    ) -> Self {
        Self {
            users,
            auth,
            jwt,
            access_ttl,
            pub_key,
        }
    }

    pub async fn sign_in(&self, req: SignInReq) -> anyhow::Result<TokenResp> {
        let usr = self
            .users
            .find_user_by_email(&req.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("email not found"))?;

        if !verify_password(&req.password, &usr.password_hash.unwrap(), &self.pub_key)? {
            return Err(anyhow::anyhow!("Invalid credentials"));
        }

        let access_token = self
            .jwt
            .sign(usr.id, usr.jwt_token_version, self.access_ttl)?;

        Ok(TokenResp { access_token })
    }

    pub async fn sign_up(&self, req: SignUpReq) -> anyhow::Result<TokenResp> {
        if !EmailAddress::is_valid(&req.email) {
            return Err(anyhow::anyhow!("Invalid email format"));
        }

        if let Some(ref name) = req.display_name
            && name.len() > MAX_USER_NAME_LENGTH
        {
            return Err(anyhow::anyhow!("Name too long"));
        }

        let hash_password = hash_password(req.password.as_str(), &self.pub_key)?;

        let usr = self
            .users
            .create(&req.email, &hash_password, req.display_name)
            .await?;

        let access_token = self
            .jwt
            .sign(usr.id, usr.jwt_token_version, self.access_ttl)?;

        Ok(TokenResp { access_token })
    }
}
