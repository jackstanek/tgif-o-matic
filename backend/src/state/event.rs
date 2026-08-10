//! Events that can occur while the server is running.

use crate::joincode::JoinCode;

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct TeamId(usize);

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct PlayerId(usize);

/// Length of the join code for games and teams
const JC_LEN: usize = 4;

/// Event that can occur while the server is running
pub enum GameEvent {
    GameStarted {
        at: jiff::Timestamp,
        root_seed: u64,
        game_join_code: JoinCode<JC_LEN>,
    },
    TeamCreated {
        id: TeamId,
        name: String,
        team_join_code: JoinCode<JC_LEN>,
    },
    PlayerJoined {
        id: PlayerId,
        name: String,
    },
}
