/* Returns 3 max(abs(d₀ t (1-t)² + d₁ t² (1-t)) for 0≤t≤1. */
// class MaxDeviationApproximatorExact
pub fn approximate_deviation(d0: f32, d1: f32) -> f32 {
    let mut candidates = [0., 1., 0., 0.];
    let mut num_candidates = 2;
    if d0 == d1 {
        candidates[num_candidates] = 0.5;
        num_candidates += 1;
    } else {
        let delta = d0 * d0 - d0 * d1 + d1 * d1;
        let t2 = 1. / (3. * (d0 - d1));
        let t0 = (2. * d0 - d1) * t2;
        if delta == 0.0 {
            candidates[num_candidates] = t0;
            num_candidates += 1
        } else if delta > 0.0 {
            /* This code can be optimized to avoid the sqrt if the solution
            	* is not feasible (ie. lies outside (0,1)).  I have implemented
            	* that in cairo-spline.c:_cairo_spline_bound().  Can be reused
            	* here.
            	*/
            let t1 = delta.sqrt() * t2;
            candidates[num_candidates] = t0 - t1;
            num_candidates += 1;
            candidates[num_candidates] = t0 + t1;
            num_candidates += 1;
        }
    }

    let mut e = 0.;
    for i in 0..num_candidates {
        let t = candidates[i];
        let ee;
        if t < 0. || t > 1. {
            continue;
        }

        ee = (3. * t * (1. - t) * (d0 * (1. - t) + d1 * t)).abs();
        e = if e > ee { e } else { ee };
    }

    return e;
}

// class ArcBezierErrorApproximatorBehdad<MaxDeviationApproximator>
pub fn approximate_bezier_arc_error(
    b0: &Bezier,
    a: &Arc,
    approximate_deviation: fn(f32, f32) -> f32,
) -> f32 {
    
    assert!(b0.p0 == a.p0);
    assert!(b0.p3 == a.p1);

    let mut ea = ErrorValue { value: 0.0 };
    let b1 = a.approximate_bezier(&mut ea);

    assert!(b0.p0 == b1.p0);
    assert!(b0.p3 == b1.p3);

    let mut v0 = b1.p1 - b0.p1;
    let mut v1 = b1.p2 - (b0.p2);

    let b = (b0.p3 - b0.p0).normalize();
    v0 = v0.rebase_other(&b);
    v1 = v1.rebase_other(&b);

    let d1 = approximate_deviation(v0.x, v1.x);
    let d2 = approximate_deviation(v0.y, v1.y);
    let v = Vector::new(d1, d2);

    /* Edge cases: If d*d is too close too large default to a weak bound. */
    if a.d * a.d > 1. - 1e-4 {
        return ea.value + v.norm();
    }

    /* If the wedge doesn't contain control points, default to weak bound. */
    if !a.wedge_contains_point(&b0.p1) || !a.wedge_contains_point(&b0.p2) {
        return ea.value + v.norm();
    }

    /* If straight line, return the max ortho deviation. */
    if a.d.abs() < 1e-6 {
        return ea.value + v.y;
    }

    /* We made sure that Math.abs(a.d) < 1 */
    let tan_half_alpha = tan2atan(a.d).abs();

    let tan_v = v.x / v.y;

    let mut _eb = 0.0;
    if tan_v.abs() <= tan_half_alpha {
        return ea.value + v.norm();
    }

    let c2 = (a.p1 - a.p0).norm() * 0.5;
    let r = a.radius();

    _eb = Vector::new(c2 + v.x, c2 / tan_half_alpha + v.y).norm() - r;
    assert!(_eb >= -0.01);

    return ea.value + _eb;
}

// export class ArcBezierApproximatorMidpointSimple
pub fn arc_bezier_approximator_midpoint_simple(
    b: Bezier,
    error: &mut ErrorValue,
    approximate_bezier_arc_error: Box<dyn Fn(Bezier, &Arc) -> f32>,
) -> Arc {
    let a = Arc::from_points(b.p0, b.p3, b.midpoint(), false);

    error.value = approximate_bezier_arc_error(b, &a);

    return a;
}

