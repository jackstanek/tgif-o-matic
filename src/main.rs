use rand_chacha::rand_core::SeedableRng;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::BackendConfig;

mod config;
mod context;
mod db;
mod random_str;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = BackendConfig::from_env();
    let db = db::init_db(&config)
        .await
        .expect("could not initialize database");

    let mut rng = rand_chacha::ChaCha12Rng::from_rng(&mut rand::rngs::OsRng)?;
    debug!("bootstrapped RNG");

    db::init_admin_account(&config, &db, &mut rng).await?;

    let app = routes::build_router(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
