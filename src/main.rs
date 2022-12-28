use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use async_lock::Mutex;
use lazy_static::lazy_static;
use systems::cache;

mod handlers;
mod middlewares;
mod systems;

lazy_static! {
    static ref CACHE_TTL: u64 = std::env::var("CACHE_TTL").unwrap().parse::<u64>().unwrap();
    static ref RESTRICTED_PUBKEYS: Vec<String> = std::env::var("RESTRICTED_PUBKEYS")
        .unwrap()
        .split(',')
        .map(|s| s.to_string())
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
            .route("/", web::get().to(handlers::index::get))
            .wrap(crate::middlewares::validate::Validate)
            .route("/nip05", web::get().to(handlers::nip05::get))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
