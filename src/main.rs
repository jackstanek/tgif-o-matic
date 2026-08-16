use axum::{Router, routing::get};
use log::debug;
use rand_chacha::rand_core::SeedableRng;

use crate::config::BackendConfig;

mod config;
mod db;
mod joincode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = BackendConfig::from_env();
    let db = db::init_db(&config)
        .await
        .expect("could not initialize database");

    let mut rng = rand_chacha::ChaCha12Rng::from_rng(&mut rand::rngs::OsRng)?;
    debug!("bootstrapped RNG");

    db::init_admin_account(&config, &db, &mut rng).await?;

    let app = Router::new().route("/", get(|| async { "Hello world!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
