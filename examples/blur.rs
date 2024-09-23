use image::ColorType;
use parry2d::bounding_volume::Aabb;
use pi_sdf::{blur::blur_box, Point};
// 1 -> 5
// 2 -> 10
fn main(){
    let info = blur_box(&[-5.0,-5.0, 15.0,10.0], 3.0, 24);

    // image::save_buffer(path, buf, width, height, color)
    let _ = image::save_buffer("blur.png", &info.tex, info.width as u32, info.height as u32, ColorType::L8);

}