use sqlx::{SqlitePool, sqlite::{SqlitePoolOptions, SqliteConnectOptions}};
use std::{num::NonZeroU32, sync::Arc, str::FromStr};
use governor::{Quota, RateLimiter};
use governor::state::{NotKeyed, InMemoryState};
use governor::clock::DefaultClock;

use crate::error::AppResult;

/// Shared application state passed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    /// Rate limiter for auth endpoints (10 req/min per process)
    pub auth_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl AppState {
    pub async fn new() -> AppResult<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://phylax.db".to_string());

        let opts = SqliteConnectOptions::from_str(&database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        // Auth rate limiter: 10 requests per minute
        let quota = Quota::per_minute(NonZeroU32::new(10).unwrap());
        let auth_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self { pool, auth_limiter })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
