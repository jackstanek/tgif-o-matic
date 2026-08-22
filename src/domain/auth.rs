//! Authentication domain logic

use std::fmt::Display;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use rand_chacha::rand_core::CryptoRngCore;
use sha2::{
    Digest, Sha256,
    digest::{array::Array, consts::U32},
};

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

#[derive(Debug)]
pub(crate) struct Admin<T> {
    username: String,
    pw_hash: String,
    tag: T,
}

impl<T> Admin<T> {
    pub(crate) fn new(username: impl Into<String>, pw_hash: impl Into<String>, tag: T) -> Self {
        Self {
            username: username.into(),
            pw_hash: pw_hash.into(),
            tag,
        }
    }

    /// Verify the user's password against the salted hash.
    pub(crate) fn check_password(&self, password: &str) -> Result<bool, Argon2Error> {
        let pw_hash = PasswordHash::new(&self.pw_hash).map_err(Argon2Error::from)?;
        let check_result = Argon2::default()
            .verify_password(password.as_bytes(), &pw_hash)
            .is_ok();
        tracing::debug!("verifying password for {}: {check_result}", self.username);
        Ok(check_result)
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    /// Get the tag object of this admin
    pub(crate) fn tag(&self) -> &T {
        &self.tag
    }
}

// 128 bits
const SESSION_TOKEN_BYTES: usize = 16;

/// Session token
pub(crate) struct SessionToken {
    token: [u8; SESSION_TOKEN_BYTES],
}

impl SessionToken {
    /// Generate a new random session token
    pub(crate) fn generate<R>(rng: &mut R) -> Self
    where
        R: CryptoRngCore,
    {
        Self { token: rng.r#gen() }
    }

    /// Calculate a SHA-256 hash of the token
    pub(crate) fn sha256(&self) -> Array<u8, U32> {
        Sha256::digest(self.token)
    }

    /// Construct a [`SessionToken`] from a byte slice
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes.try_into().ok().map(|token| Self { token })
    }

    /// Construct a [`SessionToken`] from a URL-safe base64 string without
    /// padding. This decodes using the [`URL_SAFE_NO_PAD`] base64 engine.
    pub(crate) fn from_base64(base64: &str) -> Option<Self> {
        let token = URL_SAFE_NO_PAD.decode(base64).ok()?;
        Self::from_bytes(&token)
    }

    /// Encode a `SessionToken` as a base64 string.
    pub(crate) fn base64_encode(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.token)
    }
}
