//! Axum routes for app endpoints

use axum::{Router, routing::get};
use tower_http::services;

mod index;
mod user;

pub(crate) fn build_router(state: sqlx::Pool<sqlx::Sqlite>) -> Router {
    let serve_dir = services::ServeDir::new("assets");
    Router::new()
        .route("/login", get(user::login_get).post(user::login_post))
        .route("/", get(index::index_get))
        .nest_service("/assets", serve_dir)
        .with_state(state)
}
