/**
 * 几何基础图元：点、向量、直线、线段（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/geom/primitive.rs
 */

use super::{Line, Point, Segment, SignedVector, Vector};

/**
 * 由坐标构造点。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：x — x 坐标，有限值
 * 参数：y — y 坐标，有限值
 */
pub fn point_new(x: f32, y: f32) -> Point {
    Point { x, y }
}

/**
 * 点到另一点的距离。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 起点
 * 参数：other — 目标点
 */
pub fn point_distance_to(p: Point, other: Point) -> f32 {
    let dx = p.x - other.x;
    let dy = p.y - other.y;
    (dx * dx + dy * dy).sqrt()
}

/**
 * 点到另一点的平方距离。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 起点
 * 参数：other — 目标点
 */
pub fn point_squared_distance_to(p: Point, other: Point) -> f32 {
    let dx = p.x - other.x;
    let dy = p.y - other.y;
    dx * dx + dy * dy
}

/**
 * 两点中点。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 第一点
 * 参数：other — 第二点
 */
pub fn point_midpoint(p: Point, other: Point) -> Point {
    Point {
        x: (p.x + other.x) * 0.5,
        y: (p.y + other.y) * 0.5,
    }
}

/**
 * 点转向量。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 输入点
 */
pub fn point_to_vector(p: Point) -> Vector {
    Vector { x: p.x, y: p.y }
}

/**
 * 点叠加向量。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 输入点
 * 参数：v — 位移向量
 */
pub fn point_add_vector(p: Point, v: Vector) -> Point {
    Point { x: p.x + v.x, y: p.y + v.y }
}

/**
 * 点到直线的有符号最短距离。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  直线已归一化
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 *
 * 参数：p — 输入点
 * 参数：line — 目标直线
 */
pub fn point_shortest_distance_to_line(p: Point, line: &Line) -> SignedVector {
    let offset = line.n.x * p.x + line.n.y * p.y - line.c;
    let norm_sq = line.n.x * line.n.x + line.n.y * line.n.y;
    if norm_sq == 0.0 {
        return SignedVector {
            vec2: Vector { x: 0.0, y: 0.0 },
            negative: false,
        };
    }
    let d = norm_sq.sqrt();
    let mag = offset.abs() / d;
    SignedVector {
        vec2: Vector {
            x: line.n.x / d * mag,
            y: line.n.y / d * mag,
        },
        negative: offset < 0.0,
    }
}

/**
 * 由分量构造向量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：x — x 分量，有限值
 * 参数：y — y 分量，有限值
 */
pub fn vector_new(x: f32, y: f32) -> Vector {
    Vector { x, y }
}

/**
 * 向量点积。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 * 参数：other — 另一向量
 */
pub fn vector_dot(v: Vector, other: Vector) -> f32 {
    v.x * other.x + v.y * other.y
}

/**
 * 向量二维叉积。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 * 参数：other — 另一向量
 */
pub fn vector_cross(v: Vector, other: Vector) -> f32 {
    v.x * other.y - v.y * other.x
}

/**
 * 向量模长平方。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 */
pub fn vector_norm_squared(v: Vector) -> f32 {
    v.x * v.x + v.y * v.y
}

/**
 * 向量模长。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 */
pub fn vector_norm(v: Vector) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}

/**
 * 向量归一化；零向量原样返回。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 */
pub fn vector_normalized(v: Vector) -> Vector {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len == 0.0 {
        return Vector { x: 0.0, y: 0.0 };
    }
    Vector { x: v.x / len, y: v.y / len }
}

/**
 * 逆时针 90 度正交向量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 */
pub fn vector_ortho(v: Vector) -> Vector {
    Vector { x: -v.y, y: v.x }
}

/**
 * 向量与 x 轴夹角。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 */
pub fn vector_angle(v: Vector) -> f32 {
    v.y.atan2(v.x)
}

/**
 * 在给定基下重写分量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 向量
 * 参数：bx — 基向量 x 分量
 * 参数：by — 基向量 y 分量
 */
pub fn vector_rebase(v: Vector, bx: f32, by: f32) -> Vector {
    Vector {
        x: v.x * bx + v.y * by,
        y: -v.x * by + v.y * bx,
    }
}

/**
 * 由分量与符号构造有符号向量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：x — x 分量，有限值
 * 参数：y — y 分量，有限值
 * 参数：negative — 是否取负方向
 */
pub fn signed_vector_new(x: f32, y: f32, negative: bool) -> SignedVector {
    SignedVector {
        vec2: Vector { x, y },
        negative,
    }
}

/**
 * 由向量与符号构造有符号向量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：v — 方向向量
 * 参数：negative — 是否取负方向
 */
pub fn signed_vector_from_vector(v: Vector, negative: bool) -> SignedVector {
    SignedVector { vec2: v, negative }
}

/**
 * 有符号向量取反。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 *
 * 参数：sv — 输入
 */
pub fn signed_vector_neg(sv: SignedVector) -> SignedVector {
    SignedVector {
        vec2: sv.vec2,
        negative: !sv.negative,
    }
}

/**
 * 由分量构造直线。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 *
 * 参数：a — 法向量 x 分量
 * 参数：b — 法向量 y 分量
 * 参数：c — 常数项
 */
pub fn line_new(a: f32, b: f32, c: f32) -> Line {
    Line {
        n: Vector { x: a, y: b },
        c,
    }
}

/**
 * 由两点构造直线。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 *
 * 参数：p0 — 第一点
 * 参数：p1 — 第二点
 */
pub fn line_from_points(p0: Point, p1: Point) -> Line {
    let n = Vector {
        x: -(p1.y - p0.y),
        y: p1.x - p0.x,
    };
    let c = n.x * p0.x + n.y * p0.y;
    Line { n, c }
}

