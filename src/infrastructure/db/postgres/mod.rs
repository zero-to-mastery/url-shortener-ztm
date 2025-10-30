pub mod auth;
pub mod users;
// pub mod urls;

use sqlx::{
    Error as SqlxError, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{DatabaseSettings, database::DatabaseError};

use std::str::FromStr;

const MAX_CAP: u32 = 96;
const MIN_CAP: u32 = 2;

pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, SqlxError> {
    let options = PgConnectOptions::from_str(&config.connection_string())?;

    let cores = num_cpus::get().max(1) as u32;

    let default_max = cores.saturating_mul(4);
    let default_min = cores.saturating_mul(2);

    let mut max_conn = config.max_connections.unwrap_or(default_max);
    let mut min_conn = config.min_connections.unwrap_or(default_min);

    max_conn = max_conn.clamp(MIN_CAP, MAX_CAP);
    if min_conn < MIN_CAP {
        min_conn = MIN_CAP;
    }

    if min_conn > max_conn {
        tracing::warn!(
            requested_min = %min_conn,
            requested_max = %max_conn,
            "min_connections > max_connections, adjusting min_connections to max_connections"
        );
        min_conn = max_conn;
    }

    tracing::warn!(cores = %cores, min_connections = %min_conn, max_connections = %max_conn, "Postgres pool sizes");

    PgPoolOptions::new()
        .max_connections(max_conn)
        .min_connections(min_conn)
        .connect_with(options)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), DatabaseError> {
    sqlx::migrate!("./migrations/pg")
        .run(pool)
        .await
        .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

    Ok(())
}
