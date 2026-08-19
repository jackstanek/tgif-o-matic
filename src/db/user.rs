//! User management, authentication, and sessions. There are two types of users:
//! admins and players. Admins can create templates and game instances, and then
//! run the games. Players can participate, answer questions, etc. Admin
//! accounts are persistent; if one doesn't exist on server launch, one is
//! created. Player "accounts" are ephemeral and are tied to specific game
//! instances.

use sqlx::Row;

use crate::{db::decode_timestamp, db::game::GameId};

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
/// exists, this is a no-op.
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

/// Session record in the database.
#[derive(Debug, Clone)]
pub(crate) struct Session {
    token_hash: Vec<u8>,
    game_id: GameId,
    admin_id: AdminId,
    player_id: PlayerId,
    created_at: jiff::Timestamp,
    expires_at: jiff::Timestamp,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Session {
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
    admin: &AdminRow,
) -> anyhow::Result<Session>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    todo!()
}

/// Clean up old sessions. Deletes all sessions for which the `expires_at`
/// timestamp is earlier than the current time. This function should run
/// periodically to keep the session table reasonably sized.
pub(crate) async fn cleanup_old_sessions<'e, E>(
    exec: E,
) -> anyhow::Result<sqlx::sqlite::SqliteQueryResult>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    Ok(sqlx::query!(
        r#"
            DELETE FROM sessions
            WHERE expires_at < unixepoch('now')
        "#
    )
    .execute(exec)
    .await?)
}
