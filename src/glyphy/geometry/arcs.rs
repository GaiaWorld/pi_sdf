/*
 * Approximate outlines with multiple arcs
 */

use parry2d::bounding_volume::Aabb;
use parry2d::math::Point;

use super::aabb::AabbEXT;
use super::arc::{Arc, ArcEndpoint, ErrorValue};
use super::bezier::Bezier;
use super::vector::VectorEXT;
use crate::glyphy::arc_bezier::{
    arc_bezier_error_approximator_default, ArcBezierApproximatorQuantized,
};
use crate::glyphy::geometry::point::PointExt;
use crate::glyphy::util::{is_zero, GLYPHY_INFINITY, GLYPHY_MAX_D};

pub struct GlyphyArcAccumulator {
    pub(crate) tolerance: f32,
    pub(crate) max_d: f32,
    pub(crate) d_bits: i32,
    pub(crate) start_point: Point<f32>,
    pub(crate) current_point: Point<f32>,
    pub(crate) need_moveto: bool,
    pub(crate) num_endpoints: f32,
    pub(crate) max_error: f32,
    pub(crate) success: bool,
    pub result: Vec<ArcEndpoint>,
}

impl GlyphyArcAccumulator {
    pub fn new() -> Self {
        let mut res = Self {
            tolerance: 5e-4,
            max_d: GLYPHY_MAX_D,
            d_bits: 8,
            start_point: Point::new(0., 0.),
            current_point: Point::new(0., 0.),
            need_moveto: true,
            num_endpoints: 0.0,
            max_error: 0.0,
            success: true,
            result: Vec::new(),
        };

        res.reset();
        res
    }

    pub fn reset(&mut self) {
        self.current_point = Point::new(0.0, 0.0);
        self.start_point = self.current_point;
        self.need_moveto = true;
        self.num_endpoints = 0.0;
        self.max_error = 0.0;
        self.success = true;
    }

    // d = inf，就是 移动点
    pub fn move_to(&mut self, p: Point<f32>) {
        if self.num_endpoints != 0.0 || !p.equals(&self.current_point) {
            self.accumulate(p, GLYPHY_INFINITY);
        }
    }

    // d = 0 就是 线段
    pub fn line_to(&mut self, p1: Point<f32>) {
        self.arc_to(p1, 0.0)
    }

    // 2次 贝塞尔，升阶到 3次; 公式见: https://blog.csdn.net/xhhjin/article/details/62905007
    //
    // 输入：
    //	 + P0, P2 是 2次 Bezier 的 起点，终点
    //	 + P1 是 控制点；
    //
    // 升阶到 3次后：
    //   + Q0, Q3 是 3次 Beizer 的 起点，终点
    //	 + Q1, Q2 是 控制点
    //
    // 算法：
    //   + Q0 = P0
    //	 + Q1 = 1 / 3 * P0 + 2 / 3 * P1
    //	 + Q2 = 1 / 3 * P2 + 2 / 3 * P1
    //	 + Q3 = P2
    //
    pub fn conic_to(&mut self, p1: Point<f32>, p2: Point<f32>) {
        let b = Bezier::new(
            self.current_point,
            self.current_point.lerp(&p1, 2. / 3.),
            p2.lerp(&p1, 2. / 3.),
            p2,
        );
        self.bezier(b);
    }

    // 3次 贝塞尔曲线，用 圆弧 拟合
    pub fn cubic_to(&mut self, p1: Point<f32>, p2: Point<f32>, p3: Point<f32>) {
        let b = Bezier::new(self.current_point, p1, p2, p3);
        self.bezier(b);
    }

    pub fn close_path(&mut self) {
        if !self.need_moveto && !self.current_point.equals(&self.start_point) {
            self.arc_to(self.start_point, 0.0);
        }
    }

    pub fn emit(&mut self, p: Point<f32>, d: f32) {
        let endpoint = ArcEndpoint::new(p.x, p.y, d);
        self.result.push(endpoint);

        self.num_endpoints += 1.0;
        self.current_point = p;
    }

    pub fn accumulate(&mut self, p: Point<f32>, d: f32) {
        if p.equals(&self.current_point) {
            return;
        }

        if d == GLYPHY_INFINITY {
            /* Emit moveto lazily, for cleaner outlines */
            self.need_moveto = true;
            self.current_point = p;
            return;
        }
        if self.need_moveto {
            self.emit(self.current_point, GLYPHY_INFINITY);
            self.start_point = self.current_point;
            self.need_moveto = false;
        }
        self.emit(p, d);
    }

    pub fn arc_to(&mut self, p1: Point<f32>, d: f32) {
        self.accumulate(p1, d);
    }

    // 圆弧 拟合 贝塞尔
    pub fn bezier(&mut self, b: Bezier) {
        let appx = ArcBezierApproximatorQuantized::new(Some(self.max_d), Some(self.d_bits));

        // 圆弧 拟合 贝塞尔 的 主要实现
        // let inner =  ArcsBezierApproximatorSpringSystem;

        let mut arcs = vec![];
        let e = ArcsBezierApproximatorSpringSystem::approximate_bezier_with_arcs(
            &b,
            self.tolerance,
            &appx,
            &mut arcs,
            None,
        );

        self.max_error = self.max_error.max(e);

        self.move_to(b.p0);
        for i in 0..arcs.len() {
            self.arc_to(arcs[i].p1, arcs[i].d);
        }
    }
}

