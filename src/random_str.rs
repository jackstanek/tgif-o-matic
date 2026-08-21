//! Join codes for games and teams.
//!
//! Each player uses a join code to enter a game, and each player forming a team
//! distributes a join code to the players on their team to join their team.

use std::fmt::Display;

use rustrict::CensorStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinCode(String);

const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

/// Generate a random string of the given length, excluding ambiguous character
/// pairs (O/0, l/I/1)
pub(crate) fn generate_legible_string<R>(rng: &mut R, len: usize) -> String
where
    R: rand::Rng + rand::CryptoRng,
{
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

impl JoinCode {
    /// Create a [`JoinCode`] from a u8 buffer. If the buf is the wrong length
    /// or contains an inappropriate word (per [`CensorStr`]), the buffer is
    /// rejected and `None` is returned.
    fn from_string<I>(inp: I) -> Option<Self>
    where
        I: Into<String>,
    {
        let i: String = inp.into();
        (!i.is_inappropriate()).then_some(Self(i))
    }

    /// Create a [`JoinCode`] from an [`rand::Rng`]. Rejection sample to select
    /// non-profane join codes.
    pub fn from_rng<R>(rng: &mut R, len: usize) -> Self
    where
        R: rand::Rng + rand::CryptoRng,
    {
        loop {
            let code = Self::from_string(generate_legible_string(rng, len));
            if let Some(code) = code {
                return code;
            }
        }
    }
}

impl Display for JoinCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    #[test]
    fn test_profane_join_code() {
        let code = JoinCode::from_string("shit");
        assert_matches!(code, None)
    }
}
