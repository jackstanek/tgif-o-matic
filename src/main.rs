use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand_chacha::rand_core::SeedableRng;
use sqlx::Sqlite;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::BackendConfig, domain::auth::Argon2Error, random_str::generate_legible_string,
};

mod config;
mod db;
mod domain;
mod random_str;
mod routes;

/// Default administrator username
const DEFAULT_ADMIN_USERNAME: &str = "admin";

/// Default length of the admin password, if one should be generated
const RANDOM_ADMIN_PW_LENGTH: usize = 12;

/// Initialize an admin account if one does not exist. If at least one admin
/// account exists, then this is a no-op. Otherwise, if no admin accounts exist,
/// initialize one with the username and password supplied in the configuration.
/// If no credentials are in the configuration, then use username "admin" and a
/// random password.
pub(crate) async fn init_admin_account<'e, E, R>(
    rng: &mut R,
    exec: E,
    config: &crate::config::BackendConfig,
) -> anyhow::Result<()>
where
    E: sqlx::Acquire<'e, Database = Sqlite>,
    R: rand::CryptoRng + rand::Rng,
{
    let (admin_username, admin_password) = if let Some(pw) = &config.admin_user_password {
        pw.to_owned()
    } else {
        (
            DEFAULT_ADMIN_USERNAME.to_string(),
            generate_legible_string(rng, RANDOM_ADMIN_PW_LENGTH),
        )
    };
    let admin_salt = SaltString::generate(rng);
    let phc_string = Argon2::default()
        .hash_password(admin_password.as_bytes(), &admin_salt)
        .map_err(Argon2Error::from)?;

    let res = db::init_admin_account(exec, &admin_username, &phc_string.to_string()).await?;
    if res {
        info!("admin account created: admin/{admin_password}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = BackendConfig::from_env();
    let db = db::init_db(&config)
        .await
        .expect("could not initialize database");

    let mut rng = rand_chacha::ChaCha12Rng::from_rng(&mut rand::rngs::OsRng)?;
    debug!("bootstrapped RNG");

    init_admin_account(&mut rng, &db, &config).await?;

    let app = routes::build_router(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
