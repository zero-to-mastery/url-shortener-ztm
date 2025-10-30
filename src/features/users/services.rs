// features/users/services.rs
use crate::features::users::dto::MeResp;
use crate::features::users::repositories::UserRepository;
use anyhow::{Result, anyhow};
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

    // pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<UserCore>> {
    //     self.repo.find_core_by_id(id).await
    // }

    // pub async fn change_password(&self, user_id: Uuid, old: &str, newp: &str) -> Result<()> {
    //     let stored = self.repo.get_password_hash(user_id).await?;
    //     if !verify_password(old, &stored)? {
    //         return Err(anyhow!("Old password incorrect"));
    //     }
    //     let new_hash = hash_password(newp)?;
    //     self.repo.update_password(user_id, &new_hash).await
    // }

    // pub async fn change_email(&self, user_id: Uuid, new_email: &str) -> Result<()> {
    //     if self.repo.email_exists(new_email).await? {
    //         return Err(anyhow!("Email already in use"));
    //     }
    //     self.repo.update_email(user_id, new_email).await
    // }

    // pub async fn set_last_login(&self, user_id: Uuid) -> Result<()> {
    //     self.repo.set_last_login(user_id, Utc::now()).await
    // }

    // pub async fn revoke_all_tokens(&self, user_id: Uuid) -> Result<()> {
    //     self.repo.bump_jwt_version(user_id).await
    // }
}
