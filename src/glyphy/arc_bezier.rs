/// 计算函数 `d₀ t (1-t)² + d₁ t² (1-t)` 在区间 [0,1] 上的最大绝对值
/// # 参数
/// - `d0`: 第一个系数
/// - `d1`: 第二个系数
/// # 返回值
/// 函数在区间 [0,1] 上的最大绝对值
pub fn approximate_deviation(d0: f32, d1: f32) -> f32 {
    let mut candidates = [0., 1., 0., 0.]; // 候选点数组，初始包含 0 和 1
    let mut num_candidates = 2; // 候选点数量

    // 特殊情况处理：d0 == d1 时，函数在 t=0.5 处可能有极值
    if d0 == d1 {
        candidates[num_candidates] = 0.5;
        num_candidates += 1;
    } else {
        let delta = d0 * d0 - d0 * d1 + d1 * d1; // 判别式

        let t2 = 1.0 / (3.0 * (d0 - d1)); // 分母
        let t0 = (2.0 * d0 - d1) * t2; // 极值点候选

        if delta == 0.0 {
            // 判别式为零，只有一个极值点
            candidates[num_candidates] = t0;
            num_candidates += 1;
        } else if delta > 0.0 {
            // 判别式为正，有两个极值点
            let t1 = delta.sqrt() * t2;
            candidates[num_candidates] = t0 - t1;
            num_candidates += 1;
            candidates[num_candidates] = t0 + t1;
            num_candidates += 1;
        }
    }

    // 遍历所有候选点，计算函数值并取最大值
    let mut e = 0.0;
    for i in 0..num_candidates {
        let t = candidates[i];
        let ee;
        if t < 0. || t > 1. {
            continue; // 忽略不在 [0,1] 范围内的点
        }

        ee = (3. * t * (1. - t) * (d0 * (1. - t) + d1 * t)).abs();
        e = if e > ee { e } else { ee };  // 更新最大值
    }

    return e;
}

/// 计算贝塞尔曲线与圆弧之间的近似误差
/// # 参数
/// - `b0`: 贝塞尔曲线
/// - `a`: 圆弧
/// - `approximate_deviation`: 用于计算偏差的函数
/// # 返回值
/// 贝塞尔曲线与圆弧之间的近似误差
pub fn approximate_bezier_arc_error(
    b0: &Bezier,
    a: &Arc,
    approximate_deviation: fn(f32, f32) -> f32,
) -> f32 {
    // 断言：贝塞尔曲线的起点和终点必须与圆弧的起点和终点一致
    assert!(b0.p0 == a.p0);
    assert!(b0.p3 == a.p1);

    let mut ea = ErrorValue { value: 0.0 };
    let b1 = a.approximate_bezier(&mut ea);
    // 断言：近似后的贝塞尔曲线起点和终点必须与原曲线一致
    assert!(b0.p0 == b1.p0);
    assert!(b0.p3 == b1.p3);
    // 计算控制点的偏差向量
    let mut v0 = b1.p1 - b0.p1;
    let mut v1 = b1.p2 - (b0.p2);

    let b = (b0.p3 - b0.p0).normalize();
    v0 = v0.rebase_other(&b);
    v1 = v1.rebase_other(&b);

    let d1 = approximate_deviation(v0.x, v1.x);
    let d2 = approximate_deviation(v0.y, v1.y);
    let v = Vector::new(d1, d2);

    // 处理特殊情况：圆弧的 d 值过大时，返回弱边界
    if a.d * a.d > 1. - 1e-4 {
        return ea.value + v.norm();
    }

    // 处理特殊情况：控制点不在圆弧的楔形区域内时，返回弱边界
    if !a.wedge_contains_point(&b0.p1) || !a.wedge_contains_point(&b0.p2) {
        return ea.value + v.norm();
    }

    // 处理特殊情况：圆弧接近直线时，返回最大正交偏差
    if a.d.abs() < 1e-6 {
        return ea.value + v.y;
    }

    // 计算圆弧的半角正切值
    let tan_half_alpha = tan2atan(a.d).abs();

    let tan_v = v.x / v.y;

    let mut _eb = 0.0;
    // 如果偏差向量的正切值小于半角正切值，返回弱边界
    if tan_v.abs() <= tan_half_alpha {
        return ea.value + v.norm();
    }

    let c2 = (a.p1 - a.p0).norm() * 0.5;
    let r = a.radius();

    // 计算误差边界
    _eb = Vector::new(c2 + v.x, c2 / tan_half_alpha + v.y).norm() - r;
    // log::debug!("_eb: {}", _eb);
    assert!(_eb >= -0.1);

    return ea.value + _eb;
}

/// 使用中点法简单近似贝塞尔曲线为圆弧
/// # 参数
/// - `b`: 贝塞尔曲线
/// - `error`: 用于存储误差的结构体
/// - `approximate_bezier_arc_error`: 用于计算误差的函数
/// # 返回值
/// 近似后的圆弧
pub fn arc_bezier_approximator_midpoint_simple(
    b: Bezier,
    error: &mut ErrorValue,
    approximate_bezier_arc_error: Box<dyn Fn(Bezier, &Arc) -> f32>,
) -> Arc {
    // 使用贝塞尔曲线的中点生成圆弧
    let a = Arc::from_points(b.p0, b.p3, b.midpoint(), false);

    // 计算误差
    error.value = approximate_bezier_arc_error(b, &a);

    return a;
}

