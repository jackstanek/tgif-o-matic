//! Axum routes for app endpoints

use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tower_http::services;

use crate::{appstate::AppState, domain::auth::Argon2Error, routes::user::AuthError};

mod dashboard;
mod index;
mod user;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("hashing error: {0}")]
    HashingError(#[from] Argon2Error),
    #[error("rendering error: {0}")]
    RenderError(#[from] askama::Error),
    #[error("auth error: {0}")]
    AuthError(#[from] AuthError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::DbError(error) => {
                tracing::error!("database error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::HashingError(error) => {
                tracing::error!("hashing error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::RenderError(error) => {
                tracing::error!("rendering error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::AuthError(error) => error.into_response(),
        }
    }
}

pub(crate) fn build_router(state: AppState) -> Router {
    let serve_dir = services::ServeDir::new("assets");
    Router::new()
        .route("/login", get(user::login_get).post(user::login_post))
        .route("/dashboard", get(dashboard::dashboard_get))
        .route("/", get(index::index_get))
        .nest_service("/assets", serve_dir)
        .with_state(state)
}
