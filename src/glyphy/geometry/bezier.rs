use crate::Point;

use super::{point::PointExt, vector::VectorEXT};
use parry2d::math::Vector;

/// 表示一个三次贝塞尔曲线，由四个点定义：
/// - p0: 起点
/// - p1: 第一个控制点
/// - p2: 第二个控制点
/// - p3: 终点
#[derive(Debug)]
pub struct Bezier {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

// 3次 贝塞尔曲线
impl Bezier {
    /// 创建一个新的三次贝塞尔曲线
    /// # 参数
    /// - `p0`: 起点
    /// - `p1`: 第一个控制点
    /// - `p2`: 第二个控制点
    /// - `p3`: 终点
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /// 计算参数 `t` 对应的曲线上的点（德卡斯特里奥算法）
    /// # 参数
    /// - `t`: 曲线参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 参数 `t` 对应的点坐标
    pub fn point(&self, t: f32) -> Point {
        let p01 = self.p0.lerp(&self.p1, t);
        let p12 = self.p1.lerp(&self.p2, t);
        let p23 = self.p2.lerp(&self.p3, t);

        let p012 = p01.lerp(&p12, t);
        let p123 = p12.lerp(&p23, t);

        let p0123 = p012.lerp(&p123, t);

        return p0123;
    }

    /// 计算曲线的中点（等价于 t=0.5 时的点）
    /// # 返回值
    /// 曲线中点坐标
    pub fn midpoint(&self) -> Point {
        let p01 = self.p0.midpoint(&self.p1);
        let p12 = self.p1.midpoint(&self.p2);
        let p23 = self.p2.midpoint(&self.p3);

        let p012 = p01.midpoint(&p12);
        let p123 = p12.midpoint(&p23);

        let p0123 = p012.midpoint(&p123);

        return p0123;
    }

    /// 计算参数 `t` 处的切线向量（一阶导数）
    /// # 参数
    /// - `t`: 曲线参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 参数 `t` 处的切线向量
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

    /// 计算参数 `t` 处的二阶导数向量
    /// # 参数
    /// - `t`: 曲线参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 参数 `t` 处的二阶导数向量
    pub fn d_tangent(&self, t: f32) -> Vector<f32> {
        return Vector::new(
            6. * ((-self.p0.x + 3. * self.p1.x - 3. * self.p2.x + self.p3.x) * t
                + (self.p0.x - 2. * self.p1.x + self.p2.x)),
            6. * ((-self.p0.y + 3. * self.p1.y - 3. * self.p2.y + self.p3.y) * t
                + (self.p0.y - 2. * self.p1.y + self.p2.y)),
        );
    }

    /// 计算参数 `t` 处的曲率
    /// # 公式
    /// 曲率 κ = (dpp × ddp) / |dpp|³
    /// 其中 dpp 是切线向量的正交向量，ddp 是二阶导数向量
    /// # 参数
    /// - `t`: 曲线参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 参数 `t` 处的曲率值
    pub fn curvature(&self, t: f32) -> f32 {
        let dpp = self.tangent(t).ortho();
        let ddp = self.d_tangent(t);

        // normal vector len squared */
        let len = dpp.len() as f32;
        let curvature = (dpp.dot(&ddp)) / (len * len * len);
        return curvature;
    }

    /// 在参数 `t` 处将曲线分割为两条子贝塞尔曲线
    /// # 参数
    /// - `t`: 分割参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 元组：前半段曲线和后半段曲线
    pub fn split(&self, t: f32) -> (Bezier, Bezier) {
        // 德卡斯特里奥分割算法
        let p01 = self.p0.lerp(&self.p1, t);
        let p12 = self.p1.lerp(&self.p2, t);
        let p23 = self.p2.lerp(&self.p3, t);
        let p012 = p01.lerp(&p12, t);
        let p123 = p12.lerp(&p23, t);
        let p0123 = p012.lerp(&p123, t);

        // 前半段：起点到分割点，生成新控制点
        let first = Bezier::new(self.p0, p01, p012, p0123);
        // 后半段：分割点到终点，生成新控制点
        let second = Bezier::new(p0123, p123, p23, self.p3);
        return (first, second);
    }

    /// 将曲线平分为两条等长的子曲线（等价于 split(0.5)）
    /// # 返回值
    /// 元组：前半段曲线和后半段曲线
    pub fn halve(&self) -> (Bezier, Bezier) {
         // 使用中点分割优化计算
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

    /// 提取参数区间 [t0, t1] 对应的曲线段
    /// # 参数
    /// - `t0`: 起始参数，范围 [0.0, 1.0]
    /// - `t1`: 结束参数，范围 [0.0, 1.0]
    /// # 返回值
    /// 新的贝塞尔曲线，表示原曲线在 [t0, t1] 之间的部分
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
