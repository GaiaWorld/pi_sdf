use std::ffi::c_void;

use freetype_sys::FT_Vector;
use parry2d::math::Point;

use crate::glyphy::geometry::arcs::GlyphyArcAccumulator;

pub struct User {
    pub accumulate: GlyphyArcAccumulator,
    pub path_str: String,
    pub svg_paths: Vec<String>,
    pub svg_endpoints: Vec<[f32; 2]>,
}

pub extern "C" fn move_to(to: *const FT_Vector, user: *mut c_void) -> i32 {
    let to = unsafe { &*to };
    let user = unsafe { &mut *(user as *mut User) };

    if !user.accumulate.result.is_empty() {
        log::info!("+ Z");
        user.accumulate.close_path();
        user.path_str.push_str("Z");
        user.svg_paths.push(user.path_str.clone());
        user.path_str.clear();
    }
    log::info!("M {} {} ", to.x, to.y);
   
    user.accumulate
        .move_to(Point::new(to.x as f32, to.y as f32));
    user.path_str.push_str(&format!("M {} {}", to.x, to.y));
    user.svg_endpoints.push([to.x as f32, to.y as f32]);

    return 0;
}

pub extern "C" fn line_to(to: *const FT_Vector, user: *mut c_void) -> i32 {
    let to = unsafe { &*to };
    log::info!("+ L {} {} ", to.x, to.y);

    let user = unsafe { &mut *(user as *mut User) };
    user.accumulate
        .line_to(Point::new(to.x as f32, to.y as f32));
    user.path_str.push_str(&format!("L {} {}", to.x, to.y));
    user.svg_endpoints.push([to.x as f32, to.y as f32]);

    return 0;
}

pub extern "C" fn conic_to(
    control: *const FT_Vector,
    to: *const FT_Vector,
    user: *mut c_void,
) -> i32 {
    let control = unsafe { &*control };
    let to = unsafe { &*to };
    log::info!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);

    let user = unsafe { &mut *(user as *mut User) };
    user.accumulate.conic_to(
        Point::new(control.x as f32, control.y as f32),
        Point::new(to.x as f32, to.y as f32),
    );
    user.svg_endpoints.push([to.x as f32, to.y as f32]);
    return 0;
}

pub extern "C" fn cubic_to(
    control1: *const FT_Vector,
    control2: *const FT_Vector,
    to: *const FT_Vector,
    user: *mut c_void,
) -> i32 {
    let control1 = unsafe { &*control1 };
    let control2 = unsafe { &*control2 };
    let to = unsafe { &*to };
    log::info!(
        "+ C {} {} {} {} {} {} ",
        control1.x, control1.y, control2.x, control2.y, to.x, to.y
    );

    let user = unsafe { &mut *(user as *mut User) };

    return 0;
}
