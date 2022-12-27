use async_lock::Mutex;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

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
}
