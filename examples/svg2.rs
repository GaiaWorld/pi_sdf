use image::ColorType;
use parry2d::bounding_volume::Aabb;
use pi_sdf::{
    blur::blur_box,
    shape::{compute_shape_sdf_tex, Path, PathVerb},
    Point,
};
// 1 -> 5
// 2 -> 10
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // M 50 40 L 190 40  A 10 10 0 0 1 200 50 L 200 140 A 10 10 0 0 1 190 150 L 50 150 A 10 10 0 0 1 40 140 L 40 50  A 10 10 0 0 1 50 40
    // let mut path = Path::new1(
    //     vec![
    //         PathVerb::MoveTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //     ],
    //     vec![
    //         50.0, 40.0, 190.0, 40.0, 10.0, 10.0, 0.0, 1.0, 200.0, 50.0, 200.0, 140.0, 10.0, 10.0,
    //         0.0, 1.0, 190.0, 150.0, 50.0, 150.0, 10.0, 10.0, 0.0, 1.0, 40.0, 140.0, 40.0, 50.0,
    //         10.0, 10.0, 0.0, 1.0, 50.0, 40.0,
    //     ],
    // );

    // // M 50 40 A 10 10 0 0 0 40 50  L 40 140 A 10 10 0 0 0 50 150 L 190 150 A 10 10 0 0 0 200 140  L 200 50   A 10 10 0 0 0 190 40 L 50 40
    // let mut path = Path::new1(
    //     vec![
    //         PathVerb::MoveTo,
    //         PathVerb::LineTo,
    //         PathVerb::LineTo,
    //         PathVerb::LineTo,
    //         PathVerb::LineTo,
    //     ],
    //     vec![
    //         50.0, 50.0, 100.0, 50.0, 100.0, 100.0, 50.0, 100.0, 50.0, 50.0,
    //     ],
    // );

    // // // M 0.0 25.33333 L 0.0 6.66666 A 6.6666666 6.666666 0 0 1 6.666666 0.0 L 25.3333 0.0  A 6.6666666 6.666666 0 0 1 32.0 6.66666 L 32.0 25.3333 A 6.666666 6.666666 0 0 1 25.333333 32 L 6.6666 32 A 6.66666 6.6666 0 0 1 0 25.3333
    // let mut path = Path::new1(
    //     vec![
    //         PathVerb::MoveTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //         PathVerb::LineTo,
    //         PathVerb::EllipticalArcTo,
    //     ],
    //     vec![
    //         0.0, 25.33333, 0.0, 6.66666, 6.6666666, 6.666666, 0.0, 1.0, 6.666666, 0.0, 25.3333,
    //         0.0, 6.6666666, 6.666666, 0.0, 1.0, 32.0, 6.66666, 32.0, 25.3333, 6.666666, 6.666666,
    //         0.0, 1.0, 25.333333, 32.0, 6.6666, 32.0, 6.66666, 6.6666, 0.0, 1.0, 0.0, 25.3333,
    //     ],
    // );

    let mut path = Path::new1(
        vec![
            PathVerb::MoveTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
            PathVerb::LineTo,
            PathVerb::EllipticalArcTo,
        ],
        vec![
            100., 100., 
            102., 100., 
            15.,  15., 0., 0., 117., 85., 
            117., 83., 
            15.,  15., 0., 0., 102., 68., 
            100., 68., 
            15.,  15., 0., 0., 85., 83., 
            85.,  85., 
            15.,  15., 0., 0., 100.,100.,
        ],
    );

    let info = path.get_svg_info();

    let sdf = compute_shape_sdf_tex(info, 32, 2, false, 3);
    println!("sdf.sdf_tex: {}", sdf.sdf_tex[38 * 3 + 3]);
    let _ = image::save_buffer(
        "Rounded_rectangle.png",
        &sdf.sdf_tex,
        sdf.tex_size as u32,
        sdf.tex_size as u32,
        ColorType::L8,
    );
}