pub fn glyphy_arc_list_extents(endpoints: &Vec<ArcEndpoint>, extents: &mut Aabb) {
    let mut p0 = Point::new(0., 0.);
    extents.clear();

    let num_endpoints = endpoints.len();
    for i in 0..num_endpoints {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = endpoint.p;
            continue;
        }
        let arc = Arc::new(p0, endpoint.p, endpoint.d);
        p0 = endpoint.p;

        let mut arc_extents = Aabb::new(
            Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY),
            Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY),
        );
        arc.extents(&mut arc_extents);
        extents.extend(&arc_extents);
    }
}

use ArcBezierApproximatorQuantized as ArcBezierApproximatorQuantizedDefault;
pub struct ArcsBezierApproximatorSpringSystem;

impl ArcsBezierApproximatorSpringSystem {
    pub fn calc_arcs(
        b: &Bezier,
        t: &[f32],
        appx: &ArcBezierApproximatorQuantizedDefault,
        e: &mut Vec<f32>,
        arcs: &mut Vec<Arc>,
        mut max_e: f32,
        mut min_e: f32,
    ) -> [f32; 2] {
        let n = t.len() - 1;

        *e = vec![0.0; n];
        // println!("e.len: {}", e.len());
        arcs.clear();

        max_e = 0.0;
        min_e = GLYPHY_INFINITY;

        for i in 0..n {
            let segment = b.segment(t[i], t[i + 1]);
            let mut temp = ErrorValue { value: 0.0 };
            let arc = appx.approximate_bezier_with_arc(
                segment,
                &mut temp,
                arc_bezier_error_approximator_default,
            );
            arcs.push(arc);
            // println!("n: {}", n);
            e[i] = temp.value;

            max_e = max_e.max(e[i]);
            min_e = min_e.min(e[i]);
        }

        return [min_e, max_e];
    }

    pub fn jiggle(
        b: &Bezier,
        appx: &ArcBezierApproximatorQuantizedDefault,
        t: &mut Vec<f32>,
        e: &mut Vec<f32>,
        arcs: &mut Vec<Arc>,
        mut max_e: f32,
        mut min_e: f32,
        tolerance: f32,
    ) -> [f32; 3] {
        let n = t.len() - 1;
        let conditioner = tolerance * 0.01;
        let max_jiggle = n.ilog2() + 1;

        let mut n_jiggle = 0;
        for _s in 0..max_jiggle {
            let mut total = 0.0;
            for i in 0..n {
                let l = t[i + 1] - t[i];
                let k_inv = l * (e[i] + conditioner).powf(-0.3);
                total += k_inv;
                e[i] = k_inv;
            }
            for i in 0..n {
                let k_inv = e[i];
                let l = k_inv / total;
                t[i + 1] = t[i] + l;
            }
            t[n] = 1.0; // Do self to get real 1.0, not .9999999999999998!

            [min_e, max_e] = Self::calc_arcs(&b, &t, &appx, e, arcs, max_e, min_e);

            //fprintf (stderr, "n %d jiggle %d max_e %g min_e %g\n", n, s, max_e, min_e);

            n_jiggle += 1;
            if max_e < tolerance || (2.0 * min_e - max_e > tolerance) {
                break;
            }
        }
        return [n_jiggle as f32, min_e, max_e];
    }

    // 圆弧 拟合 3次 Bezier
    // 返回 最大误差
    pub fn approximate_bezier_with_arcs(
        b: &Bezier,
        tolerance: f32,
        appx: &ArcBezierApproximatorQuantizedDefault,
        arcs: &mut Vec<Arc>,
        max_segments: Option<i32>,
    ) -> f32 {
        let max_segments = if let Some(v) = max_segments { v } else { 100 };
        /* Handle fully-degenerate cases. */
        let v1 = b.p1 - (b.p0);
        let v2 = b.p2 - (b.p0);
        let v3 = b.p3 - (b.p0);
        if is_zero(v1.sdf_cross(&v2), None) && is_zero(v2.sdf_cross(&v3), None) {
            // Curve has no area.  If endpoints are NOT the same, replace with single line segment.  Otherwise fully skip. */
            arcs.clear();
            if !b.p0.equals(&b.p1) {
                arcs.push(Arc::new(b.p0, b.p3, 0.0));
            }
            return 0.0;
        }

        let mut t = vec![];
        let mut e = vec![];

        let mut max_e = 0.0;
        let mut min_e = 0.0;
        // let mut n_jiggle = 0.0;

        /* Technically speaking we can bsearch for n. */
        for n in 1..max_segments as usize {
            t = vec![0.0; n + 1];
            for i in 0..n {
                t[i] = i as f32 / n as f32;
            }
            t[n] = 1.0; // Do self out of the loop to get real 1.0, not .9999999999999998!

            [min_e, max_e] = Self::calc_arcs(b, &t, appx, &mut e, arcs, max_e, min_e);

            let mut jiggle = 0.0;
            for i in 0..n {
                if e[i] <= tolerance {
                    [_, min_e, max_e] =
                        Self::jiggle(b, appx, &mut t, &mut e, arcs, max_e, min_e, tolerance);
                    // n_jiggle += jiggle;
                    break;
                }
            }
            if max_e <= tolerance {
                break;
            }
        }
        return max_e;
    }
}
