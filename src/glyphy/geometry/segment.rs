// import { Point } from "./point";
// import { Line } from "./line";
// import { Arc } from "./arc";
// import { SignedVector } from "./signed_vector";

use std::ops::Range;

use crate::{glyphy::geometry::line::Line, Point};
use parry2d::{bounding_volume::Aabb, shape::Segment};

use super::{point::PointExt, signed_vector::SignedVector};

pub trait SegmentEXT {
    fn sub(&self, p: &Point) -> SignedVector;
    fn distance_to_point(&self, p: Point) -> f32;
    fn squared_distance_to_point(&self, p: &Point) -> f32;
    fn squared_distance_to_point2(&self, p: &Point) -> Self;
    fn contains_in_span(&self, p: Point) -> bool;
    fn projection_to_top_area(&self, aabb: &Aabb) -> Option<(Range<f32>, f32)>;
    fn projection_to_bottom_area(&self, aabb: &Aabb) -> Option<(Range<f32>, f32)>;
    fn projection_to_left_area(&self, aabb: &Aabb) -> Option<(Range<f32>, f32)>;
    fn projection_to_right_area(&self, aabb: &Aabb) -> Option<(Range<f32>, f32)>;
    fn nearest_points_on_line_segments(&self, other: &Segment) -> Segment;
    fn norm_squared(&self) -> f32;
}

impl SegmentEXT for Segment {
    /**
     * 从点到线段 的 最短向量
     */
    fn sub(&self, p: &Point) -> SignedVector {
        // Should the order (p1, p0) depend on d??
        return p.shortest_distance_to_line(&Line::from_points(self.b, self.a));
    }

