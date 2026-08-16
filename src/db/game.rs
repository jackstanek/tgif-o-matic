//! Game instances. Each game instance runs an active trivia game.

use sqlx::prelude::FromRow;

db_id_type!(GameId);

/// Instance of a game.
#[derive(FromRow)]
struct GameInstance {
    id: crate::db::game::GameId,
    owner: crate::db::user::AdminId,
    template: crate::db::template::TemplateId,
}
