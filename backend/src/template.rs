//! Game templates contain sections and questions for a game instance.

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub(crate) struct TemplateId(usize);
