//! Game instances. Each game instance runs an active trivia game.

use sqlx::prelude::FromRow;

/// Identifier for a game instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameId(usize);

/// Instance of a game.
#[derive(FromRow)]
struct GameInstance {
    id: crate::game::GameId,
    owner: crate::user::AdminId,
    template: crate::template::TemplateId,
}
