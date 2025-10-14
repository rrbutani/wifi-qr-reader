use std::{thread::sleep, time::Duration};

use image::DynamicImage;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use rxing::ImmutableReader;

fn main() {
    // first camera in system
    let index = CameraIndex::Index(0);
    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    // make the camera
    let mut camera = Camera::new(index, requested).unwrap();
    camera.open_stream().unwrap();

    loop {
        // sleep(Duration::from_secs(1));

        // get a frame
        let frame = camera.frame().unwrap();
        println!("Captured Single Frame of {}", frame.buffer().len());

        // decode into an ImageBuffer
        let decoded = frame.decode_image::<RgbFormat>().unwrap();
        println!("Decoded Frame of {}", decoded.len());
        let img = DynamicImage::ImageRgb8(decoded);

        let reader = rxing::qrcode::QRCodeReader::default();
        if let Ok(re) = reader.immutable_decode(&mut rxing::BinaryBitmap::new(
            rxing::common::HybridBinarizer::new(rxing::BufferedImageLuminanceSource::new(
                img.clone(),
            )),
        )) {
            dbg!(re.getText());
            return;
        }
        // rxing::helpers::detect_in_luma(luma, width, height, Some(rxing::BarcodeFormat::QR_CODE));

        let img = img.to_luma8();
        let mut img = rqrr::PreparedImage::prepare(img);
        // Search for grids, without decoding
        let grids = img.detect_grids();

        if let [grid, ..] = &grids[..] {
            let Ok((meta, content)) = grid.decode() else {
                println!("Couldn't decode qr code at {:?}", grid.bounds);
                continue;
            };
            dbg!(meta, content);
            return;
        }
    }
}
