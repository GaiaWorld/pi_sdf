/**
 * 几何曲线：弧端点、三次贝塞尔曲线、弧（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/geom/curve.rs
 */

use super::{Arc, ArcEndpoint, Bezier};
use crate::model::geom::primitive::{Point, Vector};
use crate::model::geom::aabb::Aabb;
use crate::model::base::error::{Error, Result};

/**
 * 由四个控制点构造贝塞尔。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：p0 — 起点
 * 参数：p1 — 第一控制点
 * 参数：p2 — 第二控制点
 * 参数：p3 — 终点
 */
pub fn bezier_new(p0: Point, p1: Point, p2: Point, p3: Point) -> Bezier {
    Bezier { p0, p1, p2, p3 }
}

/**
 * 贝塞尔曲线上的点。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t — 参数，范围 [0,1]
 */
pub fn bezier_point(b: &Bezier, t: f32) -> Point {
    let p01 = lerp_point(b.p0, b.p1, t);
    let p12 = lerp_point(b.p1, b.p2, t);
    let p23 = lerp_point(b.p2, b.p3, t);
    let p012 = lerp_point(p01, p12, t);
    let p123 = lerp_point(p12, p23, t);
    lerp_point(p012, p123, t)
}

/**
 * 贝塞尔曲线切线。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t — 参数，范围 [0,1]
 */
pub fn bezier_tangent(b: &Bezier, t: f32) -> Vector {
    let mt = 1.0 - t;
    let c0 = 3.0 * mt * mt;
    let c1 = 6.0 * mt * t;
    let c2 = 3.0 * t * t;
    Vector {
        x: c0 * (b.p1.x - b.p0.x) + c1 * (b.p2.x - b.p1.x) + c2 * (b.p3.x - b.p2.x),
        y: c0 * (b.p1.y - b.p0.y) + c1 * (b.p2.y - b.p1.y) + c2 * (b.p3.y - b.p2.y),
    }
}

/**
 * 贝塞尔曲线切线的导数。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t — 参数，范围 [0,1]
 */
pub fn bezier_derivative_tangent(b: &Bezier, t: f32) -> Vector {
    let c0 = 6.0 * (1.0 - t);
    let c1 = 6.0 * t;
    Vector {
        x: c0 * (b.p0.x - 2.0 * b.p1.x + b.p2.x) + c1 * (b.p1.x - 2.0 * b.p2.x + b.p3.x),
        y: c0 * (b.p0.y - 2.0 * b.p1.y + b.p2.y) + c1 * (b.p1.y - 2.0 * b.p2.y + b.p3.y),
    }
}

/**
 * 贝塞尔曲线曲率。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t — 参数，范围 [0,1]
 */
pub fn bezier_curvature(b: &Bezier, t: f32) -> f32 {
    let d1 = bezier_tangent(b, t);
    let d2 = bezier_derivative_tangent(b, t);
    let cross = d1.x * d2.y - d1.y * d2.x;
    let n2 = d1.x * d1.x + d1.y * d1.y;
    let len3 = n2 * n2.sqrt();
    if len3 == 0.0 {
        return 0.0;
    }
    cross / len3
}

/**
 * 在 t 处分割贝塞尔曲线。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t — 分割参数，范围 [0,1]
 */
pub fn bezier_split(b: &Bezier, t: f32) -> (Bezier, Bezier) {
    let p01 = lerp_point(b.p0, b.p1, t);
    let p12 = lerp_point(b.p1, b.p2, t);
    let p23 = lerp_point(b.p2, b.p3, t);
    let p012 = lerp_point(p01, p12, t);
    let p123 = lerp_point(p12, p23, t);
    let p0123 = lerp_point(p012, p123, t);
    (
        Bezier {
            p0: b.p0,
            p1: p01,
            p2: p012,
            p3: p0123,
        },
        Bezier {
            p0: p0123,
            p1: p123,
            p2: p23,
            p3: b.p3,
        },
    )
}

/**
 * 取贝塞尔的子段 [t0, t1]。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 * 参数：t0 — 起始参数
 * 参数：t1 — 结束参数
 */
