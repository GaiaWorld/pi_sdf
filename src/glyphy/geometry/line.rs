

use pi_shape::glam::Vec2;
use pi_shape::plane::Point;

use crate::glyphy::util::{float_equals, GLYPHY_INFINITY};

use super::{point::PointExt, signed_vector::SignedVector, vector::VectorEXT};

#[derive(Debug, Clone)]
pub struct Line {
    pub n: Vec2,
    pub c: f32,
}

impl Line {
    pub fn new(a: f32, b: f32, c: f32) -> Self {
        Self {
            n: Vec2::new(a, b), /* line normal */
            c,                    /* n.x * x + n.y * y = c */
        }
    }

    /**
     * 从 法向量 和 距离 构造 直线
     */
    pub fn from_normal_d(n: Vec2, c: f32) -> Self {
        Self { n, c }
    }

    /**
     * 从 两点 构造 直线
     */
    pub fn from_points(p0: Point, p1: Point) -> Self {
        // let r  =Vector::new(0.0f32, 0.0f32);
        let n = (p1 - p0).ortho();
        let c = p0.into_vector().dot(n);
        Self { n, c }
    }

    /**
     * 归一化
     * @returns {Line}
     */
    pub fn normalized(&self) -> Self {
        let d = self.n.length();
        return if float_equals(d, 0.0, None) {
            self.clone()
        } else {
            Self::from_normal_d(self.n / d, self.c / d)
        };
    }

    /**
     * 返回 法向量
     * @returns {Vector}
     */
    pub fn normal(&self) -> &Vec2 {
        return &self.n;
    }

    /**
     * 交点
     */
    pub fn intersect(&self, l: Line) -> Point {
        let dot = self.n.x * l.n.y - self.n.y * l.n.x;
        if dot == 0.0 {
            return Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
        }

        return Point::new(
            (self.c * l.n.y - self.n.y * l.c) / dot,
            (self.n.x * l.c - self.c * l.n.x) / dot,
        );
    }

    /**
     * 点到直线的最短向量
     */
    pub fn sub(&self, p: &Point) -> SignedVector {
        let mag = -(self.n.dot(p.into_vector()) - self.c) / self.n.length();
        return SignedVector::from_vector(self.n.normalize() * (mag), mag < 0.0);
    }
}
