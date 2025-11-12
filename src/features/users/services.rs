// features/users/services.rs
use crate::features::users::dto::MeResp;
use crate::features::users::repositories::UserRepository;
use anyhow::{Result, anyhow};
use email_address::EmailAddress;
use uuid::Uuid;
// use chrono::Utc;
use std::sync::Arc;
// use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn me(&self, id: Uuid) -> Result<MeResp> {
        let usr = self
            .repo
            .find_user_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        Ok(MeResp {
            id: usr.id,
            email: usr.email,
            display_name: usr.display_name,
            is_email_verified: usr.is_email_verified,
            created_at: usr.created_at,
            last_login_at: usr.last_login_at,
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<MeResp> {
        if !EmailAddress::is_valid(email) {
            return Err(anyhow::anyhow!("invalid email"));
        }

        let usr = self
            .repo
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;

        Ok(MeResp {
            id: usr.id,
            email: usr.email,
            display_name: usr.display_name,
            is_email_verified: usr.is_email_verified,
            created_at: usr.created_at,
            last_login_at: usr.last_login_at,
        })
    }

    pub async fn confirm_email(&self, id: Uuid) -> Result<()> {
        self.repo.confirm_email(id).await
    }
}
