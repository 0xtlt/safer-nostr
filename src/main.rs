use actix_web::{web, App, HttpServer};
use lazy_static::lazy_static;
use systems::cache;

mod handlers;
mod middlewares;
mod systems;

#[derive(Clone)]
pub enum MediaCacheType {
    Redis,
    DiskDir(String),
    S3 {
        bucket: String,
        region: String,
        access_key: String,
        secret_key: String,
    },
}

pub struct ImageConfig {
    pub max_width: usize,
    pub max_height: usize,
}

lazy_static! {
    static ref IMAGE_CONFIG: ImageConfig = ImageConfig {
        max_width: std::env::var("IMAGE_MAX_WIDTH")
            .unwrap_or("2000".to_string())
            .parse()
            .unwrap(),
        max_height: std::env::var("IMAGE_MAX_HEIGHT")
            .unwrap_or("2000".to_string())
            .parse()
            .unwrap(),
    };
    static ref MEDIA_CACHE: MediaCacheType = {
        if std::env::var("REDIS_IMAGE").is_ok() {
            MediaCacheType::Redis
        } else if let Ok(dir) = std::env::var("DISK_IMAGE_DIR") {
            MediaCacheType::DiskDir(dir)
        } else {
            MediaCacheType::S3 {
                bucket: std::env::var("S3_BUCKET").unwrap(),
                region: std::env::var("S3_REGION").unwrap(),
                access_key: std::env::var("S3_ACCESS_KEY").unwrap(),
                secret_key: std::env::var("S3_SECRET_KEY").unwrap(),
            }
        }
    };
    static ref CACHE_TTL: usize = std::env::var("CACHE_TTL")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    static ref RESTRICTED_PUBKEYS: Vec<String> = std::env::var("RESTRICTED_PUBKEYS")
        .unwrap_or(String::new())
        .split(',')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
}

pub struct WebStates {
    pub cache: cache::Cache,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let cache = cache::Cache::new(&std::env::var("REDIS_URL").unwrap())
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WebStates {
                cache: cache.clone(),
            }))
            .wrap(crate::middlewares::time_mesure::TimeMesure)
            .route("/", web::get().to(handlers::index::get))
            .wrap(crate::middlewares::validate::Validate)
            .route("/nip05", web::get().to(handlers::nip05::get))
            .route("/image_proxy", web::get().to(handlers::image_proxy::get))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