/**
 * 直线归一化。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 *
 * 参数：line — 输入直线
 */
pub fn line_normalized(line: Line) -> Line {
    let norm_sq = line.n.x * line.n.x + line.n.y * line.n.y;
    if norm_sq == 0.0 {
        return line;
    }
    let d = norm_sq.sqrt();
    Line {
        n: Vector {
            x: line.n.x / d,
            y: line.n.y / d,
        },
        c: line.c / d,
    }
}

/**
 * 两条直线求交。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 *
 * 参数：line — 直线
 * 参数：other — 另一条直线
 */
pub fn line_intersect(line: &Line, other: &Line) -> Option<Point> {
    let det = line.n.x * other.n.y - line.n.y * other.n.x;
    if det == 0.0 {
        return None;
    }
    Some(Point {
        x: (line.c * other.n.y - line.n.y * other.c) / det,
        y: (line.n.x * other.c - line.c * other.n.x) / det,
    })
}

/**
 * 点相对直线的有符号偏移。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 *
 * 参数：line — 直线
 * 参数：p — 目标点
 */
pub fn line_sub(line: &Line, p: &Point) -> SignedVector {
    let offset = line.n.x * p.x + line.n.y * p.y - line.c;
    let norm_sq = line.n.x * line.n.x + line.n.y * line.n.y;
    if norm_sq == 0.0 {
        return SignedVector {
            vec2: Vector { x: 0.0, y: 0.0 },
            negative: false,
        };
    }
    let d = norm_sq.sqrt();
    let mag = offset.abs() / d;
    SignedVector {
        vec2: Vector {
            x: line.n.x / d * mag,
            y: line.n.y / d * mag,
        },
        negative: offset < 0.0,
    }
}

/**
 * 由两端点构造线段。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 *
 * 参数：a — 起点
 * 参数：b — 终点
 */
pub fn segment_new(a: Point, b: Point) -> Segment {
    Segment { a, b }
}

/**
 * 点到线段的距离。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 *
 * 参数：s — 线段
 * 参数：p — 目标点
 */
pub fn segment_distance_to_point(s: &Segment, p: Point) -> f32 {
    let dx = s.b.x - s.a.x;
    let dy = s.b.y - s.a.y;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        let px = p.x - s.a.x;
        let py = p.y - s.a.y;
        return (px * px + py * py).sqrt();
    }
    let t = (((p.x - s.a.x) * dx + (p.y - s.a.y) * dy) / l2).clamp(0.0, 1.0);
    let cx = s.a.x + t * dx - p.x;
    let cy = s.a.y + t * dy - p.y;
    (cx * cx + cy * cy).sqrt()
}

/**
 * 点到线段的平方距离。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 *
 * 参数：s — 线段
 * 参数：p — 目标点
 */
pub fn segment_squared_distance_to_point(s: &Segment, p: Point) -> f32 {
    let dx = s.b.x - s.a.x;
    let dy = s.b.y - s.a.y;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        let px = p.x - s.a.x;
        let py = p.y - s.a.y;
        return px * px + py * py;
    }
    let t = (((p.x - s.a.x) * dx + (p.y - s.a.y) * dy) / l2).clamp(0.0, 1.0);
    let cx = s.a.x + t * dx - p.x;
    let cy = s.a.y + t * dy - p.y;
    cx * cx + cy * cy
}

/**
 * 两条线段上的最近点对。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 *
 * 参数：a — 第一条线段
 * 参数：b — 第二条线段
 */
pub fn segment_nearest_points(a: &Segment, b: &Segment) -> (Point, Point) {
    let ux = a.b.x - a.a.x;
    let uy = a.b.y - a.a.y;
    let vx = b.b.x - b.a.x;
    let vy = b.b.y - b.a.y;
    let wx = a.a.x - b.a.x;
    let wy = a.a.y - b.a.y;

    let aa = ux * ux + uy * uy;
    let ee = vx * vx + vy * vy;
    let ff = vx * wx + vy * wy;

    let mut s;
    let mut t;
    if aa == 0.0 && ee == 0.0 {
        s = 0.0;
        t = 0.0;
    } else if aa == 0.0 {
        s = 0.0;
        t = (ff / ee).clamp(0.0, 1.0);
    } else {
        let cc = ux * wx + uy * wy;
        if ee == 0.0 {
            t = 0.0;
            s = (-cc / aa).clamp(0.0, 1.0);
        } else {
            let bb = ux * vx + uy * vy;
            let denom = aa * ee - bb * bb;
            s = if denom == 0.0 {
                0.0
            } else {
                ((bb * ff - cc * ee) / denom).clamp(0.0, 1.0)
            };
            t = (bb * s + ff) / ee;
            if t < 0.0 {
                t = 0.0;
                s = (-cc / aa).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((bb - cc) / aa).clamp(0.0, 1.0);
            }
        }
    }

    let pa = Point {
        x: a.a.x + ux * s,
        y: a.a.y + uy * s,
    };
    let pb = Point {
        x: b.a.x + vx * t,
        y: b.a.y + vy * t,
    };
    (pa, pb)
}

/**
 * 点是否落在段内。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 *
 * 参数：s — 线段
 * 参数：p — 目标点
 */
pub fn segment_contains_in_span(s: &Segment, p: Point) -> bool {
    let dx = s.b.x - s.a.x;
    let dy = s.b.y - s.a.y;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return p.x == s.a.x && p.y == s.a.y;
    }
    let t = ((p.x - s.a.x) * dx + (p.y - s.a.y) * dy) / l2;
    t >= 0.0 && t <= 1.0
}
