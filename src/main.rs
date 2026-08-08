use std::{path::Path, time::Duration};

use axum::{Router, routing::get};
use rand::rngs::Xoshiro256PlusPlus;

mod joincode;
mod state;

/// Initialize the SQLite conneciton pool for the main application database
///
// TODO: pass configuration options
async fn init_db(path: &Path) -> Result<sqlx::SqlitePool, sqlx::Error> {
    use sqlx::sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
    };
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello world!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