/// 使用中点法将贝塞尔曲线分为两部分并近似为圆弧
/// # 参数
/// - `b`: 贝塞尔曲线
/// - `error`: 用于存储误差的结构体
/// - `mid_t`: 分割参数，默认为 0.5
/// - `approximate_bezier_arc_error`: 用于计算误差的函数
/// # 返回值
/// 近似后的圆弧
pub fn arc_bezier_approximator_midpoint_two_part(
    b: &Bezier,
    error: &mut ErrorValue,
    mid_t: Option<f32>,
    approximate_bezier_arc_error: fn(&Bezier, &Arc) -> f32,
) -> Arc {
    let mid_t = if let Some(v) = mid_t { v } else { 0.5 };

    // 将贝塞尔曲线在 mid_t 处分割为两部分
    let pair = b.split(mid_t);
    let m = pair.1.p0;

    // 分别近似两部分为圆弧
    let a0 = Arc::from_points(b.p0, m, b.p3, true);
    let a1 = Arc::from_points(m, b.p3, b.p0, true);

    // 计算两部分的误差并取最大值
    let e0 = approximate_bezier_arc_error(&pair.0, &a0);
    let e1 = approximate_bezier_arc_error(&pair.1, &a1);
    error.value = if e0 > e1 { e0 } else { e1 };

    // 返回整体的近似圆弧
    return Arc::from_points(b.p0, b.p3, m, false);
}

/// 量化圆弧近似器，用于将贝塞尔曲线近似为量化后的圆弧
pub struct ArcBezierApproximatorQuantized {
    max_d: f32, // 最大 d 值
    d_bits: i32, // 量化位数
}

impl ArcBezierApproximatorQuantized {
    /// 创建一个新的量化圆弧近似器
    /// # 参数
    /// - `_max_d`: 最大 d 值，默认为无穷大
    /// - `_d_bits`: 量化位数，默认为 0
    pub fn new(_max_d: Option<f32>, _d_bits: Option<i32>) -> Self {
        let max_d = if let Some(v) = _max_d {
            v
        } else {
            GLYPHY_INFINITY
        };

        let d_bits = if let Some(v) = _d_bits { v } else { 0 };

        Self { max_d, d_bits }
    }

    /// 将贝塞尔曲线近似为量化后的圆弧
    /// # 参数
    /// - `b`: 贝塞尔曲线
    /// - `error`: 用于存储误差的结构体
    /// - `approximate_bezier_arc_error`: 用于计算误差的函数
    /// # 返回值
    /// 近似后的圆弧
    pub fn approximate_bezier_with_arc(
        &self,
        b: Bezier,
        error: &mut ErrorValue,
        approximate_bezier_arc_error: fn(&Bezier, &Arc) -> f32,
    ) -> Arc {
        let mid_t = 0.5; // 默认分割参数为 0.5
        // log::debug!("b: {:?}", b);
        // 使用中点生成初始圆弧
        let mut a = Arc::from_points(b.p0, b.p3, b.point(mid_t), false);
        let orig_a = a.clone();

        // 如果 max_d 有限，限制圆弧的 d 值
        if self.max_d < f32::INFINITY && self.max_d > -f32::INFINITY {
            assert!(self.max_d >= 0.);
            if a.d.abs() > self.max_d {
                a.d = if a.d < 0.0 { -self.max_d } else { self.max_d };
            }
        }
        // 如果 d_bits 不为零，对 d 值进行量化
        if self.d_bits != 0 && self.max_d != 0. {
            assert!(self.max_d < f32::INFINITY && self.max_d > -f32::INFINITY);
            // if a.d.abs() > self.max_d {
            //     log::debug!("a.d.abs(): {}, self.max_d: {}", a.d.abs(), self.max_d);
            // }
            // log::debug!("a.d.abs(): {}, self.max_d: {}", a.d.abs(), self.max_d);
            assert!(a.d.abs() <= self.max_d);

            let _v = 1;
            let mult = (1 << (self.d_bits - 1) as u32) - 1;
            let id = (a.d / self.max_d * mult as f32).round();
            assert!(-mult as f32 <= id && id <= mult as f32);
            a.d = id * self.max_d / mult as f32;
            assert!(a.d.abs() <= self.max_d);
        }

        // 计算量化引入的误差
        let ed = (a.d - orig_a.d).abs() * (a.p1 - a.p0).norm() * 0.5;

        // 使用中点法近似贝塞尔曲线为圆弧
        arc_bezier_approximator_midpoint_two_part(
            &b,
            error,
            Some(mid_t),
            approximate_bezier_arc_error,
        );

        // 如果量化误差不为零，尝试使用简单近似法
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

/// 默认的最大偏差计算函数
pub use approximate_deviation as max_deviation_approximator_default;

/// 默认的贝塞尔曲线与圆弧误差计算函数
pub fn arc_bezier_error_approximator_default(b0: &Bezier, a: &Arc) -> f32 {
    return approximate_bezier_arc_error(b0, a, max_deviation_approximator_default);
}

/// 默认的贝塞尔曲线近似为圆弧的函数
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
/// 默认的量化圆弧近似器
pub use ArcBezierApproximatorQuantized as ArcBezierApproximatorQuantizedDefault;

use crate::glyphy::geometry::{
    arc::{tan2atan, ErrorValue},
    vector::VectorEXT,
};

use super::{
    geometry::{arc::Arc, bezier::Bezier},
    util::GLYPHY_INFINITY,
};
