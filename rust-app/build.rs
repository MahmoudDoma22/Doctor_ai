use image::{ImageBuffer,Rgba};
use std::path::PathBuf;
fn main(){
 let mut icon:ImageBuffer<Rgba<u8>,Vec<u8>>=ImageBuffer::new(256,256);
 for(x,y,p)in icon.enumerate_pixels_mut(){let dx=x as f32-128.0;let dy=y as f32-128.0;let r=(dx*dx+dy*dy).sqrt();let inside=r<118.0;let cross=(x>110&&x<146&&y>54&&y<202)||(x>54&&x<202&&y>110&&y<146);*p=Rgba(if cross{[56,211,190,255]}else if inside{[12,39,62,255]}else{[0,0,0,0]});}
 let out=PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("doctor-ai.ico");
 icon.save_with_format(&out,image::ImageFormat::Ico).expect("write Doctor AI icon");
 #[cfg(target_os="windows")] {let mut res=winres::WindowsResource::new();res.set_icon(out.to_str().unwrap());res.compile().expect("embed app icon");}
}
