//! Game instances. Each game instance runs an active trivia game.

/// Identifier for a game instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameId(usize);

/// Instance of a game.
struct GameInstance {
    id: crate::game::GameId,
    owner: crate::user::UserId,
    template: crate::template::TemplateId,
}
