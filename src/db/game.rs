//! Game instances. Each game instance runs an active trivia game.

use sqlx::prelude::FromRow;

/// Identifier for a game instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameId(usize);

/// Instance of a game.
#[derive(FromRow)]
struct GameInstance {
    id: crate::db::game::GameId,
    owner: crate::db::user::AdminId,
    template: crate::db::template::TemplateId,
}
