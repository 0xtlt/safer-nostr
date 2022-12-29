use actix_web::{web, HttpResponse, Responder};
use image::ImageFormat;
use serde::Deserialize;
use std::io::Cursor;
use thiserror::Error;

use crate::{
    systems::{
        cache::{get_media_cache, set_media_cache},
        image_cache::{self, Info},
    },
    WebStates, IMAGE_CONFIG,
};

#[derive(Debug, Deserialize, Error)]
pub enum InfoError {
    #[error("Invalid ratio format")]
    InvalidRatioFormat,

    #[error("No width, height, or ratio defined")]
    NoSizeDefined,
}

pub async fn get(info: web::Query<Info>, data: web::Data<WebStates>) -> impl Responder {
    let (cache_content, cache_mime_type) = image_cache::cache_image(&info, &data.cache.to_owned())
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type(cache_mime_type)
        .body(cache_content)
}
