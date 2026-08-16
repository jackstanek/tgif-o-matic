//! Game phases. Each phase corresponds to a state in the game's state machine
//! model. The game phase evolves according to the state machine's transition
//! function.

pub(crate) enum GamePhase {
    /// In lobby, players joining and forming teams
    Lobby,
    /// A question is open and can be answered. (Any previous questions this
    /// section can also be edited.)
    QuestionOpen,
    /// All questions from this section can be reveiwed and edited
    SectionReview,
    /// Answers for the current section are locked and cannot be edited
    SectionScore,
    /// Terminal game state. The game is concluded and the winners are being
    /// announced.
    FinalWinners,
}
