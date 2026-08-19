//! Authentication domain logic

use std::fmt::Display;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use tracing::info;

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

pub(crate) struct Admin {
    username: String,
    pw_hash: String,
}

impl Admin {
    pub(crate) fn new(username: String, pw_hash: String) -> Self {
        Self { username, pw_hash }
    }

    /// Verify the user's password against the salted hash.
    pub(crate) fn check_password(&self, password: &str) -> Result<bool, Argon2Error> {
        let pw_hash = PasswordHash::new(&self.pw_hash).map_err(Argon2Error::from)?;
        let check_result = Argon2::default()
            .verify_password(password.as_bytes(), &pw_hash)
            .is_ok();
        info!("password check for {}: {check_result}", self.username);
        Ok(check_result)
    }
}