    /**
     * 到 点p的距离
     */
    fn distance_to_point(&self, p: Point) -> f32 {
        if self.a == self.b {
            return 0.0;
        }

        // Check if z is between p0 and p1.
        let temp = Line::from_points(self.a, self.b);

        if self.contains_in_span(p) {
            let v = p.into_vector();
            let d = temp.n.dot(&v);
            return -(d - temp.c) / temp.n.norm();
        }

        let dist_p_p0 = p.distance_to_point(&self.a);
        let dist_p_p1 = p.distance_to_point(&self.b);

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
    fn squared_distance_to_point(&self, p: &Point) -> f32 {
        if self.a == self.b {
            return 0.0;
        }

        // Check if z is between p0 and p1.
        let temp = Line::from_points(self.a, self.b);
        if self.contains_in_span(*p) {
            let a = p.into_vector().dot(&temp.n) - temp.c;
            return a * a / temp.n.dot(&temp.n);
        }

        let dist_p_p0 = p.squared_distance_to_point(&self.a);
        let dist_p_p1 = p.squared_distance_to_point(&self.b);
        return if dist_p_p0 < dist_p_p1 {
            dist_p_p0
        } else {
            dist_p_p1
        };
    }

    fn squared_distance_to_point2(&self, p: &Point) -> Segment {
        let l2 = (self.a - self.b).norm_squared(); // i.e. |w-v|^2 -  avoid a sqrt
        if l2 == 0.0 {
            return Segment::new(*p, self.a);
        };
        let t = 0.0f32.max(1.0f32.min((p - self.a).dot(&(self.b - self.a)) / l2));
        let projection = self.a + t * (self.b - self.a); // Projection falls on the segment

        return Segment::new(*p, projection);
    }

    /**
     * 包含 在 线段上
     * @param {Point} p
     * @returns {boolean}
     */
    fn contains_in_span(&self, p: Point) -> bool {
        let p0 = self.a;
        let p1 = self.b;

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

    fn projection_to_top_area(&self, top_aabb: &Aabb) -> Option<(Range<f32>, f32)> {
        // 包含线段或者与aabb的边相交

        // let top_aabb = Aabb::new(
        //     Point::new(aabb.mins.x, -f32::INFINITY),
        //     Point::new(aabb.maxs.x, aabb.mins.y),
        // );

        println!("top_aabb: {:?}", top_aabb);
        if let Some(s) = top_aabb.clip_segment(&self.a, &self.b) {
            if s.a.y != top_aabb.maxs.y && s.b.y != top_aabb.maxs.y {
                println!("s: {:?}", s);
                let rang = if s.b.x > s.a.x {
                    s.a.x..s.b.x
                } else {
                    s.b.x..s.a.x
                };

                let dest1 = top_aabb.maxs.y - s.a.y;
                let dest2 = top_aabb.maxs.y - s.b.y;
                let dest = if dest1 < dest2 { dest1 } else { dest2 };

                return Some((rang, dest));
            }
        }

        None
    }

    fn projection_to_bottom_area(&self, bottom_aabb: &Aabb) -> Option<(Range<f32>, f32)> {
        // 包含线段或者与aabb的边相交
        println!("bottom_aabb: {:?}", bottom_aabb);
        if let Some(s) = bottom_aabb.clip_segment(&self.a, &self.b) {
            if s.a.y != bottom_aabb.mins.y && s.b.y != bottom_aabb.mins.y {
                println!("s: {:?}", s);
                // let d = s.b.x - s.a.x;
                let rang = if s.b.x > s.a.x {
                    s.a.x..s.b.x
                } else {
                    s.b.x..s.a.x
                };

                let dest1 = s.a.y - bottom_aabb.mins.y;
                let dest2 = s.b.y - bottom_aabb.mins.y;
                let dest = if dest1 < dest2 { dest1 } else { dest2 };

                return Some((rang, dest));
            }
        }

        None
    }

    fn projection_to_left_area(&self, left_aabb: &Aabb) -> Option<(Range<f32>, f32)> {
        // 包含线段或者与aabb的边相交
        if let Some(s) = left_aabb.clip_segment(&self.a, &self.b) {
            if s.a.x != left_aabb.maxs.x && s.b.x != left_aabb.maxs.x {
                println!("s: {:?}", s);
                let rang = if s.b.y > s.a.y {
                    s.a.y..s.b.y
                } else {
                    s.b.y..s.a.y
                };

                let dest1 = left_aabb.maxs.x - s.a.x;
                let dest2 = left_aabb.maxs.x - s.b.x;
                let dest = if dest1 < dest2 { dest1 } else { dest2 };

                return Some((rang, dest));
            }
        }

        None
    }

    fn projection_to_right_area(&self, right_aabb: &Aabb) -> Option<(Range<f32>, f32)> {
        // 包含线段或者与aabb的边相交
        if let Some(s) = right_aabb.clip_segment(&self.a, &self.b) {
            if s.a.x != right_aabb.mins.x && s.b.x != right_aabb.mins.x {
                println!("s: {:?}", s);
                let rang = if s.b.y > s.a.y {
                    s.a.y..s.b.y
                } else {
                    s.b.y..s.a.y
                };

                let dest1 = s.a.x - right_aabb.mins.x;
                let dest2 = s.b.x - right_aabb.mins.x;
                let dest = if dest1 < dest2 { dest1 } else { dest2 };

                return Some((rang, dest));
            }
        }

        None
    }

    fn nearest_points_on_line_segments(&self, other: &Segment) -> Segment {
        let eta = 1e-6;
        let r = other.a - self.a;
        let u = self.b - self.a;
        let v = other.b - other.a;

        let ru = r.dot(&u);
        let rv = r.dot(&v);
        let uu = u.dot(&u);
        let uv = u.dot(&v);
        let vv = v.dot(&v);

        let det = uu * vv - uv * uv;
        let s1;
        let t1;

        if det < eta * uu * vv {
            s1 = (ru / uu).clamp(0.0, 1.0);
            t1 = 0.0
        } else {
            s1 = ((ru * vv - rv * uv) / det).clamp(0.0, 1.0);
            t1 = ((ru * uv - rv * uu) / det).clamp(0.0, 1.0);
        }

        let s = ((t1 * uv + ru) / uu).clamp(0.0, 1.0);
        let t = ((s1 * uv - rv) / vv).clamp(0.0, 1.0);

        let a = self.a + s * u;
        let b = other.a + t * v;
        return Segment::new(a, b);
    }

    fn norm_squared(&self) -> f32 {
        (self.a - self.b).norm_squared()
    }
}

// #[test]
// fn test_projection_to_top_area() {
//     let aabb = Aabb::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)).near_area(Direction::Top);

//     let s1 = Segment::new(Point::new(1.0, -1.0), Point::new(4.0, -4.0));
//     assert_eq!(s1.projection_to_top_area(&aabb), Some((1.0..4.0, 1.0)));

//     let s1 = Segment::new(Point::new(-2.0, 0.0), Point::new(2.0, -2.0));
//     assert_eq!(s1.projection_to_top_area(&aabb), Some((0.0..2.0, 1.0)));

//     let s1 = Segment::new(Point::new(-2.0, -2.0), Point::new(2.0, 0.0));

//     assert_eq!(s1.projection_to_top_area(&aabb), None);
// }

// #[test]
// fn test_projection_to_bottom_area() {
//     let aabb = Aabb::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)).near_area(Direction::Bottom);

//     let s1 = Segment::new(Point::new(1.0, 11.0), Point::new(5.0, 15.0));
//     assert_eq!(s1.projection_to_bottom_area(&aabb), Some((1.0..5.0, 1.0)));

//     let s1 = Segment::new(Point::new(-2.0, 10.0), Point::new(2.0, 12.0));
//     assert_eq!(s1.projection_to_bottom_area(&aabb), Some((0.0..2.0, 1.0)));

//     let s1 = Segment::new(Point::new(-2.0, 12.0), Point::new(2.0, 10.0));
//     assert_eq!(s1.projection_to_bottom_area(&aabb), None);
// }

// #[test]
// fn test_projection_to_left_area() {
//     let aabb = Aabb::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)).near_area(Direction::Left);

//     let s1 = Segment::new(Point::new(-8.0, 8.0), Point::new(-4.0, 4.0));
//     assert_eq!(s1.projection_to_left_area(&aabb), Some((4.0..8.0, 4.0)));

//     let s1 = Segment::new(Point::new(-2.0, 2.0), Point::new(0.0, -2.0));
//     assert_eq!(s1.projection_to_left_area(&aabb), Some((0.0..2.0, 1.0)));

//     let s1 = Segment::new(Point::new(-4.0, -2.0), Point::new(0.0, 2.0));
//     assert_eq!(s1.projection_to_left_area(&aabb), None);
// }

// #[test]
// fn test_projection_to_right_area() {
//     let aabb = Aabb::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)).near_area(Direction::Right);

//     let s1 = Segment::new(Point::new(12.0, 2.0), Point::new(16.0, 6.0));
//     assert_eq!(s1.projection_to_right_area(&aabb), Some((2.0..6.0, 2.0)));

//     let s1 = Segment::new(Point::new(14.0, -2.0), Point::new(12.0, 2.0));
//     assert_eq!(s1.projection_to_right_area(&aabb), Some((0.0..2.0, 2.0)));

//     let s1 = Segment::new(Point::new(4.0, 2.0), Point::new(8.0, 6.0));
//     assert_eq!(s1.projection_to_right_area(&aabb), None);
// }
