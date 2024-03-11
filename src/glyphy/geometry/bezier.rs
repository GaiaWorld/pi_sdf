use crate::Point;

use super::{point::PointExt, vector::VectorEXT};
use parry2d::math::Vector;

#[derive(Debug)]
pub struct Bezier {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

// 3次 贝塞尔曲线
impl Bezier {
    /**
     * @param p0 起点
     * @param p1 控制点1
     * @param p2 控制点2
     * @param p3 终点
     */
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /**
     * 求 参数t 对应的点
     * @param t 参数
     */
    pub fn point(&self, t: f32) -> Point {
        let p01 = self.p0.lerp(&self.p1, t);
        let p12 = self.p1.lerp(&self.p2, t);
        let p23 = self.p2.lerp(&self.p3, t);

        let p012 = p01.lerp(&p12, t);
        let p123 = p12.lerp(&p23, t);

        let p0123 = p012.lerp(&p123, t);

        return p0123;
    }

    /**
     * 求 中点
     */
    pub fn midpoint(&self) -> Point {
        let p01 = self.p0.midpoint(&self.p1);
        let p12 = self.p1.midpoint(&self.p2);
        let p23 = self.p2.midpoint(&self.p3);

        let p012 = p01.midpoint(&p12);
        let p123 = p12.midpoint(&p23);

        let p0123 = p012.midpoint(&p123);

        return p0123;
    }

    /**
     * 求 参数t 对应的切线
     * @param t 参数
     */
    pub fn tangent(&self, t: f32) -> Vector<f32> {
        let t_2_0 = t * t;
        let t_0_2 = (1. - t) * (1. - t);

        let _1_4t_1_0_3t_2_0 = 1. - 4. * t + 3. * t_2_0;
        let _2t_1_0_3t_2_0 = 2. * t - 3. * t_2_0;

        return Vector::new(
            -3. * self.p0.x * t_0_2
                + 3. * self.p1.x * _1_4t_1_0_3t_2_0
                + 3. * self.p2.x * _2t_1_0_3t_2_0
                + 3. * self.p3.x * t_2_0,
            -3. * self.p0.y * t_0_2
                + 3. * self.p1.y * _1_4t_1_0_3t_2_0
                + 3. * self.p2.y * _2t_1_0_3t_2_0
                + 3. * self.p3.y * t_2_0,
        );
    }

    /**
     * 求 参数t 对应的切线
     * @param t 参数
     */
    pub fn d_tangent(&self, t: f32) -> Vector<f32> {
        return Vector::new(
            6. * ((-self.p0.x + 3. * self.p1.x - 3. * self.p2.x + self.p3.x) * t
                + (self.p0.x - 2. * self.p1.x + self.p2.x)),
            6. * ((-self.p0.y + 3. * self.p1.y - 3. * self.p2.y + self.p3.y) * t
                + (self.p0.y - 2. * self.p1.y + self.p2.y)),
        );
    }

    /**
     * 求 参数t 对应的曲率
     * @param t 参数
     */
    pub fn curvature(&self, t: f32) -> f32 {
        let dpp = self.tangent(t).ortho();
        let ddp = self.d_tangent(t);

        // normal vector len squared */
        let len = dpp.len() as f32;
        let curvature = (dpp.dot(&ddp)) / (len * len * len);
        return curvature;
    }

    /**
     * 分割 曲线
     * @param t 参数
     */
    pub fn split(&self, t: f32) -> (Bezier, Bezier) {
        let p01 = self.p0.lerp(&self.p1, t);
        let p12 = self.p1.lerp(&self.p2, t);
        let p23 = self.p2.lerp(&self.p3, t);
        let p012 = p01.lerp(&p12, t);
        let p123 = p12.lerp(&p23, t);
        let p0123 = p012.lerp(&p123, t);

        let first = Bezier::new(self.p0, p01, p012, p0123);
        let second = Bezier::new(p0123, p123, p23, self.p3);
        return (first, second);
    }

    /**
     * TODO
     */
    pub fn halve(&self) -> (Bezier, Bezier) {
        let p01 = self.p0.midpoint(&self.p1);
        let p12 = self.p1.midpoint(&self.p2);
        let p23 = self.p2.midpoint(&self.p3);

        let p012 = p01.midpoint(&p12);
        let p123 = p12.midpoint(&p23);

        let p0123 = p012.midpoint(&p123);

        let first = Bezier::new(self.p0, p01, p012, p0123);
        let second = Bezier::new(p0123, p123, p23, self.p3);

        return (first, second);
    }

    /**
     * TODO
     * @param t0 {f32} 参数
     * @param t1 {f32} 参数
     * @returns {Bezier}
     */
    pub fn segment(&self, t0: f32, t1: f32) -> Bezier {
        let p01 = self.p0.lerp(&self.p1, t0);
        let p12 = self.p1.lerp(&self.p2, t0);
        let p23 = self.p2.lerp(&self.p3, t0);
        let p012 = p01.lerp(&p12, t0);
        let p123 = p12.lerp(&p23, t0);
        let p0123 = p012.lerp(&p123, t0);

        let q01 = self.p0.lerp(&self.p1, t1);
        let q12 = self.p1.lerp(&self.p2, t1);
        let q23 = self.p2.lerp(&self.p3, t1);
        let q012 = q01.lerp(&q12, t1);
        let q123 = q12.lerp(&q23, t1);
        let q0123 = q012.lerp(&q123, t1);

        let rp0 = p0123;
        let rp1 = p0123 + (p123 - p0123).scale((t1 - t0) / (1.0 - t0));
        let rp2 = q0123 + (q012 - q0123).scale((t1 - t0) / t1);
        let rp3 = q0123;
        return Bezier::new(rp0, rp1, rp2, rp3);
    }
}