// class ArcBezierApproximatorMidpointTwoPart
pub fn arc_bezier_approximator_midpoint_two_part(
    b: &Bezier,
    error: &mut ErrorValue,
    mid_t: Option<f32>,
    approximate_bezier_arc_error: fn(&Bezier, &Arc) -> f32,
) -> Arc {
    let mid_t = if let Some(v) = mid_t { v } else { 0.5 };

    let pair = b.split(mid_t);
    let m = pair.1.p0;

    let a0 = Arc::from_points(b.p0, m, b.p3, true);
    let a1 = Arc::from_points(m, b.p3, b.p0, true);

    let e0 = approximate_bezier_arc_error(&pair.0, &a0);
    let e1 = approximate_bezier_arc_error(&pair.1, &a1);
    error.value = if e0 > e1 { e0 } else { e1 };

    return Arc::from_points(b.p0, b.p3, m, false);
}

pub struct ArcBezierApproximatorQuantized {
    max_d: f32,
    d_bits: i32,
}

impl ArcBezierApproximatorQuantized {
    pub fn new(_max_d: Option<f32>, _d_bits: Option<i32>) -> Self {
        let max_d = if let Some(v) = _max_d {
            v
        } else {
            GLYPHY_INFINITY
        };

        let d_bits = if let Some(v) = _d_bits { v } else { 0 };

        Self { max_d, d_bits }
    }

    pub fn approximate_bezier_with_arc(
        &self,
        b: Bezier,
        error: &mut ErrorValue,
        approximate_bezier_arc_error: fn(&Bezier, &Arc) -> f32,
    ) -> Arc {
        let mid_t = 0.5;
        // println!("b: {:?}", b);
        let mut a = Arc::from_points(b.p0, b.p3, b.point(mid_t), false);
        let orig_a = a.clone();

        if self.max_d < f32::INFINITY && self.max_d > -f32::INFINITY {
            assert!(self.max_d >= 0.);
            if a.d.abs() > self.max_d {
                a.d = if a.d < 0.0 { -self.max_d } else { self.max_d };
            }
        }
        if self.d_bits != 0 && self.max_d != 0. {
            assert!(self.max_d < f32::INFINITY && self.max_d > -f32::INFINITY);
            // if a.d.abs() > self.max_d {
            //     println!("a.d.abs(): {}, self.max_d: {}", a.d.abs(), self.max_d);
            // }
            // println!("a.d.abs(): {}, self.max_d: {}", a.d.abs(), self.max_d);
            assert!(a.d.abs() <= self.max_d);

            let _v = 1;
            let mult = (1 << (self.d_bits - 1) as u32) - 1;
            let id = (a.d / self.max_d * mult as f32).round();
            assert!(-mult as f32 <= id && id <= mult as f32);
            a.d = id * self.max_d / mult as f32;
            assert!(a.d.abs() <= self.max_d);
        }

        /* Error introduced by arc quantization */
        let ed = (a.d - orig_a.d).abs() * (a.p1 - a.p0).norm() * 0.5;

        arc_bezier_approximator_midpoint_two_part(
            &b,
            error,
            Some(mid_t),
            approximate_bezier_arc_error,
        );

        if ed != 0.0 {
            error.value += ed;

            /* Try a simple one-arc approx which works with the quantized arc.
            	* May produce smaller error bound. */
            let e = approximate_bezier_arc_error(&b, &a);
            if e < error.value {
                error.value = e;
            }
        }

        return a;
    }
}

pub use approximate_deviation as max_deviation_approximator_default;

pub fn arc_bezier_error_approximator_default(b0: &Bezier, a: &Arc) -> f32 {
    return approximate_bezier_arc_error(b0, a, max_deviation_approximator_default);
}

pub fn arc_bezier_approximator_default(
    b: Bezier,
    error: &mut ErrorValue,
    mid_t: Option<f32>,
) -> Arc {
    return arc_bezier_approximator_midpoint_two_part(
        &b,
        error,
        mid_t,
        arc_bezier_error_approximator_default,
    );
}

use parry2d::math::Vector;
pub use ArcBezierApproximatorQuantized as ArcBezierApproximatorQuantizedDefault;

use crate::glyphy::geometry::{
    arc::{tan2atan, ErrorValue},
    vector::VectorEXT,
};

use super::{
    geometry::{arc::Arc, bezier::Bezier},
    util::GLYPHY_INFINITY,
};
