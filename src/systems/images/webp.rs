use std::io::Cursor;

use image::{DynamicImage, ImageFormat};

pub fn run(image: &DynamicImage, width: u32, height: u32) -> Vec<u8> {
    let image = image.resize_to_fill(width, height, image::imageops::FilterType::Lanczos3);

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::WebP).unwrap();

    cursor.into_inner()
}
