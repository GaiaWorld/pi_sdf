use image::ColorType;
use parry2d::bounding_volume::Aabb;
use pi_sdf::{blur::blur_box, Point};
// 1 -> 5
// 2 -> 10
fn main(){
    let (pixmap, w, h, atlas_bounds) = blur_box(Aabb::new(Point::new(-5.0,-5.0), Point::new(5.0,5.0)), 1.5, 24);

    // image::save_buffer(path, buf, width, height, color)
    let _ = image::save_buffer("blur.png", &pixmap, w, h, ColorType::L8);

}