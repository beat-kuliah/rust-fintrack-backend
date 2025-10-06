use std::env;

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub addr: String,
    pub password: Option<String>,
    pub db: u8,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub enabled: bool,
}

impl RedisConfig {
    pub fn from_env() -> Self {
        let addr = env::var("REDIS_ADDR").unwrap_or_else(|_| "localhost:6379".to_string());
        let password = env::var("REDIS_PASSWORD").ok().filter(|p| !p.is_empty());
        let db = env::var("REDIS_DB")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);
        let max_connections = env::var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);
        let connection_timeout = env::var("REDIS_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);
        let enabled = env::var("REDIS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            addr,
            password,
            db,
            max_connections,
            connection_timeout,
            enabled,
        }
    }

    pub fn build_url(&self) -> String {
        match &self.password {
            Some(password) => format!("redis://:{}@{}/{}", password, self.addr, self.db),
            None => format!("redis://{}/{}", self.addr, self.db),
        }
    }
}