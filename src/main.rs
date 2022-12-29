use actix_web::{web, App, HttpServer};
use lazy_static::lazy_static;
use strum::EnumString;
use systems::cache;

mod handlers;
mod middlewares;
mod systems;

#[derive(Clone, EnumString)]
pub enum MediaCacheType {
    #[strum(ascii_case_insensitive)]
    Redis,
    #[strum(ascii_case_insensitive)]
    RAM,
    #[strum(ascii_case_insensitive)]
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

#[derive(EnumString)]
pub enum RestrictedImages {
    #[strum(ascii_case_insensitive)]
    NSFW,
}

#[derive(EnumString)]
pub enum DynamicCacheType {
    #[strum(ascii_case_insensitive)]
    REDIS,
    #[strum(ascii_case_insensitive)]
    RAM,
}

pub struct EnvConfig {
    // DYNAMIC_CACHE_TYPE = "redis" | "ram"
    pub dynamic_cache_type: DynamicCacheType,
    pub redis_url: Option<String>,
    // RAM_LIMIT_OBJECTS
    pub ram_limit_objects: usize,
    // IMAGES_CACHE_TYPE
    pub images_cache_type: MediaCacheType,
    // IMAGE_MAX_WIDTH
    pub image_max_width: usize,
    // IMAGE_MAX_HEIGHT
    pub image_max_height: usize,
    // RESTRICTED_PUBKEYS
    pub restricted_pubkeys: Vec<String>,
    // RESTRICTED_IMAGES
    pub restricted_images: Vec<RestrictedImages>,
    // CACHE_TTL_NIP05
    pub cache_ttl_nip05: usize,
    // CACHE_TTL_IMAGES
    pub cache_ttl_images: usize,
    // CACHE_TTL_WEBPREVIEW
    pub cache_ttl_webpreview: usize,
}

lazy_static! {
    static ref ENV_CONFIG: EnvConfig = EnvConfig {
        dynamic_cache_type: std::env::var("DYNAMIC_CACHE_TYPE")
            .unwrap_or("redis".to_string())
            .parse()
            .expect("DYNAMIC_CACHE_TYPE must be 'redis' or 'ram'"),
        redis_url: std::env::var("REDIS_URL").ok(),
        ram_limit_objects: std::env::var("RAM_LIMIT_OBJECTS")
            .unwrap_or("100000".to_string())
            .parse()
            .expect("RAM_LIMIT_OBJECTS must be a number"),
        images_cache_type: std::env::var("IMAGES_CACHE_TYPE")
            .unwrap_or("redis".to_string())
            .parse()
            .expect("IMAGES_CACHE_TYPE must be 'redis' or 'ram' or 's3'"),
        image_max_width: std::env::var("IMAGE_MAX_WIDTH")
            .unwrap_or("2000".to_string())
            .parse()
            .expect("IMAGE_MAX_WIDTH must be a number"),
        image_max_height: std::env::var("IMAGE_MAX_HEIGHT")
            .unwrap_or("2000".to_string())
            .parse()
            .expect("IMAGE_MAX_HEIGHT must be a number"),
        restricted_pubkeys: std::env::var("RESTRICTED_PUBKEYS")
            .unwrap_or(String::new())
            .split(',')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        restricted_images: std::env::var("RESTRICTED_IMAGES")
            .unwrap_or(String::new())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("RESTRICTED_IMAGES must be 'nsfw'"))
            .collect(),
        cache_ttl_nip05: std::env::var("CACHE_TTL_NIP05")
            .unwrap_or("3600".to_string())
            .parse()
            .expect("CACHE_TTL_NIP05 must be a number"),
        cache_ttl_images: std::env::var("CACHE_TTL_IMAGES")
            .unwrap_or("3600".to_string())
            .parse()
            .expect("CACHE_TTL_IMAGES must be a number"),
        cache_ttl_webpreview: std::env::var("CACHE_TTL_WEBPREVIEW")
            .unwrap_or("3600".to_string())
            .parse()
            .expect("CACHE_TTL_WEBPREVIEW must be a number"),
    };
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
            .route(
                "/website_preview",
                web::get().to(handlers::website_previews::get),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
