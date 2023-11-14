// import { Point } from "./point";
// import { Line } from "./line";
// import { Arc } from "./arc";
// import { SignedVector } from "./signed_vector";

use crate::glyphy::geometry::line::Line;
use parry2d::math::Point;

use super::{point::PointExt, signed_vector::SignedVector};

pub struct Segment {
    p0: Point<f32>,
    p1: Point<f32>,
}

impl Segment {
    pub fn new(p0: Point<f32>, p1: Point<f32>) -> Self {
        Self { p0, p1 }
    }

    /**
     * 从点到线段 的 最短向量
     */
    pub fn sub(&self, p: &Point<f32>) -> SignedVector {
        // Should the order (p1, p0) depend on d??
        return p.shortest_distance_to_line(&Line::from_points(self.p1, self.p0));
    }

    /**
     * 到 点p的距离
     */
    pub fn distance_to_point(&self, p: Point<f32>) -> f32 {
        if self.p0 == self.p1 {
            return 0.0;
        }

        // Check if z is between p0 and p1.
        let temp = Line::from_points(self.p0, self.p1);

        if self.contains_in_span(p) {
            let v = p.into_vector();
            let d = temp.n.dot(&v);
            return -(d - temp.c) / temp.n.norm();
        }

        let dist_p_p0 = p.distance_to_point(&self.p0);
        let dist_p_p1 = p.distance_to_point(&self.p1);

        let d = if dist_p_p0 < dist_p_p1 {
            dist_p_p0
        } else {
            dist_p_p1
        };
        let rv = p.into_vector();
        let mag = temp.n.dot(&rv);
        let c = if -(mag - temp.c) < 0.0 { -1.0 } else { 1.0 };

        return d * c;
    }

    /**
     * 到 点p的距离的平方
     */
    pub fn squared_distance_to_point(&self, p: &Point<f32>) -> f32 {
        if self.p0 == self.p1 {
            return 0.0;
        }

        // Check if z is between p0 and p1.
        let temp = Line::from_points(self.p0, self.p1);
        if self.contains_in_span(*p) {
            let a = p.into_vector().dot(&temp.n) - temp.c;
            return a * a / temp.n.dot(&temp.n);
        }

        let dist_p_p0 = p.squared_distance_to_point(&self.p0);
        let dist_p_p1 = p.squared_distance_to_point(&self.p1);
        return if dist_p_p0 < dist_p_p1 {
            dist_p_p0
        } else {
            dist_p_p1
        };
    }

    /**
     * 包含 在 线段上
     * @param {Point} p
     * @returns {boolean}
     */
    pub fn contains_in_span(&self, p: Point<f32>) -> bool {
        let p0 = self.p0;
        let p1 = self.p1;

        if p0 == p1 {
            return false;
        }

        // shortest vector from point to line
        let temp = Line::from_points(p0, p1);
        let v = p.into_vector();
        let d = temp.n.dot(&v);
        let mag = -(d - temp.c) / temp.n.norm();

        let y = temp.n.normalize().scale(mag);
        let z = p + y;

        // Check if z is between p0 and p1.
        if (p1.y - p0.y).abs() > (p1.x - p0.x).abs() {
            return (z.y - p0.y > 0.0 && p1.y - p0.y > z.y - p0.y)
                || (z.y - p0.y < 0.0 && p1.y - p0.y < z.y - p0.y);
        } else {
            return (0.0 < z.x - p0.x && z.x - p0.x < p1.x - p0.x)
                || (0.0 > z.x - p0.x && z.x - p0.x > p1.x - p0.x);
        }
    }

    // /**
    //  * 到 圆弧的 最大距离
    //  */
    // pub fn max_distance_to_arc(&self, a: Arc) {
    //     let max_distance = Math.abs(a.distance_to_point(self.p0));
    //     return max_distance > Math.abs(a.distance_to_point(self.p1)) ? max_distance : Math.abs(a.distance_to_point(self.p1));
    // }
}
