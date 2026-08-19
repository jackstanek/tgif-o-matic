//! User management and authentication routes

use askama::Template;
use axum::{Form, extract::State, response::Html};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::debug;

use crate::db;
use crate::domain;
use crate::routes::AppError;

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

/// Check a username/password combination against the database. Returns Ok(true)
/// if the username/password pair is in the admins table, and Ok(false)
/// otherwise.
pub(crate) async fn check_credentials<'e, E>(
    exec: E,
    username: &str,
    password: &str,
) -> Result<bool, AppError>
where
    E: sqlx::Acquire<'e, Database = sqlx::Sqlite>,
{
    let mut conn = exec.acquire().await?;
    let row = db::AdminRow::by_username(&mut *conn, username).await?;

    if let Some(admin) = row {
        let admin = domain::auth::Admin::from(admin);
        return Ok(admin.check_password(password)?);
    }

    debug!("auth attempted for non-existent user {username}");
    return Ok(false);
}

/// Auth flow for registered admin login
pub(crate) async fn login_post(
    State(pool): State<SqlitePool>,
    Form(input): Form<AuthInput>,
) -> Html<&'static str> {
    let res = check_credentials(&pool, &input.username, &input.password)
        .await
        .is_ok_and(|x| x);
    if res {
        Html("success")
    } else {
        Html("failure")
    }
}
