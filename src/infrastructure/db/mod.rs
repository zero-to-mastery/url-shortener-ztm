pub mod postgres;

use std::sync::Arc;

use crate::{
    configuration::{DatabaseSettings, DatabaseType},
    features::{auth::repositories::AuthRepository, users::repositories::UserRepository},
    infrastructure::db::postgres::{auth::PgAuthRepository, users::PgUserRepository},
};

use sqlx::{PgPool, SqlitePool};

#[derive(Clone)]
pub enum DbPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

pub struct RepoSet {
    pub users: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthRepository>,
}

pub async fn make_pools(cfg: &DatabaseSettings) -> anyhow::Result<DbPool> {
    match cfg.r#type {
        DatabaseType::Postgres => {
            let pool = postgres::get_connection_pool(cfg).await?;
            postgres::migrate(&pool).await?;
            Ok(DbPool::Postgres(pool))
        } // DatabaseType::Sqlite => {
        //     let pool = crate::database::sqlite::get_connection_pool(cfg).await?;
        //     Ok(DbPools::Sqlite(pool))
        // }
        _ => unimplemented!("Repository for this database type is not implemented yet"),
    }
}

pub async fn make_repos(pools: &DbPool) -> RepoSet {
    match pools {
        DbPool::Postgres(pg) => RepoSet {
            users: Arc::new(PgUserRepository { pool: pg.clone() }),
            auth: Arc::new(PgAuthRepository { pool: pg.clone() }),
        },
        // DbPools::Sqlite(sq) => RepoSet {
        //     users: Arc::new(SqUserRepository { pool: sq.clone() }),
        //     auth: Arc::new(SqAuthRepository { pool: sq.clone() }),
        //     urls: Arc::new(SqUrlRepository { pool: sq.clone() }),
        // },
        _ => unimplemented!("Repository for this database type is not implemented yet"),
    }
}
