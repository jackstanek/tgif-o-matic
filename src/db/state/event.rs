//! Events that can occur while the server is running.

use crate::random_str::JoinCode;

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct TeamId(usize);

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct PlayerId(usize);

/// Event that can occur while the server is running
pub enum GameEvent {
    GameStarted {
        at: jiff::Timestamp,
        root_seed: u64,
        game_join_code: JoinCode,
    },
    TeamCreated {
        id: TeamId,
        name: String,
        team_join_code: JoinCode,
    },
    PlayerJoined {
        id: PlayerId,
        name: String,
    },
}
