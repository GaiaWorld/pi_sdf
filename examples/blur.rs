use image::ColorType;
use parry2d::bounding_volume::Aabb;
use pi_sdf::{
    blur::blur_box,
    shape::{compute_shape_sdf_tex, computer_svg_sdf, Path},
    Point,
};
// 1 -> 5
// 2 -> 10
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let path = Path::new(
        vec![1, 17],
        vec![110., 215., 30., 50., 0., 1., 162.55, 162.45],
    );
    let info = path.get_svg_info();
    let sdf = compute_shape_sdf_tex(info, 32, 10, false);
    let _ = image::save_buffer(
        "svg.png",
        &sdf.sdf_tex,
        sdf.tex_size as u32,
        sdf.tex_size as u32,
        ColorType::L8,
    );

}