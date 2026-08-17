//! User management and authentication routes

use axum::{Form, extract::State, response::Html};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::db;

#[derive(Deserialize, Clone)]
pub(crate) struct AuthInput {
    username: String,
    password: String,
}

/// Send login form to client
pub(crate) async fn show_login_form() -> Html<&'static str> {
    Html(
        r#"
        <form method="post">
            <input type="text" name="username" />
            <input type="password" name="password" />
            <button type="submit">Login</button>
        </form>
        "#,
    )
}

/// Auth flow for registered admin login
pub(crate) async fn accept_login_form(
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
