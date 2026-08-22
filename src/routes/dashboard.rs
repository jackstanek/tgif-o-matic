//! Admin dashboard for game, player, and team managemnet

use askama::Template;
use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::{
    appstate::AppState,
    routes::{
        AppError,
        user::{CurrentAdmin, CurrentUser},
    },
};

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate<'s> {
    user: &'s str,
}

impl<'s> DashboardTemplate<'s> {
    fn new(user: &'s str) -> Self {
        Self { user }
    }
}

pub(crate) async fn dashboard_get(
    CurrentAdmin(user): CurrentAdmin,
) -> Result<Response<Body>, AppError> {
    let content = DashboardTemplate::new(user.username());
    Ok(content.render().map(Html::from)?.into_response())
}
