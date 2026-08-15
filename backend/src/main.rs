use std::time::Duration;

use axum::{Router, routing::get};
use log::{debug, info, warn};
use rand_chacha::rand_core::SeedableRng;

use crate::{config::BackendConfig, user::Admin};

mod config;
mod game;
mod joincode;
mod state;
mod template;
mod user;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Initialize the SQLite conneciton pool for the main application database.
/// This applies migrations to the database upon connecting.
async fn init_db(config: &config::BackendConfig) -> anyhow::Result<sqlx::SqlitePool> {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = BackendConfig::from_env();
    let db = init_db(&config)
        .await
        .expect("could not initialize database");

    let mut rng = rand_chacha::ChaCha12Rng::from_rng(&mut rand::rngs::OsRng)?;
    debug!("bootstrapped RNG");

    Admin::init_admin_account(&config, &db, &mut rng).await?;

    let app = Router::new().route("/", get(|| async { "Hello world!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
