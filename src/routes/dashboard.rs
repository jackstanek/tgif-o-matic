//! Admin dashboard for game, player, and team managemnet

use askama::Template;
use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::{
    appstate::AppState,
    routes::{AppError, user::CurrentUser},
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
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Response<Body>, AppError> {
    if let CurrentUser::Admin(admin) = user {
        let content = DashboardTemplate::new(admin.username());
        Ok(content.render().map(Html::from)?.into_response())
    } else {
        Ok("not logged in".into_response())
    }
}
