//! Database access and wrapper structx

use std::time::Duration;

use tracing::{debug, info, warn};

use crate::config;

macro_rules! db_id_type {
    ($($name:ident),+) => {
        $(
            #[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
            #[sqlx(transparent)]
            pub(crate) struct $name(i32);
        )+
    };
}

mod game;
mod template;
mod user;

pub(crate) use user::*;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Initialize the SQLite conneciton pool for the main application database.
/// This applies migrations to the database upon connecting.
pub(crate) async fn init_db(config: &config::BackendConfig) -> anyhow::Result<sqlx::SqlitePool> {
    use sqlx::sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
    };
    let mut opts = SqliteConnectOptions::new()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    if let Some(path) = &config.db_path {
        info!("Using SQLite file at {}", path.to_string_lossy());
        opts = opts.filename(path);
    } else {
        warn!(
            "No SQLite database path supplied; in-memory database in use (data will NOT be persisted)"
        );
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;
    debug!("got db connection");
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// Helper to decode timestamps from the database
fn decode_timestamp(t: i64) -> sqlx::Result<time::Timestamp> {
    time::Timestamp::from_seconds(t).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}
