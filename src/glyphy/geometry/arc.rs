use parry2d::{
    bounding_volume::Aabb,
    math::{Point, Vector},
};

use crate::glyphy::util::{float_equals, xor};

use super::{
    aabb::AabbEXT, bezier::Bezier, line::Line, point::PointExt, segment::Segment,
    signed_vector::SignedVector, vector::VectorEXT,
};

// sin( 2 * atan(d) )
pub fn sin2atan(d: f32) -> f32 {
    return 2.0 * d / (1.0 + d * d);
}

// cos( 2 * atan(d) )
pub fn cos2atan(d: f32) -> f32 {
    return (1. - d * d) / (1. + d * d);
}

// tan( 2 * atan(d) )
pub fn tan2atan(d: f32) -> f32 {
    return 2. * d / (1. - d * d);
}

pub struct ErrorValue {
    pub(crate) value: f32,
}

#[derive(Debug, Clone)]
pub struct ArcEndpoint {
    pub p: Point<f32>,
    pub d: f32,

    // 线段特殊处理，只有一个值
    pub line_key: Option<String>,

    pub line_encode: Option<[f32; 4]>, // rgba
}

impl ArcEndpoint {
    pub fn new(x: f32, y: f32, d: f32) -> Self {
        Self {
            p: Point::new(x, y),
            d,   
            line_key: None,
            line_encode: None,
        }
    }
}

// d 几何意义 为 tan( 圆心角 / 4 )
// 绝对值：圆心角 [0, 2 PI]，圆心角 / 4 [0, PI / 2]，tan [0, +∞]
//
// 区分 小圆弧 还是 大圆弧
//    小圆弧，圆心角 < PI，圆心角 / 4 < PI / 4，tan < 1，|d| < 1
//    大圆弧，圆心角 > PI，圆心角 / 4 > PI / 4，tan > 1，|d| > 1
//
// d符号，表示圆心的方向（在 圆弧垂线的左边，还是右边）
//    d > 0，和 (终 - 起).otho() 同向
//    d < 0，和 上面 相反
#[derive(Debug, Clone)]
pub struct Arc {
    pub p0: Point<f32>,
    pub p1: Point<f32>,
    pub d: f32,
}

impl Arc {
    /**
     * 构造函数
     */
    pub fn new(p0: Point<f32>, p1: Point<f32>, d: f32) -> Self {
        Self { p0, p1, d }
    }

    /**
     * 从三个点 构造 圆弧
     * @param p0 起点
     * @param p1 终点
     * @param pm 中间点
     * @param complement 是否补弧
     */
    pub fn from_points(p0: Point<f32>, p1: Point<f32>, pm: Point<f32>, complement: bool) -> Self {
        let mut arc = Arc::new(p0, p1, 0.0);
        if p0 != pm && p1 != pm {
            let v = p1 - pm;
            let u = p0 - pm;
            arc.d = (((v.sdf_angle() - u.sdf_angle()) / 2.)
                - (if complement {
                    0.
                } else {
                    std::f32::consts::PI / 2.
                }))
            .tan();
        }
        return arc;
    }

    /**
     * 从圆心、半径、起始角度、终止角度 构造 圆弧
     * @param center 圆心
     * @param radius 半径
     * @param a0 起始角度
     * @param a1 终止角度
     * @param complement 是否补弧
     */
    pub fn from_center_radius_angle(
        center: Point<f32>,
        radius: f32,
        a0: f32,
        a1: f32,
        complement: bool,
    ) -> Self {
        let p0 = center + Vector::new(a0.cos(), a0.sin()).scale(radius);
        let p1 = center + Vector::new(a1.cos(), a1.sin()).scale(radius);
        let v1 = (a1 - a0) / 4.0;
        let v2 = if complement {
            0.
        } else {
            std::f32::consts::PI / 2.
        };

        return Arc::new(p0, p1, (v1 - v2).tan());
    }

    pub fn to_svg_command(&self) -> String {
        let start_point = self.p0;
        let end_point = self.p1;

        let radius = self.radius();
        let center = self.center();

        let start_angle = (start_point.y - center.y).atan2(start_point.x - center.x);
        let end_angle = (end_point.y - center.y).atan2(end_point.x - center.x);

        // large-arc-flag 是一个布尔值（0 或 1），表示是否选择较大的弧（1）或较小的弧（0）
        let large_arc_flag = if (end_angle - start_angle).abs() > std::f32::consts::PI {
            1
        } else {
            0
        };

        // sweep-flag 是一个布尔值（0 或 1），表示弧是否按顺时针（1）或逆时针（0）方向绘制。
        let sweep_flag = if self.d > 0. { 1. } else { 0. };

        // x-axis-rotation 是椭圆的 x 轴与水平方向的夹角，单位为度。
        // A rx ry x-axis-rotation large-arc-flag sweep-flag x y
        let arc_command = format!(
            "A {} {} 0 {} {} {} {} ",
            radius, radius, large_arc_flag, sweep_flag, end_point.x, end_point.y
        );

        return arc_command;
    }