pub fn bezier_segment(b: &Bezier, t0: f32, t1: f32) -> Bezier {
    if t1 <= 0.0 || t0 >= 1.0 || t0 >= t1 {
        let tp = if t0 >= 1.0 {
            1.0
        } else if t0 <= 0.0 {
            0.0
        } else {
            t0
        };
        let p = bezier_point(b, tp);
        return Bezier {
            p0: p,
            p1: p,
            p2: p,
            p3: p,
        };
    }
    if t0 <= 0.0 {
        return bezier_split(b, t1).0;
    }
    if t1 >= 1.0 {
        return bezier_split(b, t0).1;
    }
    let (left, _right) = bezier_split(b, t1);
    let (_before, sub) = bezier_split(&left, t0 / t1);
    sub
}

/**
 * 贝塞尔曲线中点。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 *
 * 参数：b — 曲线
 */
pub fn bezier_midpoint(b: &Bezier) -> Point {
    bezier_point(b, 0.5)
}

/**
 * 由端点与曲率参数构造弧。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：p0 — 起点
 * 参数：p1 — 终点
 * 参数：d — 曲率参数 = tan(圆心角/4)；0 表示线段
 */
pub fn arc_new(p0: Point, p1: Point, d: f32) -> Arc {
    debug_assert!(d.is_finite());
    Arc { p0, p1, d }
}

/**
 * 弧的半径。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_radius(a: &Arc) -> f32 {
    let dx = a.p1.x - a.p0.x;
    let dy = a.p1.y - a.p0.y;
    let chord = (dx * dx + dy * dy).sqrt();
    let denom = 2.0 * sin2atan(a.d);
    if denom == 0.0 {
        return f32::INFINITY;
    }
    (chord / denom).abs()
}

/**
 * 弧的圆心。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_center(a: &Arc) -> Point {
    let mx = (a.p0.x + a.p1.x) * 0.5;
    let my = (a.p0.y + a.p1.y) * 0.5;
    if a.d == 0.0 {
        return Point { x: mx, y: my };
    }
    let dx = a.p1.x - a.p0.x;
    let dy = a.p1.y - a.p0.y;
    let t = 1.0 / (2.0 * tan2atan(a.d));
    Point {
        x: mx - dy * t,
        y: my + dx * t,
    }
}

/**
 * 弧的圆心角。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_angle(a: &Arc) -> f32 {
    4.0 * a.d.atan().abs()
}

/**
 * 弧长。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_len(a: &Arc) -> f32 {
    if a.d == 0.0 {
        return point_distance(a.p0, a.p1);
    }
    arc_radius(a) * arc_angle(a)
}

/**
 * 点到弧的最短距离。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 * 参数：p — 目标点
 */
pub fn arc_distance_to_point(a: &Arc, p: Point) -> f32 {
    if a.d == 0.0 {
        return point_segment_squared_distance(a.p0.x, a.p0.y, a.p1.x, a.p1.y, p.x, p.y).sqrt();
    }
    if arc_wedge_contains_point(a, p) {
        let c = arc_center(a);
        return (point_distance(p, c) - arc_radius(a)).abs();
    }
    point_distance(p, a.p0).min(point_distance(p, a.p1))
}

/**
 * 点到弧的平方距离。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 * 参数：p — 目标点
 */
pub fn arc_squared_distance_to_point(a: &Arc, p: Point) -> f32 {
    let dist = arc_distance_to_point(a, p);
    dist * dist
}

/**
 * 弧两端点的切线。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_tangents(a: &Arc) -> (Vector, Vector) {
    let dx = (a.p1.x - a.p0.x) * 0.5;
    let dy = (a.p1.y - a.p0.y) * 0.5;
    let sd = -sin2atan(a.d);
    let cd = cos2atan(a.d);
    let ppx = -dy * sd;
    let ppy = dx * sd;
    (
        Vector {
            x: dx * cd + ppx,
            y: dy * cd + ppy,
        },
        Vector {
            x: dx * cd - ppx,
            y: dy * cd - ppy,
        },
    )
}

/**
 * 弧的包围盒。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_extents(a: &Arc) -> Aabb {
    let mins = Point {
        x: a.p0.x.min(a.p1.x),
        y: a.p0.y.min(a.p1.y),
    };
    let maxs = Point {
        x: a.p0.x.max(a.p1.x),
        y: a.p0.y.max(a.p1.y),
    };
    let mut bb = Aabb::new(mins, maxs);
    if a.d == 0.0 {
        return bb;
    }
    let c = arc_center(a);
    let r = arc_radius(a);
    let candidates = [
        Point { x: c.x - r, y: c.y },
        Point { x: c.x + r, y: c.y },
        Point { x: c.x, y: c.y - r },
        Point { x: c.x, y: c.y + r },
    ];
    for q in candidates.iter() {
        if arc_wedge_contains_point(a, *q) {
            bb.extend(*q);
        }
    }
    bb
}

/**
 * 判定点是否落在弧的楔形区域。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 * 参数：p — 目标点
 */
