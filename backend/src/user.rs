//! User management and authentication. User management is very simple and
//! mostly boils down to handling permissions in-game.

use anyhow::{Context, anyhow};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{Salt, SaltString},
};
use log::{debug, info};
use rand::distributions::{Alphanumeric, DistString};

/// Opaque user ID for administrators.
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub(crate) struct AdminId(i32);

/// Opaque user ID for players.
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(transparent)]
pub(crate) struct PlayerId(i32);

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
const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

fn generate_password<R>(rng: &mut R, len: usize) -> String
where
    R: rand::CryptoRng + rand::Rng,
{
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

impl Admin {
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
                generate_password(rng, RANDOM_ADMIN_PW_LENGTH),
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
        .bind(&admin_hash.to_string())
        .execute(&mut *conn)
        .await
        .context("couldn't fetch admin accounts from database")?;

        sqlx::query("COMMIT").execute(&mut *conn).await?;

        if result.rows_affected() > 0 {
            info!("admin account created: admin/{admin_password}");
        }
        Ok(())
    }
}
