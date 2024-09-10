use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use lyon_geom::{point, vector, Angle, ArcFlags};
use parry2d::na::Matrix3;

use crate::{glyphy::util::float_equals, utils::GlyphVisitor, Vector2};

fn arc_to_beziers(angle_start: f32, angle_extent: f32) -> Vec<f32> {
    let num_segments = (angle_extent.abs() * 2.0 / std::f32::consts::PI).ceil(); // (angleExtent / 90deg)

    let angle_increment = angle_extent / num_segments;

    // The length of each control point vector is given by the following formula.
    let control_length =
        4.0 / 3.0 * (angle_increment / 2.0).sin() / (1.0 + (angle_increment / 2.0).cos());

    let mut coords = Vec::with_capacity(num_segments as usize * 6);
    let mut pos = 0;

    for i in 0..num_segments as usize {
        let mut angle = angle_start + i as f32 * angle_increment;
        // Calculate the control vector at this angle
        let mut dx = angle.cos();
        let mut dy = angle.sin();
        // First control point
        coords[pos] = dx - control_length * dy;
        pos += 1;
        coords[pos] = dy + control_length * dx;
        pos += 1;
        // Second control point
        angle += angle_increment;
        dx = angle.cos();
        dy = angle.sin();
        coords[pos] = dx + control_length * dy;
        pos += 1;
        coords[pos] = dy - control_length * dx;
        pos += 1;
        // Endpoint of bezier
        coords[pos] = dx;
        pos += 1;
        coords[pos] = dy;
        pos += 1;
    }
    return coords;
}

fn checked_arc_cos(val: f32) -> f32 {
    if val < -1.0 {
        return std::f32::consts::PI;
    } else {
        if val > 1.0 {
            return 0.0;
        } else {
            return val.acos();
        }
    };
}

