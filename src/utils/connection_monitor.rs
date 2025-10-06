use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};

pub struct ConnectionMonitor {
    pool: PgPool,
    check_interval: Duration,
}

impl ConnectionMonitor {
    pub fn new(pool: PgPool, check_interval_secs: u64) -> Self {
        Self {
            pool,
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    pub async fn start_monitoring(&self) {
        let mut interval = interval(self.check_interval);
        
        loop {
            interval.tick().await;
            self.check_pool_health().await;
        }
    }

    async fn check_pool_health(&self) {
        let pool_size = self.pool.size();
        let idle_connections = self.pool.num_idle();
        
        info!(
            "Connection Pool Status - Total: {}, Idle: {}, Active: {}",
            pool_size,
            idle_connections,
            pool_size.saturating_sub(idle_connections as u32)
        );

        // Check if pool is under stress
        if idle_connections == 0 && pool_size > 0 {
            warn!("Connection pool exhausted! All connections are in use.");
        }

        // Test connection health
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                info!("Database connection health check: OK");
            }
            Err(e) => {
                error!("Database connection health check failed: {}", e);
            }
        }
    }
}

pub async fn start_connection_monitoring(pool: PgPool) {
    let monitor = ConnectionMonitor::new(pool, 30); // Check every 30 seconds
    
    tokio::spawn(async move {
        monitor.start_monitoring().await;
    });
}