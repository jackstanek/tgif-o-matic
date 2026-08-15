//! Top-level application configuration options.

use std::path::PathBuf;

/// Application configuration
pub(crate) struct BackendConfig {
    /// Username and password for an admin user
    pub admin_user_password: Option<(String, String)>,
    /// Path to the production SQLite database
    pub db_path: Option<PathBuf>,
    /// RNG seed
    pub rng_seed: Option<u64>,
}

impl BackendConfig {
    pub(crate) fn from_env() -> Self {
        let user = std::env::var("ADMIN_USER").ok();
        let pw = std::env::var("ADMIN_PASSWORD").ok();
        let db_path = std::env::var("DATABASE_URL")
            .ok()
            .map(|s| PathBuf::from(s.trim_start_matches("sqlite://")));
        let rng_seed = std::env::var("RNG_SEED")
            .ok()
            .and_then(|s| s.parse().ok());
        Self {
            admin_user_password: user.zip(pw),
            db_path,
            rng_seed,
        }
    }
}
