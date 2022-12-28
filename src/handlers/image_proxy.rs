use std::io::Cursor;

use actix_web::{web, HttpResponse, Responder};
use image::ImageFormat;
use serde::Deserialize;

use crate::WebStates;

#[derive(Deserialize)]
pub struct Info {
    url: String,
    width: u32,
}

pub async fn get(info: web::Query<Info>, _data: web::Data<WebStates>) -> impl Responder {
    println!("URL: {}", info.url);
    println!("Width: {:?}", info.width);

    // Fetch the url
    let client = reqwest::Client::new();
    let response = client.get(&info.url).send().await.unwrap();
    let body_response = response.bytes().await.unwrap();

    // Check if the body is an image
    let image = image::load_from_memory(&body_response).unwrap();
    let image = image.resize(
        info.width,
        info.width,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert the image to a Vec of u8
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png).unwrap();

    // Return the image
    HttpResponse::Ok()
        .content_type("image/png")
        .body(cursor.into_inner())
}
