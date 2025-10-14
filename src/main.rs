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

    for iter in 0.. {
        // sleep(Duration::from_secs(1));

        // get a frame
        let frame = camera.frame().unwrap();
        println!("Captured Single Frame of {}", frame.buffer().len());

        // decode into an ImageBuffer
        let decoded = frame.decode_image::<RgbFormat>().unwrap();
        println!("Decoded Frame of {}", decoded.len());

        if iter % 10 == 5 {
            let (width, height) = decoded.dimensions();
            let img_rgb888 = decoded.clone().into_raw();
            // Encode as SIXEL data
            let sixel_data = icy_sixel::sixel_string(
                &img_rgb888,
                width as i32,
                height as i32,
                icy_sixel::PixelFormat::RGB888,
                icy_sixel::DiffusionMethod::Auto, // Auto, None, Atkinson, FS, JaJuNi, Stucki, Burkes, ADither, XDither
                icy_sixel::MethodForLargest::Auto, // Auto, Norm, Lum
                icy_sixel::MethodForRep::Auto,    // Auto, CenterBox, AverageColors, Pixels
                icy_sixel::Quality::HIGH,         // AUTO, HIGH, LOW, FULL, HIGHCOLOR
            )
            .expect("Failed to encode image to SIXEL format");
            println!("{sixel_data}");
        }

        let img = DynamicImage::ImageRgb8(decoded);

        {
            let rgba_img = img.to_rgba8();
            let rgba_img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = rgba_img;
            let (width, height) = rgba_img.dimensions();
            let buf = rgba_img.into_vec();
            let rgba_img =
                image_0_24::ImageBuffer::<image_0_24::Rgba<u8>, _>::from_vec(width, height, buf)
                    .unwrap();
            let decoder = bardecoder::default_decoder();
            let codes = decoder.decode(&rgba_img);
            for code in codes {
                match code {
                    Ok(code) => {
                        println!("bardecoder found code {code:?}");
                        return;
                    }
                    Err(err) => {
                        println!("bardecoder error {err:?}");
                    }
                }
            }
        }

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
