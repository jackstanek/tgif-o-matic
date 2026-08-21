//! User management, authentication, and sessions. There are two types of users:
//! admins and players. Admins can create templates and game instances, and then
//! run the games. Players can participate, answer questions, etc. Admin
//! accounts are persistent; if one doesn't exist on server launch, one is
//! created. Player "accounts" are ephemeral and are tied to specific game
//! instances.

use sqlx::Row;

use crate::db::{decode_timestamp, game::GameId};

db_id_type!(AdminId, PlayerId);

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct AdminRow {
    pub(crate) id: AdminId,
    pub(crate) username: String,
    pub(crate) pw_hash: String,
}

impl AdminRow {
    /// Get a single [`AdminRow`] by username.
    pub(crate) async fn by_username<'e, E>(exec: E, username: &str) -> sqlx::Result<Option<Self>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        sqlx::query_as::<_, AdminRow>(
            r#"
            SELECT id, username, pw_hash FROM admins WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(exec)
        .await
    }
}

/// Initialize an admin account if one does not exist. If an admin account
/// exists, this is a no-op. Returns true if the account was created, false
/// otherwise.
pub(crate) async fn init_admin_account<'e, E>(
    exec: E,
    username: &str,
    phc_string: &str,
) -> sqlx::Result<bool>
where
    E: sqlx::Acquire<'e, Database = sqlx::Sqlite>,
{
    // Transaction to insert the admin account if one does not exist
    let mut conn = exec.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let result = sqlx::query(
        r#"
            INSERT INTO admins (username, pw_hash)
            SELECT ?, ?
            WHERE NOT EXISTS (SELECT 1 FROM admins)
        "#,
    )
    .bind(username)
    .bind(phc_string)
    .execute(&mut *conn)
    .await?;

    sqlx::query("COMMIT").execute(&mut *conn).await?;

    Ok(result.rows_affected() > 0)
}

/// Update the admin password for the given user. No-op if the user does not
/// exist; returns Ok(true) if a user was updated and Ok(false) otherwise.
pub(crate) async fn update_admin_password<'e, E>(
    exec: E,
    username: &str,
    phc_string: &str,
) -> sqlx::Result<bool>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query!(
        r#"
            UPDATE admins
            SET pw_hash = ?
            WHERE username = ?
        "#,
        phc_string,
        username,
    )
    .execute(exec)
    .await
    .map(|res| res.rows_affected() > 0)
}

/// Session record in the database.
#[derive(Debug, Clone)]
pub(crate) struct SessionRow {
    pub(crate) token_hash: Vec<u8>,
    pub(crate) game_id: Option<GameId>,
    pub(crate) admin_id: Option<AdminId>,
    pub(crate) player_id: Option<PlayerId>,
    pub(crate) created_at: jiff::Timestamp,
    pub(crate) expires_at: jiff::Timestamp,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for SessionRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            token_hash: row.try_get("token_hash")?,
            game_id: row.try_get("game_id")?,
            admin_id: row.try_get("admin_id")?,
            player_id: row.try_get("player_id")?,
            created_at: row.try_get("created_at").and_then(decode_timestamp)?,
            expires_at: row.try_get("expires_at").and_then(decode_timestamp)?,
        })
    }
}

/// Create a user session for the given admin.
pub(crate) async fn create_admin_session<'e, E>(
    exec: E,
    token_hash: &[u8],
    admin_id: AdminId,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query!(
        r#"
            INSERT INTO sessions (token_hash, admin_id)
            VALUES ($1, $2)
        "#,
        token_hash,
        admin_id,
    )
    .execute(exec)
    .await?;
    Ok(())
}

/// Get a session by the token hash.
pub(crate) async fn get_session_by_token_hash<'e, E>(
    exec: E,
    token_hash: &[u8],
) -> sqlx::Result<Option<SessionRow>>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query_as::<_, SessionRow>(
        r#"
            SELECT * FROM sessions
            WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(exec)
    .await
}

/// Clean up old sessions. Deletes all sessions for which the `expires_at`
/// timestamp is earlier than the current time. This function should run
/// periodically to keep the session table reasonably sized.
pub(crate) async fn cleanup_old_sessions<'e, E>(exec: E) -> sqlx::Result<u64>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query!(
        r#"
            DELETE FROM sessions
            WHERE expires_at < unixepoch('now')
        "#
    )
    .execute(exec)
    .await
    .map(|res| {
        let rows = res.rows_affected();
        if rows > 0 {
            tracing::info!("cleaned up {rows} sessions");
        }
        rows
    })
}