    /**
     * 减去 点
     */
    pub fn sub(&self, p: Point<f32>) -> SignedVector {
        // todo!()
        if self.d.abs() < 1e-5 {
            let arc_segment = Segment::new(self.p0, self.p1);
            return arc_segment.sub(&p);
        }

        if self.wedge_contains_point(&p) {
            let difference = (self.center() - p)
                .normalize()
                .scale((p.distance_to_point(&self.center()) - self.radius()).abs());

            let d = xor(self.d < 0., (p - self.center()).norm() < self.radius());
            return SignedVector::from_vector(difference, d);
        }

        let d0 = p.squared_distance_to_point(&self.p0);
        let d1 = p.squared_distance_to_point(&self.p1);

        let other_arc = Arc::new(self.p0, self.p1, (1.0 + self.d) / (1.0 - self.d));
        let normal = self.center() - (if d0 < d1 { self.p0 } else { self.p1 });

        if normal.len() == 0 {
            return SignedVector::from_vector(Vector::new(0., 0.), true);
        }

        let min_p = if d0 < d1 { self.p0 } else { self.p1 };
        let l = Line::new(normal.x, normal.y, normal.dot(&min_p.into_vector()));
        return SignedVector::from_vector(l.sub(&p).vec2, !other_arc.wedge_contains_point(&p));
    }

    /**
     * 计算圆弧的半径
     * @returns {f32} 圆弧半径
     */
    pub fn radius(&self) -> f32 {
        return ((self.p1 - (self.p0)).norm() / (2.0 * sin2atan(self.d))).abs();
    }

    /**
     * 计算圆弧的圆心
     * @returns {Point} 圆弧的圆心
     */
    pub fn center(&self) -> Point<f32> {
        return (self.p0.midpoint(&self.p1)).add_vector(
            &(self.p1 - (self.p0))
                .ortho()
                .scale(1. / (2. * tan2atan(self.d))),
        );
    }

    /**
     * 计算圆弧 的 切线向量对
     *
     * 圆弧切线，就是 圆弧端点在圆上的切线
     *
     * 切线向量 和 圆心到圆弧端点的向量 垂直
     *
     * 算法：以 半弦 为基准，计算切线向量
     *
     * 圆心 为 O，起点是A，终点是B
     *
     * 以 A 为圆心，半弦长 为半径，画一个圆，和 AO 相交于 点 C
     *
     * |AC| = |AB| / 2
     *
     * 将有向线段 AC 分解到 半弦 和 半弦 垂线上，分别得到下面的 result_dp 和 pp
     */
    pub fn tangents(&self) -> (Vector<f32>, Vector<f32>) {
        let dp = (self.p1 - self.p0).scale(0.5);
        let pp = dp.ortho().scale(-sin2atan(self.d));

        let result_dp = dp.scale(cos2atan(self.d));

        return (
            result_dp + pp,   // 起点 切线向量，注：没有单位化
            result_dp - pp, // 终点 切线向量，注：没有单位化
        );
    }

    /**
     * 将圆弧近似为贝塞尔曲线
     */
    pub fn approximate_bezier(&self, error: &mut ErrorValue) -> Bezier {
        let dp = self.p1 - (self.p0);
        let pp = dp.ortho();

        error.value = dp.norm() * self.d.abs().powf(5.0) / (54. * (1. + self.d * self.d));

        let result_dp = dp.scale((1. - self.d * self.d) / 3.);
        let result_pp = pp.scale(2. * self.d / 3.);

        let p0s = self.p0 + (result_dp) - (result_pp);
        let p1s = self.p1 - (result_dp) - (result_pp);

        return Bezier::new(self.p0, p0s, p1s, self.p1);
    }

    /**
     * 判断 p 是否包含在 圆弧对扇形的夹角内。
     *
     * 包括 圆弧边缘 的 线
     *
     */
    pub fn wedge_contains_point(&self, p: &Point<f32>) -> bool {
        let t = self.tangents();

        if self.d.abs() <= 1. {
            // 小圆弧，夹角 小于等于 PI
            // 在 夹角内，意味着 下面两者 同时成立：
            //     向量 <P0, P> 和 起点切线 成 锐角
            //     向量 <P1, P> 和 终点切线 是 钝角
            return (p - self.p0).dot(&t.0) >= 0.0 && (p - (self.p1)).dot(&t.1) <= 0.0;
        } else {
            // 大圆弧，夹角 大于 PI
            // 如果 点 在 小圆弧 内，那么：下面两者 同时成立
            //     向量 <P0, P> 和 起点切线 成 钝角
            //     向量 <P1, P> 和 终点切线 是 锐角
            // 所以这里要 取反
            return (p - (self.p0)).dot(&t.0) >= 0. || (p - (self.p1)).dot(&t.1) <= 0.;
        }
    }

