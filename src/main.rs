use axum::{Router, routing::get};
use log::debug;
use rand_chacha::rand_core::SeedableRng;
use tower_http::services;

use crate::config::BackendConfig;

mod config;
mod db;
mod joincode;

fn build_router() -> Router {
    let serve_dir = services::ServeDir::new("assets");
    Router::new()
        .route("/", get(|| async { "Hello world!" }))
        .nest_service("/assets", serve_dir)
}

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

    let app = build_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
