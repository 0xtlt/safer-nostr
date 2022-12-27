use actix_web::{web, App, HttpServer};

mod handlers;
mod systems;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    // let cache = cache::Cache::new(&std::env::var("REDIS_URL").unwrap())
    //     .await
    //     .unwrap();

    // let m: String = match cache.connection.lock().await.get("ok").await {
    //     Ok(Value) => {
    //         println!("Redis is connected");
    //         Value
    //     }
    //     Err(_) => {
    //         println!("Redis is not connected");
    //         "Redis is not connected".to_string()
    //     }
    // };

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(handlers::index::get))
            .route("/nip05", web::get().to(handlers::nip05::get))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
