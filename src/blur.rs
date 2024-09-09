use parry2d::bounding_volume::Aabb;

use crate::{glyphy::geometry::aabb::AabbEXT, Point, Vector2};

fn erf(mut x: f32) -> f32 {
    let negative = x < 0.0;
    if negative {
        x = -x;
    }

    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;
    let denom = 1.0 + 0.278393 * x + 0.230389 * x2 + 0.000972 * x3 + 0.078108 * x4;
    let result = 1.0 - 1.0 / (denom * denom * denom * denom);
    return if negative { -result } else { result };
}

// A useful helper for calculating integrals of the Gaussian function via the error function:
//
//      "erf"_sigma(x) = 2 int 1/sqrt(2 pi sigma^2) e^(-x^2/(2 sigma^2)) dx
//                     = "erf"(x/(sigma sqrt(2)))
fn erf_sigma(x: f32, sigma: f32) -> f32 {
    return erf(x / (sigma * 1.4142135623730951));
}

// Returns the blurred color value from the box itself (not counting any rounded corners). 'p_0' is
// the vector distance to the top left corner of the box; 'p_1' is the vector distance to its
// bottom right corner.
//
//      "colorFromRect"_sigma(p_0, p_1)
//          = int_{p_{0_y}}^{p_{1_y}} int_{p_{1_x}}^{p_{0_x}} G_sigma(y) G_sigma(x) dx dy
//          = 1/4 ("erf"_sigma(p_{1_x}) - "erf"_sigma(p_{0_x}))
//              ("erf"_sigma(p_{1_y}) - "erf"_sigma(p_{0_y}))
fn color_from_rect(p0: Vector2, p1: Vector2, sigma: f32) -> f32 {
    return (erf_sigma(p1.x, sigma) - erf_sigma(p0.x, sigma))
        * (erf_sigma(p1.y, sigma) - erf_sigma(p0.y, sigma))
        / 4.0;
}

// The blurred color value for the point at 'pos' with the top left corner of the box at
// 'p_{0_"rect"}' and the bottom right corner of the box at 'p_{1_"rect"}'.
fn get_shadow_alpha(pos: Point, pt_min: &Point, pt_max: &Point, sigma: f32) -> f32 {
    // Compute the vector distances 'p_0' and 'p_1'.
    let d_min = pos - pt_min;
    let d_max = pos - pt_max;

    // Compute the basic color '"colorFromRect"_sigma(p_0, p_1)'. This is all we have to do if
    // the box is unrounded.
    return color_from_rect(d_min, d_max, sigma);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct BlurInfo {
    pub tex: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bbox: Vec<f32>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn blur_box(bbox: &[f32], sigma: f32, txe_size: usize) -> BlurInfo {
    let bbox = Aabb::new(Point::new(bbox[0], bbox[1]), Point::new(bbox[2], bbox[3]));
    let b_w = bbox.width();
    let b_h = bbox.height();
    let px_dsitance = b_h.max(b_w) / txe_size as f32;

    let px_num = (sigma + sigma * 5.0).ceil();
    let dsitance = px_dsitance * px_num;
    println!("{:?}", (b_w, b_h, px_dsitance, px_num, dsitance));
    let p_w = (b_w / px_dsitance).ceil() + px_num * 2.0;
    let p_h = (b_h / px_dsitance).ceil() + px_num * 2.0;
    let mut pixmap = vec![0; (p_w * p_h) as usize];
    println!("{:?}", (p_w, p_h));
    let start = Point::new(bbox.mins.x - dsitance, bbox.mins.y - dsitance);
    for i in 0..p_w as usize {
        for j in 0..p_h as usize {
            let pos = Point::new(
                start.x + i as f32 * px_dsitance,
                start.y + j as f32 * px_dsitance,
            );
            let a = get_shadow_alpha(pos, &bbox.mins, &bbox.maxs, sigma);
            pixmap[j * p_w as usize + i as usize] = (a * 255.0) as u8;
        }
    }
    let atlas_bounds = Aabb::new(
        Point::new(px_num, px_num),
        Point::new(p_w - px_num, p_h - px_num),
    );
    println!("atlasBounds: {:?}", atlas_bounds);

    BlurInfo {
        tex: pixmap,
        width: p_w as usize,
        height: p_h as usize,
        bbox: vec![px_num, px_num, p_w - px_num, p_h - px_num],
    }
}
