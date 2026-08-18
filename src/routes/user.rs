//! User management and authentication routes

use askama::Template;
use axum::{Form, extract::State, response::Html};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::db;

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
    State(pool): State<SqlitePool>,
    Form(input): Form<AuthInput>,
) -> Html<&'static str> {
    let res = db::check_credentials(&pool, &input.username, &input.password)
        .await
        .is_ok_and(|x| x);
    if res {
        Html("success")
    } else {
        Html("failure")
    }
}
