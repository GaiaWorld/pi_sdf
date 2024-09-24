use image::ColorType;
use parry2d::bounding_volume::Aabb;
use pi_sdf::{blur::{blur_box, blur_box2, compute_box_layout}, Point};
// 1 -> 5
// 2 -> 10
fn main(){
    // let info = blur_box(&[0.0,0.0, 32.0, 32.0], 5.0, 32);

    // // image::save_buffer(path, buf, width, height, color)
    // let _ = image::save_buffer("blur.png", &info.tex, info.width as u32, info.height as u32, ColorType::L8);

    let info  = compute_box_layout(pi_sdf::glyphy::geometry::aabb::Aabb(Aabb::new(Point::new(0.0,0.0,), Point::new( 32.0,64.0))), 32, 5);
    let r = blur_box2(info.clone());

    let _ = image::save_buffer("blur2.png", &r, info.p_w as u32, info.p_h as u32, ColorType::L8).unwrap();


    // let info  = compute_box_layout(pi_sdf::glyphy::geometry::aabb::Aabb(Aabb::new(Point::new(-5.0,-5.0,), Point::new( 15.0,15.0))), 32, 5);
    // let r = blur_box2(info.clone());

    // let _ = image::save_buffer("blur3.png", &r, info.p_w as u32, info.p_h as u32, ColorType::L8).unwrap();
}