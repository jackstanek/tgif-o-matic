//! Axum routes for app endpoints

use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tower_http::services;

use crate::domain::auth::Argon2Error;

mod index;
mod user;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("database error")]
    DbError(#[from] sqlx::Error),

    #[error("hashing error: {0}")]
    HashingError(#[from] Argon2Error),
}

impl AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::DbError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::HashingError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}

pub(crate) fn build_router(state: sqlx::Pool<sqlx::Sqlite>) -> Router {
    let serve_dir = services::ServeDir::new("assets");
    Router::new()
        .route("/login", get(user::login_get).post(user::login_post))
        .route("/", get(index::index_get))
        .nest_service("/assets", serve_dir)
        .with_state(state)
}