    /**
     * 计算点到圆弧的距离
     */
    pub fn distance_to_point(&self, p: Point<f32>) -> f32 {
        if self.d.abs() < 1e-5 {
            // d = 0, 当 线段 处理
            let arc_segment = Segment::new(self.p0, self.p1);
            return arc_segment.distance_to_point(p);
        }

        let difference = self.sub(p);

        if self.wedge_contains_point(&p) && self.d.abs() > 1e-5 {
            // 在 夹角内

            // 距离的绝对值 就是 |点到圆心的距离 - 半径|
            // 符号，看 difference 的 neggative
            let v = if difference.negative { -1. } else { 1. };
            return (p.distance_to_point(&self.center()) - self.radius()).abs() * v;
        }

        let d1 = p.squared_distance_to_point(&self.p0);
        let d2 = p.squared_distance_to_point(&self.p1);

        let v1 = if d1 < d2 { d1.sqrt() } else { d2.sqrt() };
        let v2 = if difference.negative { -1.0 } else { 1.0 };

        return v1 * v2;
    }

    /**
     * 计算点到圆弧的平方距离
     */
    pub fn squared_distance_to_point(&self, p: Point<f32>) -> f32 {
        if self.d.abs() < 1e-5 {
            let arc_segment = Segment::new(self.p0, self.p1);
            // 点 到 线段 的 距离 的 平方
            return arc_segment.squared_distance_to_point(&p);
        }

        if self.wedge_contains_point(&p) && self.d.abs() > 1e-5 {
            // 在圆弧的 夹角 里面，sdf = 点到圆心的距离 - 半径
            let answer = p.distance_to_point(&self.center()) - self.radius();
            return answer * answer;
        }

        // 在 夹角外，就是 点 到 啷个端点距离的 最小值
        let d1 = p.squared_distance_to_point(&self.p0);
        let d2 = p.squared_distance_to_point(&self.p1);

        return if d1 < d2 { d1 } else { d2 };
    }

    /**
     * 计算点到圆弧的扩展距离
     */
    pub fn extended_dist(&self, p: &Point<f32>) -> f32 {
        // m 是 P0 P1 的 中点
        let m = self.p0.lerp(&self.p1, 0.5);

        // dp 是 向量 <P0, P1>
        let dp = self.p1 - (self.p0);

        // pp 是 dp 的 正交向量，逆时针
        let pp = dp.ortho();

        // d2 是 圆弧的 圆心角一半 的正切
        let d2 = tan2atan(self.d);

        if (p - m).dot(&(self.p1 - (m))) < 0.0 {
            // 如果 <M, P> 和 <P1, P> 夹角 为 钝角
            // 代表 P 在 直径为 <M, P1> 的 圆内

            // <P0, P> 与 N1 方向的 投影
            // N1 = pp + dp * tan(angle / 2)
            return (p - (self.p0)).dot(&(pp + (dp.scale(d2))).normalize());
        } else {
            // <P1, P> 与 N2 的 点积
            // N2 = pp - dp * tan(angle / 2)
            return (p - (self.p1)).dot(&(pp - (dp.scale(d2))).normalize());
        }
    }

    /**
     * 计算圆弧的包围盒
     * @returns {Array<Point>} 包围盒的顶点数组
     */
    pub fn extents(&self, e: &mut Aabb) {
        e.clear();
        e.add(self.p0);
        e.add(self.p1);

        let c = self.center();
        let r = self.radius();
        let p = [
            c.add_vector(&Vector::new(-1., 0.).scale(r)),
            c.add_vector(&Vector::new(1., 0.).scale(r)),
            c.add_vector(&Vector::new(0., -1.).scale(r)),
            c.add_vector(&Vector::new(0., 1.).scale(r)),
        ];

        for i in 0..4 {
            if self.wedge_contains_point(&p[i]) {
                e.add(p[i]);
            }
        }
    }
}

impl PartialEq for Arc {
    fn eq(&self, other: &Self) -> bool {
        self.p0.equals(&other.p0)
            && self.p1.equals(&other.p1)
            && float_equals(self.d, other.d, None)
    }
}

/**
 * 圆弧 减去 点
 */
pub fn sub_point_from_arc(p: Point<f32>, a: Arc) -> SignedVector {
    return a.sub(p).neg();
}
