use actix_web::{web, HttpResponse, Responder};
use image::ImageFormat;
use serde::Deserialize;
use std::io::Cursor;
use thiserror::Error;

use crate::{
    systems::cache::{get_media_cache, set_media_cache},
    WebStates, IMAGE_CONFIG,
};

#[derive(Debug, Deserialize, Error)]
pub enum InfoError {
    #[error("Invalid ratio format")]
    InvalidRatioFormat,

    #[error("No width, height, or ratio defined")]
    NoSizeDefined,
}

#[derive(Deserialize)]
pub struct Info {
    url: String,
    width: Option<f64>,
    height: Option<f64>,
    ratio: Option<String>, // Format: "width:height"
}

impl Info {
    pub fn get_new_size(&self, width: f64, height: f64) -> Result<(u32, u32), InfoError> {
        if let Some(w) = self.width {
            if let Some(h) = self.height {
                // If both width and height are defined, check that the new size does not exceed the given width and height
                let new_width = if w > width { width } else { w };
                let new_height = if h > height { height } else { h };

                // Floor both
                let new_width = new_width.floor() as u32;
                let new_height = new_height.floor() as u32;

                Ok((new_width, new_height))
            } else if let Some(ratio) = &self.ratio {
                // If only width is defined and ratio is defined, calculate height using cross product
                let ratio_parts: Vec<&str> = ratio.split(':').collect();
                if ratio_parts.len() != 2 {
                    return Err(InfoError::InvalidRatioFormat);
                }
                let w_ratio: f64 = ratio_parts[0].parse().unwrap();
                let h_ratio: f64 = ratio_parts[1].parse().unwrap();
                let new_height = w / w_ratio * h_ratio;

                // Floor both
                let new_width = w.floor() as u32;
                let new_height = new_height.floor() as u32;

                Ok((new_width, new_height))
            } else {
                // If only width is defined, return the given width
                let new_width = w.floor() as u32;
                let new_height = (w / width * height).floor() as u32;

                Ok((new_width, new_height))
            }
        } else if let Some(h) = self.height {
            if let Some(ratio) = &self.ratio {
                // If only height is defined and ratio is defined, calculate width using cross product
                let ratio_parts: Vec<&str> = ratio.split(':').collect();
                if ratio_parts.len() != 2 {
                    return Err(InfoError::InvalidRatioFormat);
                }
                let w_ratio: f64 = ratio_parts[0].parse().unwrap();
                let h_ratio: f64 = ratio_parts[1].parse().unwrap();
                let new_width = h / h_ratio * w_ratio;

                // Floor both
                let new_width = new_width.floor() as u32;
                let new_height = h.floor() as u32;

                Ok((new_width, new_height))
            } else {
                // If only height is defined, return the given height
                let new_width = (h / height * width).floor() as u32;
                let new_height = h.floor() as u32;

                Ok((new_width, new_height))
            }
        } else if let Some(ratio) = &self.ratio {
            // If only ratio is defined, calculate width and height using cross product using the initial values of width and height
            let ratio_parts: Vec<&str> = ratio.split(':').collect();
            if ratio_parts.len() != 2 {
                return Err(InfoError::InvalidRatioFormat);
            }
            let w_ratio: f64 = ratio_parts[0].parse().unwrap();
            let h_ratio: f64 = ratio_parts[1].parse().unwrap();
            let new_width = (height / h_ratio * w_ratio) as u32;
            let new_height = (width / w_ratio * h_ratio) as u32;

            Ok((new_width, new_height))
        } else {
            Err(InfoError::NoSizeDefined)
        }
    }
}

pub async fn get(info: web::Query<Info>, data: web::Data<WebStates>) -> impl Responder {
    // Fetch the url
    let client = reqwest::Client::new();
    let response = client.get(&info.url).send().await.unwrap();
    let body_response = response.bytes().await.unwrap();

    // First size check
    if info.width.is_some() && info.width.unwrap() > IMAGE_CONFIG.max_width as f64 {
        return HttpResponse::BadRequest().body("Width is too large");
    }

    if info.height.is_some() && info.height.unwrap() > IMAGE_CONFIG.max_height as f64 {
        return HttpResponse::BadRequest().body("Height is too large");
    }

    // Check if the body is an image
    let file_name = &format!(
        "{}-{}-{}-{}.png",
        info.url,
        info.ratio.clone().unwrap_or(String::new()),
        info.width.unwrap_or(0.0),
        info.height.unwrap_or(0.0)
    );
    let mut image_cache = get_media_cache(file_name, &data.cache).await;

    if image_cache.is_none() {
        let image = image::load_from_memory(&body_response).unwrap();

        let (new_width, new_height) = info
            .get_new_size(image.width() as f64, image.height() as f64)
            .unwrap();

        // Second size check
        if new_width > IMAGE_CONFIG.max_width as u32 {
            return HttpResponse::BadRequest().body("Width is too large after ratio calculation");
        }

        if new_height > IMAGE_CONFIG.max_height as u32 {
            return HttpResponse::BadRequest().body("Height is too large after ratio calculation");
        }

        let image =
            image.resize_to_fill(new_width, new_height, image::imageops::FilterType::Nearest);

        let mut cursor = Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();

        set_media_cache(file_name, cursor.get_ref().to_vec(), &data.cache).await;

        data.cache
            .to_owned()
            .set_str(
                file_name,
                &chrono::Utc::now().timestamp().to_string(),
                crate::systems::cache::MEDIA_CACHE_TTL,
            )
            .await
            .unwrap();

        image_cache = get_media_cache(file_name, &data.cache).await;
    }

    let (cache_content, cache_mime_type) = image_cache.unwrap();

    HttpResponse::Ok()
        .content_type(cache_mime_type)
        .body(cache_content)

    // Return the image
    // HttpResponse::Ok()
    //     .content_type("image/png")
    //     .body(cursor.into_inner())
}
