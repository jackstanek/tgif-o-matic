//! User management and authentication routes

use askama::Template;
use axum::{
    Form,
    body::Body,
    extract::{FromRequestParts, State},
    http::{Response, StatusCode, request::Parts},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use derive_more::From;
use serde::Deserialize;

use crate::domain;
use crate::routes::AppError;
use crate::{appstate::AppState, domain::auth::SessionToken};
use crate::{
    db::{self, AdminId, PlayerId},
    domain::auth::Admin,
};

/// Extractor for the current logged-in user. In the event that no user session
/// exists, a handler with this extractor will bounce the request.
#[derive(Debug)]
pub(crate) enum CurrentUser {
    Admin(AdminId),
    Player(PlayerId),
}

/// Extractor for a user which might be logged in. This is similar to [`CurrentUser`],
/// but will not bounce the request if no session is found, instead wrapping [`None`].
#[derive(Debug, From)]
pub(crate) struct MaybeCurrentUser(Option<CurrentUser>);

#[derive(Debug)]
pub(crate) enum AuthError {
    NoSession,
    InvalidSession,
    BadCredentials,
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AuthError::NoSession | AuthError::InvalidSession | AuthError::BadCredentials => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            AuthError::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

/// Get the user corresponding to the given session token. If no session exists
/// matching the session ID, returns [`AuthError::NoSession`]. If the session
/// token is malformed, returns [`AuthError::InvalidSession`].
async fn validate_session<'e, E>(exec: E, sid: &str) -> Result<Option<CurrentUser>, AuthError>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let tok = SessionToken::from_base64(sid).ok_or(AuthError::InvalidSession)?;
    let Some(session) = db::get_session_by_token_hash(exec, &tok.sha256())
        .await
        .map_err(|e| {
            tracing::error!("couldn't fetch session from database: {e}");
            AuthError::Internal
        })?
    else {
        return Ok(None);
    };

    if let Some(admin_id) = session.admin_id {
        tracing::debug!("validated admin session for {admin_id:?}");
        Ok(Some(CurrentUser::Admin(admin_id)))
    } else if let Some(player_id) = session.player_id {
        tracing::debug!("validated player session for {player_id:?}");
        Ok(Some(CurrentUser::Player(player_id)))
    } else {
        tracing::debug!("db invariant: session has no admin or player id");
        Err(AuthError::Internal)
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut exec = state.pool().acquire().await.map_err(|e| {
            tracing::error!("couldn't connect to the database: {e}");
            AuthError::Internal
        })?;
        let jar = CookieJar::from_headers(&parts.headers);
        let sid = jar.get("sid").ok_or(AuthError::NoSession)?;
        validate_session(&mut *exec, sid.value())
            .await
            .and_then(|mu| mu.ok_or(AuthError::InvalidSession))
    }
}

impl FromRequestParts<AppState> for MaybeCurrentUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut exec = state.pool().acquire().await.map_err(|e| {
            tracing::error!("couldn't connect to the database: {e}");
            AuthError::Internal
        })?;
        let jar = CookieJar::from_headers(&parts.headers);
        if let Some(sid) = jar.get("sid") {
            validate_session(&mut *exec, sid.value())
                .await
                .map(MaybeCurrentUser::from)
        } else {
            // No session
            Ok(None.into())
        }
    }
}

impl<'a> From<&'a db::AdminRow> for domain::auth::Admin<'a> {
    fn from(row: &'a db::AdminRow) -> Self {
        domain::auth::Admin::new(&row.username, &row.pw_hash)
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
pub(crate) async fn login_get(MaybeCurrentUser(user): MaybeCurrentUser) -> Response<Body> {
    if user.is_some_and(|user| matches!(user, CurrentUser::Admin(_))) {
        // TODO: change this to the dashboard
        return Redirect::to("/").into_response();
    }
    let content = LoginPageTemplate {}.render().unwrap().to_string();
    Html(content).into_response()
}

/// Auth flow for registered admin login
pub(crate) async fn login_post(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(input): Form<AuthInput>,
) -> Result<Response<Body>, AppError> {
    let mut conn = state.pool().acquire().await?;
    let row = db::AdminRow::by_username(&mut *conn, &input.username).await?;

    if let Some(arow) = row {
        let admin = Admin::from(&arow);
        if !admin.check_password(&input.password)? {
            return Err(AuthError::BadCredentials.into());
        }
        let tok = SessionToken::generate(&mut state.rng());
        db::create_admin_session(&mut *conn, &tok.sha256(), arow.id).await?;

        let cookie = Cookie::build(("sid", tok.base64_encode()))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .path("/")
            .build();
        let jar = jar.add(cookie);
        Ok((jar, Redirect::to("/")).into_response())
    } else {
        Err(AuthError::BadCredentials.into())
    }
}
