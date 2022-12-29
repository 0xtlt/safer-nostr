use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::WebStates;

#[derive(Deserialize)]
pub struct Info {
    nip05: String,
}

pub async fn get(info: web::Query<Info>, data: web::Data<WebStates>) -> impl Responder {
    println!("NIP05: {}", info.nip05);

    let cache_key = format!("nip05:{}", info.nip05);

    let cache_response = data.cache.to_owned().get_str(&cache_key).await;

    if let Ok(cache_response) = cache_response {
        return HttpResponse::Ok().body(cache_response);
    }

    let nip05_response = nostr_rust::nips::nip5::get_nip05(&info.nip05).await;

    let body_response = match nip05_response {
        Ok(response) => {
            json!({
                "status": "success",
                "data": response,
                "updated_at": chrono::Utc::now().timestamp()
            })
        }
        Err(err) => {
            json!({
                "status": "error",
                "message": err.to_string()
            })
        }
    };

    data.cache
        .to_owned()
        .set_str(
            &cache_key,
            &body_response.to_string(),
            crate::ENV_CONFIG.cache_ttl_nip05.to_owned(),
        )
        .await
        .unwrap();

    HttpResponse::Ok().body(body_response.to_string())
}
