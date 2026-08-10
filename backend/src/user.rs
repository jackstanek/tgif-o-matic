//! User management and authentication. User management is very simple and
//! mostly boils down to handling permissions in-game.

/// Opaque user ID.
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub(crate) struct UserId(usize);
