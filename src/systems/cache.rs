use async_lock::Mutex;
use redis::AsyncCommands;
use std::{io::Write, sync::Arc};
use thiserror::Error;

use crate::MEDIA_CACHE;

pub static DEFAULT_CACHE_TTL: usize = 300;
pub static MEDIA_CACHE_TTL: usize = 3600;
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

    pub async fn get_str(&self, key: &str) -> Result<String, CacheError> {
        let mut connection = self.connection.lock().await;
        let value: String = connection.get(key).await?;

        Ok(value)
    }

    pub async fn get_bytes(&self, key: &str) -> Result<Vec<u8>, CacheError> {
        let mut connection = self.connection.lock().await;
        let value: Vec<u8> = connection.get(key).await?;

        Ok(value)
    }

    pub async fn set_str(
        &self,
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

    pub async fn set_bytes(
        &self,
        key: &str,
        value: Vec<u8>,
        expiration: usize,
    ) -> Result<(), CacheError> {
        println!("Set cache key: {key}");

        let mut connection = self.connection.lock().await;
        connection.set(key, value).await?;
        connection.expire(key, expiration).await?; // Set the expiration time to 300 seconds (5 minutes)

        Ok(())
    }
}

// Replace all specials chars which are not compatible with file name with "_" but keep "."
// https://hello.world -> https___hello.world
pub fn normalize_file_name(url: &str) -> String {
    url.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "_")
}

pub async fn set_media_cache(file_name: &str, content: Vec<u8>, cache: &Cache) {
    use crate::MediaCacheType::*;
    match MEDIA_CACHE.to_owned() {
        Redis => {
            let cache_key = format!("media_media:{}", file_name);

            cache
                .set_bytes(&cache_key, content, MEDIA_CACHE_TTL)
                .await
                .unwrap();
        }
        DiskDir(folder_path) => {
            let mut file_path = std::path::PathBuf::from(folder_path);
            file_path.push(normalize_file_name(file_name));

            let mut file = std::fs::File::create(file_path).unwrap();
            file.write_all(&content).unwrap();
        }
        S3 {
            bucket,
            region,
            access_key,
            secret_key,
        } => todo!(),
    }
}

pub async fn get_media_cache(file_name: &str, cache: &Cache) -> Option<(Vec<u8>, String)> {
    use crate::MediaCacheType::*;
    match MEDIA_CACHE.to_owned() {
        Redis => {
            let cache_key = format!("media_media:{file_name}");

            let cache_response = cache.get_bytes(&cache_key).await;
            if let Ok(cache_response) = cache_response {
                let mime_type = mime_guess::from_path(file_name)
                    .first_or_octet_stream()
                    .to_string();

                if cache_response.is_empty() {
                    return None;
                }

                Some((cache_response, mime_type))
            } else {
                None
            }
        }
        DiskDir(folder_path) => {
            let mut file_path = std::path::PathBuf::from(folder_path);
            file_path.push(normalize_file_name(file_name));
            if file_path.exists() {
                let content = std::fs::read(&file_path).unwrap();
                let mime_type = mime_guess::from_path(file_path)
                    .first_or_octet_stream()
                    .to_string();

                Some((content, mime_type))
            } else {
                None
            }
        }
        S3 {
            bucket,
            region,
            access_key,
            secret_key,
        } => todo!(),
    }
}
