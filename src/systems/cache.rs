use async_lock::Mutex;
use redis::AsyncCommands;
use std::sync::Arc;
use thiserror::Error;

pub static DEFAULT_CACHE_TTL: usize = 300;
pub static SECURITY_SIG_CACHE_TTL: usize = 300;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

#[derive(Clone)]
pub struct Cache {
    pub client: redis::Client,
    pub connection: Arc<Mutex<redis::aio::Connection>>,
}

impl Cache {
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url)?;
        let connection = client.get_async_connection().await?;
        let connection = Arc::new(Mutex::new(connection));

        Ok(Self { client, connection })
    }

    pub async fn get_str(&mut self, key: &str) -> Result<String, CacheError> {
        let mut connection = self.connection.lock().await;
        let value: String = connection.get(key).await?;

        Ok(value)
    }

    pub async fn set_str(
        &mut self,
        key: &str,
        value: &str,
        expiration: usize,
    ) -> Result<(), CacheError> {
        println!("Set cache key: {key}");

        let mut connection = self.connection.lock().await;
        connection.set(key, value).await?;
        connection.expire(key, expiration).await?; // Set the expiration time to 300 seconds (5 minutes)

        Ok(())
    }
}
