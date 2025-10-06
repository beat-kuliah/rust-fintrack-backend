use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}, ConnectOptions};
use std::{env, time::Duration, str::FromStr};
use tracing;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20); // Reduced default for resource-constrained systems
    let min_connections: u32 = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5); // Conservative minimum
    let acquire_timeout = Duration::from_secs(
        env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10), // Reasonable timeout
    );
    let idle_timeout = Duration::from_secs(
        env::var("DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60), // Keep connections alive longer
    );
    let max_lifetime = Duration::from_secs(
        env::var("DB_MAX_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600), // 1 hour lifetime
    );

    // Parse connection options to enable statement caching
    let mut connect_options = PgConnectOptions::from_str(&database_url)?;
    
    // Enable statement caching for better performance
    connect_options = connect_options
        .statement_cache_capacity(128) // Reduced cache size for memory efficiency
        .disable_statement_logging(); // Reduce logging overhead

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(acquire_timeout)
        .idle_timeout(Some(idle_timeout))
        .max_lifetime(Some(max_lifetime))
        .test_before_acquire(true) // Enable connection testing to catch stale connections
        .connect_with(connect_options)
        .await?;

    // Log pool configuration for debugging
    tracing::info!(
        "Database pool created with max_connections={}, min_connections={}, acquire_timeout={}s",
        max_connections, min_connections, acquire_timeout.as_secs()
    );

    Ok(pool)
}