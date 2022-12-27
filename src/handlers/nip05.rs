use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Info {
    nip05: String,
}

pub async fn get(info: web::Query<Info>) -> impl Responder {
    println!("NIP05: {}", info.nip05);
    let response = nostr_rust::nips::nip5::get_nip05(&info.nip05).await;

    HttpResponse::Ok().body(format!("Hello world! {:?}", response))
}
