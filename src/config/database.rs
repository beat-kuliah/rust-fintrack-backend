use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}, ConnectOptions};
use std::{env, time::Duration, str::FromStr};
use tracing;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Optimized pool configuration for low-resource environment (2CPU 2GB RAM)
    let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8); // Drastically reduced for 2GB RAM
    let min_connections: u32 = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2); // Minimal connections to reduce memory usage
    let acquire_timeout = Duration::from_secs(
        env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5), // Faster timeout for quick failure
    );
    let idle_timeout = Duration::from_secs(
        env::var("DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30), // Shorter idle timeout to free connections
    );
    let max_lifetime = Duration::from_secs(
        env::var("DB_MAX_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1800), // 30 minutes max lifetime
    );

    // Parse connection options to enable statement caching
    let mut connect_options = PgConnectOptions::from_str(&database_url)?;
    
    // Enable statement caching for better performance
    connect_options = connect_options
        .statement_cache_capacity(64) // Further reduced cache size for memory efficiency
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