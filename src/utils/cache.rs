use redis::{Client, AsyncCommands};
use redis::aio::ConnectionManager;
use serde::{Serialize, Deserialize};
use tracing::{error, info, warn};
use crate::config::RedisConfig;

#[derive(Clone)]
pub struct CacheService {
    connection_manager: Option<ConnectionManager>,
    enabled: bool,
}

impl CacheService {
    pub async fn new(config: &RedisConfig) -> Self {
        if !config.enabled {
            info!("Redis cache is disabled");
            return Self {
                connection_manager: None,
                enabled: false,
            };
        }

        let redis_url = config.build_url();
        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                match client.get_connection_manager().await {
                    Ok(connection_manager) => {
                        info!("Redis connection established successfully");
                        Self {
                            connection_manager: Some(connection_manager),
                            enabled: true,
                        }
                    }
                    Err(e) => {
                        error!("Failed to create Redis connection manager: {}", e);
                        warn!("Running without Redis cache");
                        Self {
                            connection_manager: None,
                            enabled: false,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create Redis client: {}", e);
                warn!("Running without Redis cache");
                Self {
                    connection_manager: None,
                    enabled: false,
                }
            }
        }
    }

    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled || self.connection_manager.is_none() {
            return None;
        }

        let mut conn = self.connection_manager.as_ref()?.clone();
        
        match conn.get::<_, String>(key).await {
            Ok(value) => {
                match serde_json::from_str::<T>(&value) {
                    Ok(deserialized) => Some(deserialized),
                    Err(e) => {
                        error!("Failed to deserialize cached value for key '{}': {}", key, e);
                        None
                    }
                }
            }
            Err(e) => {
                if !e.to_string().contains("nil") {
                    error!("Failed to get value from cache for key '{}': {}", key, e);
                }
                None
            }
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> bool
    where
        T: Serialize,
    {
        if !self.enabled || self.connection_manager.is_none() {
            return false;
        }

        let mut conn = match self.connection_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => return false,
        };

        let serialized = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize value for key '{}': {}", key, e);
                return false;
            }
        };

        let result = if let Some(ttl) = ttl_seconds {
            conn.set_ex::<_, _, ()>(key, serialized, ttl).await
        } else {
            conn.set::<_, _, ()>(key, serialized).await
        };

        match result {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to set value in cache for key '{}': {}", key, e);
                false
            }
        }
    }

    pub async fn delete(&self, key: &str) -> bool {
        if !self.enabled || self.connection_manager.is_none() {
            return false;
        }

        let mut conn = match self.connection_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => return false,
        };

        match conn.del::<_, ()>(key).await {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to delete key '{}' from cache: {}", key, e);
                false
            }
        }
    }

    pub async fn exists(&self, key: &str) -> bool {
        if !self.enabled || self.connection_manager.is_none() {
            return false;
        }

        let mut conn = match self.connection_manager.as_ref() {
            Some(cm) => cm.clone(),
            None => return false,
        };

        match conn.exists::<_, bool>(key).await {
            Ok(exists) => exists,
            Err(e) => {
                error!("Failed to check existence of key '{}' in cache: {}", key, e);
                false
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// Helper functions for generating cache keys
pub fn user_cache_key(user_id: &uuid::Uuid) -> String {
    format!("user:{}", user_id)
}

pub fn user_pockets_cache_key(user_id: &uuid::Uuid) -> String {
    format!("user:{}:pockets", user_id)
}

pub fn jwt_cache_key(token_hash: &str) -> String {
    format!("jwt:{}", token_hash)
}