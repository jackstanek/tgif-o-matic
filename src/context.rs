//! Shim trait to trace errors logged with [`anyhow::Context`].

use std::fmt::Display;

use tracing::error;

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
