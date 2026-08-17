//! Axum routes for app endpoints

use axum::{Router, routing::get};
use tower_http::services;

pub(crate) mod user;

pub(crate) fn build_router(state: sqlx::Pool<sqlx::Sqlite>) -> Router {
    let serve_dir = services::ServeDir::new("assets");
    Router::new()
        .route(
            "/",
            get(user::show_login_form).post(user::accept_login_form),
        )
        .nest_service("/assets", serve_dir)
        .with_state(state)
}
