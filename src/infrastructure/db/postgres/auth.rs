use async_trait::async_trait;

use sqlx::PgPool;

use crate::features::auth::repositories::AuthRepository;

#[derive(Clone)]
pub struct PgAuthRepository {
    pub pool: PgPool,
}

#[async_trait]
impl AuthRepository for PgAuthRepository {}