pub fn arc_to(
    last_x: f32,
    last_y: f32,
    mut rx: f32,
    mut ry: f32,
    angle: f32,
    large_arc_flag: bool,
    sweep_flag: bool,
    x: f32,
    y: f32,
    sink: &mut impl OutlineSink,
) {
    if last_x == x && last_y == y {
        // If the endpoints (x, y) and (x0, y0) are identical, then this
        // is equivalent to omitting the elliptical arc segment entirely.
        // (behaviour specified by the spec)
        return;
    }

    // Handle degenerate case (behaviour specified by the spec)
    if float_equals(rx, 0.0, None) || float_equals(ry, 0.0, None) {
        sink.line_to(Vector2F::new(x, y));
        return;
    }

    // Sign of the radii is ignored (behaviour specified by the spec)
    rx = rx.abs();
    ry = ry.abs();

    // Convert angle from degrees to radians
    let angle_rad = (angle % 360.0).to_radians();
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();

    // We simplify the calculations by transforming the arc so that the origin is at the
    // midpoint calculated above followed by a rotation to line up the coordinate axes
    // with the axes of the ellipse.

    // Compute the midpoint of the line between the current and the end point
    let dx2 = (last_x - x) / 2.0;
    let dy2 = (last_y - y) / 2.0;

    // Step 1 : Compute (x1', y1')
    // x1,y1 is the midpoint vector rotated to take the arc's angle out of consideration
    let x1 = cos_angle * dx2 + sin_angle * dy2;
    let y1 = -sin_angle * dx2 + cos_angle * dy2;

    let mut rx_sq = rx * rx;
    let mut ry_sq = ry * ry;
    let x1_sq = x1 * x1;
    let y1_sq = y1 * y1;

    // Check that radii are large enough.
    // If they are not, the spec says to scale them up so they are.
    // This is to compensate for potential rounding errors/differences between SVG implementations.
    let radii_check = x1_sq / rx_sq + y1_sq / ry_sq;
    if radii_check > 0.99999 {
        let radii_scale = radii_check.sqrt() * 1.00001;
        rx = radii_scale * rx;
        ry = radii_scale * ry;
        rx_sq = rx * rx;
        ry_sq = ry * ry;
    }

    // Step 2 : Compute (cx1, cy1) - the transformed centre point
    let mut sign = if large_arc_flag == sweep_flag {
        -1.0
    } else {
        1.0
    };
    let mut sq =
        ((rx_sq * ry_sq) - (rx_sq * y1_sq) - (ry_sq * x1_sq)) / ((rx_sq * y1_sq) + (ry_sq * x1_sq));
    sq = if sq < 0.0 { 0.0 } else { sq };
    let coef = sign * sq.sqrt();
    let cx1 = coef * ((rx * y1) / ry);
    let cy1 = coef * -((ry * x1) / rx);

    // Step 3 : Compute (cx, cy) from (cx1, cy1)
    let sx2 = (last_x + x) / 2.0;
    let sy2 = (last_y + y) / 2.0;
    let cx = sx2 + (cos_angle * cx1 - sin_angle * cy1);
    let cy = sy2 + (sin_angle * cx1 + cos_angle * cy1);

    // Step 4 : Compute the angleStart (angle1) and the angleExtent (dangle)
    let ux = (x1 - cx1) / rx;
    let uy = (y1 - cy1) / ry;
    let vx = (-x1 - cx1) / rx;
    let vy = (-y1 - cy1) / ry;
    let mut p;
    let mut n;

    // Angle betwen two vectors is +/- acos( u.v / len(u) * len(v))
    // Where '.' is the dot product. And +/- is calculated from the sign of the cross product (u x v)

    let two_pi = std::f32::consts::PI * 2.0;

    // Compute the start angle
    // The angle between (ux,uy) and the 0deg angle (1,0)
    n = ((ux * ux) + (uy * uy)).sqrt(); // len(u) * len(1,0) == len(u)
    p = ux; // u.v == (ux,uy).(1,0) == (1 * ux) + (0 * uy) == ux
    sign = if uy < 0.0 { -1.0 } else { 1.0 }; // u x v == (1 * uy - ux * 0) == uy
    let mut angle_start = sign * (p / n).acos(); // No need for checkedArcCos() here. (p >= n) should always be true.

    // Compute the angle extent
    n = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
    p = ux * vx + uy * vy;
    sign = if ux * vy - uy * vx < 0.0 { -1.0 } else { 1.0 };
    let mut angle_extent = sign * checked_arc_cos(p / n);

    // Catch angleExtents of 0, which will cause problems later in arcToBeziers
    if float_equals(angle_extent, 0.0, None) {
        sink.line_to(Vector2F::new(x, y));
        return;
    }

    if !sweep_flag && angle_extent > 0.0 {
        angle_extent -= two_pi;
    } else if sweep_flag && angle_extent < 0.0 {
        angle_extent += two_pi;
    }
    angle_extent %= two_pi;
    angle_start %= two_pi;

    // Many elliptical arc implementations including the Java2D and Android ones, only
    // support arcs that are axis aligned.  Therefore we need to substitute the arc
    // with bezier curves.  The following method call will generate the beziers for
    // a unit circle that covers the arc angles we want.
    let bezier_points = arc_to_beziers(angle_start, angle_extent);

    // Calculate a transformation matrix that will move and scale these bezier points to the correct location.
    // 定义缩放矩阵 (缩放 2 倍)
    let scale_matrix = Matrix3::new(rx, 0.0, 0.0, 0.0, ry, 0.0, 0.0, 0.0, 1.0);

    let rotation_matrix = Matrix3::new(
        angle.cos(),
        -angle.sin(),
        0.0,
        angle.sin(),
        angle.cos(),
        0.0,
        0.0,
        0.0,
        1.0,
    );

    // 定义平移矩阵 (平移 (2, 3))
    let translation_matrix = Matrix3::new(1.0, 0.0, cx, 0.0, 1.0, cy, 0.0, 0.0, 1.0);

    // The last point in the bezier set should match exactly the last coord pair in the arc (ie: x,y). But
    // considering all the mathematical manipulation we have been doing, it is bound to be off by a tiny
    // fraction. Experiments show that it can be up to around 0.00002.  So why don't we just set it to
    // exactly what it ought to be.
    // bezierPoints[bezierPoints.len() - 2] = x;
    // bezierPoints[bezierPoints.len() - 1] = y;

    // Final step is to add the bezier curves to the path
    for i in 0..bezier_points.len() / 6 {
        let c1 = Vector2::new(bezier_points[i * 6], bezier_points[i * 6 + 1]);
        let c1 = translation_matrix * rotation_matrix * scale_matrix * c1.to_homogeneous();

        let c2 = Vector2::new(bezier_points[i * 6 + 2], bezier_points[i * 6 + 3]);
        let c2 = translation_matrix * rotation_matrix * scale_matrix * c2.to_homogeneous();

        let to = Vector2::new(bezier_points[i * 6 + 4], bezier_points[i * 6 + 5]);
        let to = translation_matrix * rotation_matrix * scale_matrix * to.to_homogeneous();
        sink.cubic_curve_to(
            LineSegment2F::new(Vector2F::new(c1.x, c1.y), Vector2F::new(c2.x, c2.y)),
            Vector2F::new(to.x, to.y),
        );
    }
}

#[test]
fn test() {
    let mut sink = GlyphVisitor::new(0.1);
    let flags = ArcFlags {
        large_arc: false,
        sweep: true,
    };
    let arc = lyon_geom::SvgArc {
        from: point(172.55, 152.45),
        to: point(215.1, 109.9),
        radii: vector(30.0, 50.0),
        x_rotation: Angle::radians(-45.0),
        flags,
    };

    arc.for_each_cubic_bezier(&mut |s| {
        sink.cubic_curve_to(
            LineSegment2F::new(
                Vector2F::new(s.ctrl1.x as f32, s.ctrl1.y as f32),
                Vector2F::new(s.ctrl2.x as f32, s.ctrl2.y as f32),
            ),
            Vector2F::new(s.to.x as f32, s.to.y as f32),
        )
    });
}
