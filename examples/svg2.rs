use image::ColorType;
use pi_sdf::{shape::{self, Path, PathVerb, SvgInfo}, utils::SdfInfo2};
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

    // let mut path = Path::new1(
    //     vec![
    //         PathVerb::MoveTo, PathVerb::EllipticalArcTo, PathVerb::LineTo, PathVerb::EllipticalArcTo, PathVerb::LineTo, PathVerb::EllipticalArcTo, PathVerb::LineTo, PathVerb::EllipticalArcTo, PathVerb::Close
    //         // PathVerb::MoveTo,
    //         // PathVerb::LineTo,
    //         // PathVerb::EllipticalArcTo,
    //         // PathVerb::LineTo,
    //         // PathVerb::EllipticalArcTo,
    //         // PathVerb::LineTo,
    //         // PathVerb::EllipticalArcTo,
    //         // PathVerb::LineTo,
    //         // PathVerb::EllipticalArcTo,
    //     ],
    //     vec![
    //         0.0, 10.0, 8.0, 8.0, 0.0, 0.0, 8.0, 18.0, 10.0, 18.0, 8.0, 8.0, 0.0, 0.0, 18.0, 10.0, 18.0, 8.0, 8.0, 8.0, 0.0, 0.0, 10.0, 0.0, 8.0, 0.0, 8.0, 8.0, 0.0, 0.0, 0.0, 8.0
    //     ],
    // );

    // let info = path.get_svg_info();

    // let point = [
    //     100.0, 100.0, f32::INFINITY,
    //     102.0, 100.0, 0.0,
    //     117.0, 85.0, -std::f32::consts::FRAC_PI_8.tan(),
    //     117.0, 83.0, 0.0,
    //     102.0, 68.0, -std::f32::consts::FRAC_PI_8.tan(),
    //     100.0, 68.0, 0.0,
    //     85.0,  83.0, -std::f32::consts::FRAC_PI_8.tan(),
    //     85.,   85., 0.0,
    //     100., 100., -std::f32::consts::FRAC_PI_8.tan(),
    // ];
    // let binding_box = [85.0, 68.0, 117.0, 100.0];
    // let info = SvgInfo::new(&binding_box, point.to_vec(), true, None);

    let rect = shape::Segment::new(10., 400.0, 250.0, 400.0, Some([20.0,10.0]));
    let info = rect.get_svg_info();

    let bbox = &info.binding_box;
    let pxrange = 5;
    let cur_off = 2;
    let sdf_tex_size = (bbox[2] - bbox[0]).max(bbox[3] - bbox[1]) * 0.5;
    
    let sdf = SvgInfo::compute_sdf_tex(&info,sdf_tex_size as usize, pxrange as u32, false, cur_off as u32, 1.0);
    // let sdf :SdfInfo2 = bitcode::deserialize(&sdf) .unwrap();
    log::debug!("sdf.sdf_tex: {}", sdf.sdf_tex[38 * 3 + 3]);
    let _ = image::save_buffer(
        "Rounded_rectangle.png",
        &sdf.sdf_tex,
        sdf.tex_size as u32,
        sdf.tex_size as u32,
        ColorType::L8,
    );
    // let buf = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 236, 207, 192, 191, 191, 192, 207, 236, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 222, 162, 115, 82, 66, 64, 64, 66, 82, 115, 162, 222, 255, 255, 255, 255, 255, 255, 255, 255, 255, 192, 115, 49, 0, 0, 0, 0, 0, 0, 0, 0, 49, 115, 192, 255, 255, 255, 255, 255, 255, 255, 192, 99, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 99, 192, 255, 255, 255, 255, 255, 222, 115, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 115, 222, 255, 255, 255, 255, 162, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 49, 162, 255, 255, 255, 236, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 115, 236, 255, 255, 207, 82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 82, 207, 255, 255, 192, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 192, 255, 255, 191, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 191, 255, 255, 191, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 191, 255, 255, 192, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 192, 255, 255, 207, 82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 82, 207, 255, 255, 236, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 115, 236, 255, 255, 255, 162, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 49, 162, 255, 255, 255, 255, 222, 115, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 115, 222, 255, 255, 255, 255, 255, 192, 99, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 99, 192, 255, 255, 255, 255, 255, 255, 255, 192, 115, 49, 0, 0, 0, 0, 0, 0, 0, 0, 49, 115, 192, 255, 255, 255, 255, 255, 255, 255, 255, 255, 222, 162, 115, 82, 66, 64, 64, 66, 82, 115, 162, 222, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 236, 207, 192, 191, 191, 192, 207, 236, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
    // let _ = image::save_buffer(
    //     "Rounded_rectangle2.png",
    //     &buf,
    //     22,
    //     22,
    //     ColorType::L8,
    // );
}
