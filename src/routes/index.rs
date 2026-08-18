//! Main route for the landing page.

use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

pub(crate) async fn index_get() -> Html<String> {
    Html(IndexTemplate {}.to_string())
}
