use image::{AnimationDecoder, ImageFormat};
use serde::Deserialize;
use std::{
    fs::File,
    io::{Cursor, Read},
    mem,
};
use thiserror::Error;

use crate::systems::cache::{get_media_cache, set_media_cache};

use super::cache::Cache;

#[derive(Debug, Deserialize, Error)]
pub enum InfoError {
    #[error("Invalid ratio format")]
    InvalidRatioFormat,

    #[error("No width, height, or ratio defined")]
    NoSizeDefined,
}

#[derive(Deserialize)]
pub struct Info {
    pub url: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub ratio: Option<String>, // Format: "width:height"
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

#[derive(Debug, Error)]
pub enum ImageCacheError {
    #[error("Width is too large")]
    WidthTooLarge,

    #[error("Height is too large")]
    HeightTooLarge,

    #[error("Width or height is too large after ratio applied")]
    SizeTooLargeAfterRatio,
}

pub async fn cache_image(
    params: &Info,
    cache: &Cache,
) -> Result<(Vec<u8>, String), ImageCacheError> {
    let file_name = &format!(
        "{}-{}-{}-{}",
        params.url,
        params.ratio.clone().unwrap_or(String::new()),
        params.width.unwrap_or(0.0),
        params.height.unwrap_or(0.0)
    );
    let mut image_cache = get_media_cache(file_name, cache).await;

    // Check if the body is an image

    if image_cache.is_none() {
        // Fetch the url
        let client = reqwest::Client::new();
        let response = client.get(&params.url).send().await.unwrap();
        let body_response = response.bytes().await.unwrap();

        // First size check
        if params.width.is_some()
            && params.width.unwrap() > crate::ENV_CONFIG.image_max_width as f64
        {
            return Err(ImageCacheError::WidthTooLarge);
        }

        if params.height.is_some()
            && params.height.unwrap() > crate::ENV_CONFIG.image_max_height as f64
        {
            return Err(ImageCacheError::HeightTooLarge);
        }

        let image = image::load_from_memory(&body_response).unwrap();

        let (new_width, new_height) = params
            .get_new_size(image.width() as f64, image.height() as f64)
            .unwrap();

        // Second size check
        if new_width > crate::ENV_CONFIG.image_max_width as u32
            || new_height > crate::ENV_CONFIG.image_max_height as u32
        {
            return Err(ImageCacheError::SizeTooLargeAfterRatio);
        }

        // Determine the image format
        let type_image = image::guess_format(&body_response).unwrap();

        match type_image {
            ImageFormat::Png => {
                let new_png = crate::systems::images::png::run(&image, new_width, new_height);

                set_media_cache(file_name, &new_png, cache).await;

                cache
                    .set_str(
                        &format!("{file_name}+ext"),
                        "image/png",
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                cache
                    .set_str(
                        file_name,
                        &chrono::Utc::now().timestamp().to_string(),
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                image_cache = get_media_cache(file_name, cache).await;

                return Ok(image_cache.unwrap());
            }
            ImageFormat::Jpeg => {
                let new_jpeg = crate::systems::images::jpg::run(&image, new_width, new_height);

                set_media_cache(file_name, &new_jpeg, cache).await;

                cache
                    .set_str(
                        &format!("{file_name}+ext"),
                        "image/jpeg",
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                cache
                    .set_str(
                        file_name,
                        &chrono::Utc::now().timestamp().to_string(),
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                image_cache = get_media_cache(file_name, cache).await;

                return Ok(image_cache.unwrap());
            }
            ImageFormat::Gif => {
                let new_gif = crate::systems::images::gif::run(
                    &body_response.try_into().unwrap(),
                    new_width,
                    new_height,
                );

                set_media_cache(file_name, &new_gif, cache).await;

                cache
                    .set_str(
                        &format!("{file_name}+ext"),
                        "image/gif",
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                cache
                    .set_str(
                        file_name,
                        &chrono::Utc::now().timestamp().to_string(),
                        crate::ENV_CONFIG.cache_ttl_images,
                    )
                    .await
                    .unwrap();

                image_cache = get_media_cache(file_name, cache).await;

                return Ok(image_cache.unwrap());
            }
            _ => todo!("type_image: {:#?} not supported yet", type_image),
        }
    }

    Ok(image_cache.unwrap())
}
