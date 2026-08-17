//! Shim trait to trace errors logged with [`anyhow::Context`].

use std::fmt::Display;

use tracing::error;

/// Wrapper type for [`argon2::Error`], which doesn't implement
/// [`std::error::Error`].
// TODO: This should be removed when upstream implements that trait properly.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Argon2Error {
    Base(argon2::Error),
    Hash(argon2::password_hash::Error),
}

impl Display for Argon2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Argon2Error::Base(e) => Display::fmt(e, f),
            Argon2Error::Hash(e) => Display::fmt(e, f),
        }
    }
}
impl std::error::Error for Argon2Error {}
impl From<argon2::Error> for Argon2Error {
    fn from(e: argon2::Error) -> Self {
        Argon2Error::Base(e)
    }
}
impl From<argon2::password_hash::Error> for Argon2Error {
    fn from(e: argon2::password_hash::Error) -> Self {
        Argon2Error::Hash(e)
    }
}

/// Wrapper trait for [`anyhow::Context`] which hooks in a [`tracing::Error`]
/// event.
pub(crate) trait TracingContext<T, E> {
    fn tracing_context<C>(self, context: C) -> Result<T, anyhow::Error>
    where
        C: Display + Send + Sync + 'static;
}

impl<T, E> TracingContext<T, E> for Result<T, E>
where
    E: Into<anyhow::Error> + Send + Sync + 'static,
{
    fn tracing_context<C>(self, context: C) -> Result<T, anyhow::Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            let e = err.into();
            error!(%e, %context);
            e.context(context)
        })
    }
}

impl<T, E> TracingContext<T, E> for Option<T> {
    fn tracing_context<C>(self, context: C) -> Result<T, anyhow::Error>
    where
        C: Display + Send + Sync + 'static,
    {
        error!(%context);
        anyhow::Context::context(self, context)
    }
}
