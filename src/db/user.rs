//! User management and authentication. User management is very simple and
//! mostly boils down to handling permissions in-game.

use anyhow::{Context, anyhow};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use log::info;

use crate::random_str::generate_legible_string;

db_id_type!(AdminId, PlayerId);

#[derive(sqlx::FromRow)]
#[sqlx(rename_all = "snake_case")]
pub(crate) struct Admin {
    id: AdminId,
    username: String,
    pw_hash: String,
}

/// Default administrator username
const DEFAULT_ADMIN_USERNAME: &str = "admin";
const RANDOM_ADMIN_PW_LENGTH: usize = 12;

/// Initialize an admin account if one does not exist. If at least one admin
/// account exists, then this is a no-op. Otherwise, if no admin accounts
/// exist, initialize one with the username and password supplied in the
/// configuration. If no credentials are in the configuration, then use
/// username "admin" and a random password.
pub(crate) async fn init_admin_account<R>(
    config: &crate::config::BackendConfig,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    rng: &mut R,
) -> anyhow::Result<()>
where
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
    let admin_hash = Argon2::default()
        .hash_password(admin_password.as_bytes(), &admin_salt)
        .map_err(|e| anyhow!("couldn't hash password: {e}"))?;

    // Transaction to insert the admin account if one does not exist
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let result = sqlx::query(
        r#"
            INSERT INTO admins (username, pw_hash)
            SELECT ?, ?
            WHERE NOT EXISTS (SELECT 1 FROM admins)
            "#,
    )
    .bind(admin_username)
    .bind(admin_hash.to_string())
    .execute(&mut *conn)
    .await
    .context("couldn't fetch admin accounts from database")?;

    sqlx::query("COMMIT").execute(&mut *conn).await?;

    if result.rows_affected() > 0 {
        info!("admin account created: admin/{admin_password}");
    }
    Ok(())
}
