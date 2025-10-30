pub mod postgres;

use std::sync::Arc;

use crate::{
    configuration::{DatabaseSettings, DatabaseType},
    features::{auth::repositories::AuthRepository, users::repositories::UserRepository},
    infrastructure::db::postgres::{auth::PgAuthRepository, users::PgUserRepository},
};

use sqlx::{PgPool, SqlitePool};

#[derive(Clone)]
pub enum DbPools {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

pub struct RepoSet {
    pub users: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthRepository>,
}

pub async fn make_pools(cfg: &DatabaseSettings) -> anyhow::Result<DbPools> {
    match cfg.r#type {
        DatabaseType::Postgres => {
            let pool = postgres::get_connection_pool(cfg).await?;
            Ok(DbPools::Postgres(pool))
        } // DatabaseType::Sqlite => {
        //     let pool = crate::database::sqlite::get_connection_pool(cfg).await?;
        //     Ok(DbPools::Sqlite(pool))
        // }
        _ => unimplemented!("Repository for this database type is not implemented yet"),
    }
}

pub async fn make_repos(pools: &DbPools) -> RepoSet {
    match pools {
        DbPools::Postgres(pg) => {
            postgres::migrate(pg).await.unwrap();
            RepoSet {
                users: Arc::new(PgUserRepository { pool: pg.clone() }),
                auth: Arc::new(PgAuthRepository { pool: pg.clone() }),
                // urls: Arc::new(PgUrlRepository { pool: pg.clone() }),
            }
        }
        // DbPools::Sqlite(sq) => RepoSet {
        //     users: Arc::new(SqUserRepository { pool: sq.clone() }),
        //     auth: Arc::new(SqAuthRepository { pool: sq.clone() }),
        //     urls: Arc::new(SqUrlRepository { pool: sq.clone() }),
        // },
        _ => unimplemented!("Repository for this database type is not implemented yet"),
    }
}
