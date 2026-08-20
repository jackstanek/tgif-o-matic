//! User management and authentication routes

use askama::Template;
use axum::{
    Form,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::db::{self, AdminId, PlayerId, SessionRow};
use crate::domain;
use crate::routes::AppError;
use crate::{appstate::AppState, domain::auth::SessionToken};

/// Extractor for the current user.
pub(crate) enum CurrentUser {
    Admin(AdminId),
    Player(PlayerId),
}

#[derive(Debug)]
pub(crate) enum AuthError {
    NoSession,
    InvalidSession,
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthError::NoSession | AuthError::InvalidSession => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            AuthError::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut exec = state.pool().acquire().await.or_else(|e| {
            tracing::error!("couldn't connect to the database: {e}");
            Err(AuthError::Internal)
        })?;
        let jar = CookieJar::from_headers(&parts.headers);
        let sid = jar.get("sid").ok_or(AuthError::NoSession)?;
        let tok = SessionToken::from_bytes(sid.value_trimmed().as_bytes())
            .ok_or(AuthError::InvalidSession)?;
        let session = db::get_session_by_token_hash(&mut *exec, &tok.sha256())
            .await
            .or_else(|e| {
                tracing::error!("couldn't fetch session from database: {e}");
                Err(AuthError::Internal)
            })?
            .ok_or(AuthError::NoSession)?;
        if let Some(admin_id) = session.admin_id {
            Ok(CurrentUser::Admin(admin_id))
        } else if let Some(player_id) = session.player_id {
            Ok(CurrentUser::Player(player_id))
        } else {
            tracing::error!("db invariant: session has no admin or player id");
            Err(AuthError::Internal)
        }
    }
}

impl From<db::AdminRow> for domain::auth::Admin {
    fn from(row: db::AdminRow) -> Self {
        domain::auth::Admin::new(row.username, row.pw_hash)
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPageTemplate {}

#[derive(Deserialize, Clone)]
pub(crate) struct AuthInput {
    username: String,
    password: String,
}

/// Send login form to client
pub(crate) async fn login_get() -> Html<String> {
    let content = LoginPageTemplate {}.render().unwrap().to_string();
    Html(content)
}

/// Auth flow for registered admin login
pub(crate) async fn login_post(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(input): Form<AuthInput>,
) -> Result<Html<&'static str>, AppError> {
    let mut conn = state.pool().acquire().await?;
    let row = db::AdminRow::by_username(&mut *conn, &input.username).await?;

    if let Some(admin) = row {
        let tok = SessionToken::generate(&mut state.rng());
        db::create_admin_session(&mut *conn, &tok.sha256(), admin.id).await?;
        todo!()
    } else {
        todo!()
    }
}
