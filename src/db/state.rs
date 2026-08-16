//! Game loop state machine.

mod event;
mod phase;

pub(crate) struct GameState {
    phase: phase::GamePhase,
}
