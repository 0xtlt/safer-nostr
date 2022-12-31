use std::{
    fs::File,
    io::{Cursor, Read},
};

use image::AnimationDecoder;

pub fn run(gif_content: &Vec<u8>, new_width: u32, new_height: u32) -> Vec<u8> {
    let image = image::codecs::gif::GifDecoder::new(Cursor::new(gif_content)).unwrap();
    let frames = image.into_frames();
    let frames = frames.collect_frames().expect("error decoding gif");

    let gif_speed = frames[1].delay().numer_denom_ms();

    let mut new_frames = Vec::new();

    for frame in frames {
        // Step 1: Transform the frame into a Rgba8 dynamic image
        // Step 2: Resize the image
        // Step 3: Transform the image back into a gif frame

        let tmp_image = frame.into_buffer();
        let tmp_image = image::DynamicImage::ImageRgba8(tmp_image);
        let tmp_image =
            tmp_image.resize_to_fill(new_width, new_height, image::imageops::FilterType::Lanczos3);
        let tmp_image = tmp_image.into_rgba8();

        // Tmp image must be Frame type
        let tmp_image = image::Frame::new(tmp_image);

        new_frames.push(tmp_image);
    }

    let mut file_out = File::create("tmp/out.gif").unwrap();

    let mut encoder = image::codecs::gif::GifEncoder::new_with_speed(
        &mut file_out,
        (gif_speed.1).try_into().unwrap(),
    );

    encoder.encode_frames(new_frames).unwrap();

    // Reopen the file to get the bytes
    let mut file_out = File::open("tmp/out.gif").unwrap();
    let mut cursor_out_bytes = Vec::new();
    file_out.read_to_end(&mut cursor_out_bytes).unwrap();

    // Delete the file
    std::fs::remove_file("tmp/out.gif").unwrap();

    cursor_out_bytes
}
