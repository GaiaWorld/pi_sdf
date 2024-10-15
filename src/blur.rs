
use crate::{ glyphy::geometry::aabb::Aabb, Point, Vector2};

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

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct BlurInfo {
    pub tex: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bbox: Vec<f32>,
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn blur_box(bbox: &[f32], pxrange: f32, txe_size: usize) -> BlurInfo {
    let bbox = Aabb::new(Point::new(bbox[0], bbox[1]), Point::new(bbox[2], bbox[3]));
    let b_w = bbox.width();
    let b_h = bbox.height();
    let px_dsitance = b_h.max(b_w) / (txe_size - 1) as f32; // 两边pxrange + 0.5， 中间应该减一

    // let px_num = (sigma + sigma * 5.0).ceil();
    let px_num = pxrange.ceil();
    let px_num2 = px_num + 0.5;
    let sigma = px_num / 6.0;
    let dsitance = px_dsitance * (px_num);
    // log::debug!("{:?}", (b_w, b_h, px_dsitance, px_num, dsitance, bbox));
    let p_w = (b_w / px_dsitance).ceil() + px_num2 * 2.0;
    let p_h = (b_h / px_dsitance).ceil() + px_num2 * 2.0;
    let mut pixmap = vec![0; (p_w * p_h) as usize];
    // log::debug!("{:?}", (p_w, p_h));
    let start = Point::new(bbox.mins.x - dsitance, bbox.mins.y - dsitance);
    // log::debug!("{:?}", start);
    let mut pos = Point::default();
    for i in 0..p_w as usize {
        for j in 0..p_h as usize {
            
            pos = Point::new(
                start.x + i as f32 * px_dsitance,
                start.y + j as f32 * px_dsitance,
            );
            // log::debug!("pos: {}", pos);
            let a = get_shadow_alpha(pos, &bbox.mins, &bbox.maxs, sigma);
            pixmap[j * p_w as usize + i as usize] = (a * 255.0) as u8;
        }
    }

    let maxs = if b_h > b_w{
        Point::new( b_w / px_dsitance + px_num2, p_h - px_num2)
    }else{
        Point::new( p_w - px_num2,  b_h / px_dsitance + px_num2)
    };

    let atlas_bounds = Aabb::new(
        Point::new(px_num2, px_num2),
        maxs,
    );
    log::debug!("atlasBounds: {:?}", atlas_bounds);

    BlurInfo {
        tex: pixmap,
        width: p_w as usize,
        height: p_h as usize,
        bbox: vec![px_num, px_num, p_w - px_num, p_h - px_num],
    }
}

const SCALE: f32 = 10.0;

pub fn gaussian_blur(
    sdf_tex: Vec<u8>,
    width: u32,
    height: u32,
    radius: u32,
    weight: f32,
) -> Vec<u8> {
    // let (width, height) = img.dimensions();
    let mut output = Vec::with_capacity(sdf_tex.len());
    let weight = -weight / SCALE;
    let kernel = create_gaussian_kernel(radius);
    let kernel_size = kernel.len() as u32;

    for y in 0..height {
        for x in 0..width {
            // let mut r = 0.0;
            // let mut g = 0.0;
            // let mut b = 0.0;
            let mut a = 0.0;
            let mut weight_sum = 0.0;

            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let px =
                        (x as i32 + kx as i32 - radius as i32).clamp(0, width as i32 - 1) as u32;
                    let py =
                        (y as i32 + ky as i32 - radius as i32).clamp(0, height as i32 - 1) as u32;

                    let sdf = sdf_tex[(px + py * width) as usize] as f32 / 255.0;
                    let fill_sd_px = sdf - (0.5 + weight);
                    let pixel = (fill_sd_px + 0.5).clamp(0.0, 1.0);

                    let weight = kernel[ky as usize][kx as usize];

                    // r += pixel[0] as f32 * weight;
                    // g += pixel[1] as f32 * weight;
                    // b += pixel[2] as f32 * weight;
                    a += pixel as f32 * weight;
                    weight_sum += weight;
                }
            }

            let pixel = (a / weight_sum * 255.0) as u8;

            output.push(pixel);
        }
    }

    output
}

fn create_gaussian_kernel(radius: u32) -> Vec<Vec<f32>> {
    let sigma = radius as f32 / 2.0;
    let size = radius * 2 + 1;
    let mut kernel = vec![vec![0.0; size as usize]; size as usize];
    let mut sum = 0.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - radius as f32;
            let dy = y as f32 - radius as f32;
            let value = (-((dx * dx + dy * dy) / (2.0 * sigma * sigma))).exp()
                / (2.0 * std::f32::consts::PI * sigma * sigma);
            kernel[y as usize][x as usize] = value;
            sum += value;
        }
    }

    for y in 0..size {
        for x in 0..size {
            kernel[y as usize][x as usize] /= sum;
        }
    }

    kernel
}


pub fn blur_box2(info: BoxInfo) -> Vec<u8> {
    let BoxInfo {
        p_w,
        p_h,
        start,
        px_dsitance,
        sigma,
        bbox,
        ..
    } = info;
    let mut pixmap = vec![0; (p_w * p_h) as usize];
    let start = Point::new(0.5,0.5);
    for i in 0..p_w as usize {
        for j in 0..p_h as usize {
            let pos: parry2d::na::OPoint<f32, parry2d::na::Const<2>> = Point::new(
                start.x + i as f32,
                start.y + j as f32,
            );
            let a = get_shadow_alpha(pos, &bbox.mins, &bbox.maxs, sigma);

            pixmap[j * p_w as usize + i as usize] = (a * 255.0) as u8;
        }
    }

    pixmap
}

#[derive(Debug, Clone)]
pub struct BoxInfo {
    pub p_w: f32,
    pub p_h: f32,
    start: Point,
    px_dsitance: f32,
    sigma: f32,
    pub atlas_bounds: Aabb,
    bbox: Aabb,
    pub radius: u32
}

pub fn compute_box_layout(bbox: Aabb, txe_size: usize, radius: u32) -> BoxInfo {
    let b_w = bbox.maxs.x - bbox.mins.x;
    let b_h = bbox.maxs.y - bbox.mins.y;

    let px_dsitance = b_h.max(b_w) / (txe_size - 1) as f32; // 两边pxrange + 0.5， 中间应该减一

    // let px_num = (sigma + sigma * 5.0).ceil();
    let px_num = radius as f32;
    let px_num2 = px_num + 0.5;
    let sigma = px_num / 3.0;
    let dsitance = px_dsitance * px_num;
    log::debug!("{:?}", (b_w, b_h, px_dsitance, px_num, dsitance, bbox));
    let p_w = (b_w / px_dsitance).ceil() + px_num2 * 2.0;
    let p_h = (b_h / px_dsitance).ceil() + px_num2 * 2.0;
    // let mut pixmap = vec![0; (p_w * p_h) as usize];
    log::debug!("{:?}", (p_w, p_h));
    let start = Point::new(bbox.mins.x - dsitance, bbox.mins.y - dsitance);

    let maxs = if b_h > b_w {
        Point::new(b_w / px_dsitance + px_num2, p_h - px_num2)
    } else {
        Point::new(p_w - px_num2, b_h / px_dsitance + px_num2)
    };

    let atlas_bounds = Aabb::new(Point::new(px_num2, px_num2), maxs);
    let info  =BoxInfo {
        p_w,
        p_h,
        start,
        px_dsitance,
        sigma,
        atlas_bounds,
        bbox: atlas_bounds,
        radius
    };
    log::debug!("BoxInfo: {:?}", info);
    info
    
}