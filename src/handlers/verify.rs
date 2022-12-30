use actix_web::{HttpResponse, Responder};
use serde_json::json;

pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "OK!", "code": 1 }))
}
