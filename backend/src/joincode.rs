//! Join codes for games and teams.
//!
//! Each player uses a join code to enter a game, and each player forming a team
//! distributes a join code to the players on their team to join their team.

use std::fmt::Display;

use rustrict::CensorStr;

#[derive(Debug, Clone, Copy)]
pub struct JoinCode<const N: usize>([u8; N]);
impl<const N: usize> JoinCode<N> {
    /// Create a [`JoinCode`] from a u8 buffer. If the buf is the wrong length
    /// or contains an inappropriate word (per [`CensorStr`]), the buffer is
    /// rejected and `None` is returned.
    fn from_buf(buf: &[u8]) -> Option<Self> {
        str::from_utf8(buf)
            .ok()
            .filter(|&s| !s.is_inappropriate())
            .and_then(|_| buf.as_array().map(|a| Self(*a)))
    }

    /// Create a [`JoinCode`] from an [`rand::RngExt`]
    pub fn from_rng(rng: &mut impl rand::Rng) -> Self {
        let mut buf = [0; N];
        loop {
            for ptr in buf.iter_mut() {
                *ptr = rng.sample(rand::distributions::Alphanumeric);
            }
            let code = Self::from_buf(&buf);
            if let Some(code) = code {
                return code;
            }
        }
    }
}

impl<const N: usize> Display for JoinCode<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    #[test]
    fn test_profane_join_code() {
        let code = JoinCode::<4>::from_buf("shit".as_bytes());
        assert_matches!(code, None)
    }
}