pub fn arc_wedge_contains_point(a: &Arc, p: Point) -> bool {
    if a.d == 0.0 {
        let dx = a.p1.x - a.p0.x;
        let dy = a.p1.y - a.p0.y;
        let l2 = dx * dx + dy * dy;
        if l2 == 0.0 {
            return p.x == a.p0.x && p.y == a.p0.y;
        }
        let t = ((p.x - a.p0.x) * dx + (p.y - a.p0.y) * dy) / l2;
        return t >= 0.0 && t <= 1.0;
    }
    let c = arc_center(a);
    let rpx = p.x - c.x;
    let rpy = p.y - c.y;
    if rpx == 0.0 && rpy == 0.0 {
        return true;
    }
    let r0x = a.p0.x - c.x;
    let r0y = a.p0.y - c.y;
    let ang0 = r0y.atan2(r0x);
    let angp = rpy.atan2(rpx);
    let sigma = if a.d > 0.0 { 1.0 } else { -1.0 };
    let delta = (sigma * (angp - ang0)).rem_euclid(std::f32::consts::TAU);
    delta <= arc_angle(a) + 1e-5
}

/**
 * 以三次贝塞尔近似弧。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 *
 * 参数：a — 弧
 */
pub fn arc_approximate_bezier(a: &Arc) -> Bezier {
    let dx = a.p1.x - a.p0.x;
    let dy = a.p1.y - a.p0.y;
    let d = a.d;
    let rdx = dx * ((1.0 - d * d) / 3.0);
    let rdy = dy * ((1.0 - d * d) / 3.0);
    let rpx = -dy * (2.0 * d / 3.0);
    let rpy = dx * (2.0 * d / 3.0);
    Bezier {
        p0: a.p0,
        p1: Point {
            x: a.p0.x + rdx - rpx,
            y: a.p0.y + rdy - rpy,
        },
        p2: Point {
            x: a.p1.x - rdx - rpx,
            y: a.p1.y - rdy - rpy,
        },
        p3: a.p1,
    }
}

/**
 * 弧转为弧端点对（起点、终点）。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   返回 [起点端点, 终点端点]，d 均为 a.d，tag 一律 None
 *   - 错误      无
 *
 * 参数：a — 弧
 */
pub fn arc_endpoints(a: &Arc) -> Vec<ArcEndpoint> {
    vec![
        ArcEndpoint {
            px: a.p0.x,
            py: a.p0.y,
            d: a.d,
            tag: None,
        },
        ArcEndpoint {
            px: a.p1.x,
            py: a.p1.y,
            d: a.d,
            tag: None,
        },
    ]
}

/**
 * 由弧端点对（起点、终点）还原弧。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  eps 长度为 2；d 为有限值且不为裸 NaN
 *   - ensures   以 eps[0] 为起点、eps[1] 为终点、eps[0].d 为曲率参数构造弧
 *   - 错误      InvalidParam —— eps 长度不为 2
 *
 * 参数：eps — 弧端点对（起、终）
 */
pub fn arc_from_endpoints(eps: Vec<ArcEndpoint>) -> Result<Arc> {
    if eps.len() != 2 {
        return Err(Error::InvalidParam("arc endpoints must be exactly two"));
    }
    debug_assert!(eps[0].d.is_finite() && eps[1].d.is_finite());
    let p0 = Point {
        x: eps[0].px,
        y: eps[0].py,
    };
    let p1 = Point {
        x: eps[1].px,
        y: eps[1].py,
    };
    Ok(Arc {
        p0,
        p1,
        d: eps[0].d,
    })
}

fn lerp_point(a: Point, b: Point, t: f32) -> Point {
    Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn point_distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn point_segment_squared_distance(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / l2).clamp(0.0, 1.0);
    let cx = ax + t * dx - px;
    let cy = ay + t * dy - py;
    cx * cx + cy * cy
}

fn sin2atan(d: f32) -> f32 {
    2.0 * d / (1.0 + d * d)
}

fn cos2atan(d: f32) -> f32 {
    (1.0 - d * d) / (1.0 + d * d)
}

fn tan2atan(d: f32) -> f32 {
    2.0 * d / (1.0 - d * d)
}
