use image::{ImageBuffer, Rgba, ImageEncoder};
use std::path::PathBuf;

fn main() {
    let mut icon: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);
    for (x, y, pixel) in icon.enumerate_pixels_mut() {
        let dx = x as f32 - 128.0;
        let dy = y as f32 - 128.0;
        let inside = (dx * dx + dy * dy).sqrt() < 118.0;
        let cross = (x > 110 && x < 146 && y > 54 && y < 202)
            || (x > 54 && x < 202 && y > 110 && y < 146);
        *pixel = Rgba(if cross { [56, 211, 190, 255] }
            else if inside { [12, 39, 62, 255] } else { [0, 0, 0, 0] });
    }

    // ICO directory containing a PNG-compressed 256x256 RGBA image.
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(icon.as_raw(), 256, 256, image::ExtendedColorType::Rgba8)
        .expect("encode icon PNG");
    let mut ico = Vec::with_capacity(22 + png.len());
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&[0, 0, 0, 0]); // 256x256, no palette
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(&png);

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("doctor-ai.ico");
    std::fs::write(&out, ico).expect("write Doctor AI ICO");
    #[cfg(target_os = "windows")]
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon(out.to_str().unwrap());
        resource.compile().expect("embed executable icon");
    }
}
