/**
 * 几何基础图元：点、向量、直线、线段
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::geom::primitive_inner;

/**
 * 平面点。
 *
 * 契约：API-003
 *
 * 约束：
 *   - requires  坐标有限
 *   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
 *   - 错误      无（坐标非法由上游构造点保证）
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// x 坐标，有限值
    pub x: f32,
    /// y 坐标，有限值
    pub y: f32,
}

impl Point {
    /// 由坐标构造点。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：x — x 坐标，有限值
    /// 参数：y — y 坐标，有限值
    pub fn new(x: f32, y: f32) -> Self {
        primitive_inner::point_new(x, y)
    }

    /// 到另一点的距离。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：other — 目标点
    pub fn distance_to(self, other: Point) -> f32 {
        primitive_inner::point_distance_to(self, other)
    }

    /// 到另一点的平方距离。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：other — 目标点
    pub fn squared_distance_to(self, other: Point) -> f32 {
        primitive_inner::point_squared_distance_to(self, other)
    }

    /// 两点中点。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：other — 另一点
    pub fn midpoint(self, other: Point) -> Point {
        primitive_inner::point_midpoint(self, other)
    }

    /// 转为向量。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    pub fn to_vector(self) -> Vector {
        primitive_inner::point_to_vector(self)
    }

    /// 叠加向量。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  坐标有限
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：v — 位移向量
    pub fn add_vector(self, v: Vector) -> Point {
        primitive_inner::point_add_vector(self, v)
    }

    /// 到直线的最短有符号距离。
    ///
    /// 契约：API-003
    ///
    /// 约束：
    ///   - requires  直线已归一化
    ///   - ensures   distance_to 恒非负；midpoint 位于两点连线中点
    ///   - 错误      无（坐标非法由上游构造点保证）
    ///
    /// 参数：line — 目标直线
    pub fn shortest_distance_to_line(self, line: &Line) -> SignedVector {
        primitive_inner::point_shortest_distance_to_line(self, line)
    }
}

/**
 * 平面向量。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    /// x 分量，有限值
    pub x: f32,
    /// y 分量，有限值
    pub y: f32,
}

/**
 * 带符号的向量：方向向量加一个符号位。
 *
 * 契约：API-004
 *
 * 约束：
 *   - requires  normalized 的输入模长不为 0
 *   - ensures   normalized 结果模长为 1（容差内）
 *   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignedVector {
    /// 方向向量
    pub vec2: Vector,
    /// 符号位，true 表示取负方向
    pub negative: bool,
}

impl Vector {
    /// 由分量构造向量。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：x — x 分量，有限值
    /// 参数：y — y 分量，有限值
    pub fn new(x: f32, y: f32) -> Self {
        primitive_inner::vector_new(x, y)
    }

    /// 点积。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：other — 另一向量
    pub fn dot(self, other: Vector) -> f32 {
        primitive_inner::vector_dot(self, other)
    }

    /// 二维叉积（标量）。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：other — 另一向量
    pub fn cross(self, other: Vector) -> f32 {
        primitive_inner::vector_cross(self, other)
    }

    /// 模长平方。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn norm_squared(self) -> f32 {
        primitive_inner::vector_norm_squared(self)
    }

    /// 模长。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn norm(self) -> f32 {
        primitive_inner::vector_norm(self)
    }

    /// 归一化；零向量返回零向量。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn normalized(self) -> Vector {
        primitive_inner::vector_normalized(self)
    }

    /// 逆时针 90 度正交向量。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn ortho(self) -> Vector {
        primitive_inner::vector_ortho(self)
    }

    /// 与 x 轴夹角。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn angle(self) -> f32 {
        primitive_inner::vector_angle(self)
    }

    /// 在 (bx, by) 基下重写分量。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：bx — 基向量 x 分量
    /// 参数：by — 基向量 y 分量
    pub fn rebase(self, bx: f32, by: f32) -> Vector {
        primitive_inner::vector_rebase(self, bx, by)
    }
}

impl SignedVector {
    /// 由分量与符号构造。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：x — x 分量，有限值
    /// 参数：y — y 分量，有限值
    /// 参数：negative — 是否取负方向
    pub fn new(x: f32, y: f32, negative: bool) -> Self {
        primitive_inner::signed_vector_new(x, y, negative)
    }

    /// 由向量与符号构造。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    ///
    /// 参数：v — 方向向量
    /// 参数：negative — 是否取负方向
    pub fn from_vector(v: Vector, negative: bool) -> Self {
        primitive_inner::signed_vector_from_vector(v, negative)
    }

    /// 取反。
    ///
    /// 契约：API-004
    ///
    /// 约束：
    ///   - requires  normalized 的输入模长不为 0
    ///   - ensures   normalized 结果模长为 1（容差内）
    ///   - 错误      Geometry —— normalized 输入模长为 0 时返回零向量
    pub fn neg(self) -> Self {
        primitive_inner::signed_vector_neg(self)
    }
}

/**
 * 直线：方程 n·x = c。
 *
 * 契约：API-005
 *
 * 约束：
 *   - requires  from_points 的两点不重合
 *   - ensures   normalized 后法向量模长为 1
 *   - 错误      Geometry —— 两点重合时退化为零法向量
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// 法向量
    pub n: Vector,
    /// 常数项
    pub c: f32,
}

impl Line {
    /// 由分量构造直线。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    ///
    /// 参数：a — 法向量 x 分量
    /// 参数：b — 法向量 y 分量
    /// 参数：c — 常数项
    pub fn new(a: f32, b: f32, c: f32) -> Self {
        primitive_inner::line_new(a, b, c)
    }

    /// 由两点构造直线。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    ///
    /// 参数：p0 — 第一点
    /// 参数：p1 — 第二点
    pub fn from_points(p0: Point, p1: Point) -> Self {
        primitive_inner::line_from_points(p0, p1)
    }

    /// 归一化。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    pub fn normalized(self) -> Self {
        primitive_inner::line_normalized(self)
    }

    /// 取法向量引用。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    pub fn normal(&self) -> &Vector {
        &self.n
    }

    /// 与另一条直线求交；平行返回 None。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    ///
    /// 参数：other — 另一条直线
    pub fn intersect(&self, other: &Line) -> Option<Point> {
        primitive_inner::line_intersect(self, other)
    }

    /// 点相对本直线的有符号偏移。
    ///
    /// 契约：API-005
    ///
    /// 约束：
    ///   - requires  from_points 的两点不重合
    ///   - ensures   normalized 后法向量模长为 1
    ///   - 错误      Geometry —— 两点重合时退化为零法向量
    ///
    /// 参数：p — 目标点
    pub fn sub(&self, p: &Point) -> SignedVector {
        primitive_inner::line_sub(self, p)
    }
}

/**
 * 线段。
 *
 * 契约：API-006
 *
 * 约束：
 *   - requires  端点为有限点
 *   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
 *   - 错误      无
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// 起点
    pub a: Point,
    /// 终点
    pub b: Point,
}

impl Segment {
    /// 由两端点构造线段。
    ///
    /// 契约：API-006
    ///
    /// 约束：
    ///   - requires  端点为有限点
    ///   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
    ///   - 错误      无
    ///
    /// 参数：a — 起点
    /// 参数：b — 终点
    pub fn new(a: Point, b: Point) -> Self {
        primitive_inner::segment_new(a, b)
    }

    /// 点到线段的距离。
    ///
    /// 契约：API-006
    ///
    /// 约束：
    ///   - requires  端点为有限点
    ///   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
    ///   - 错误      无
    ///
    /// 参数：p — 目标点
    pub fn distance_to_point(&self, p: Point) -> f32 {
        primitive_inner::segment_distance_to_point(self, p)
    }

    /// 点到线段的平方距离。
    ///
    /// 契约：API-006
    ///
    /// 约束：
    ///   - requires  端点为有限点
    ///   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
    ///   - 错误      无
    ///
    /// 参数：p — 目标点
    pub fn squared_distance_to_point(&self, p: Point) -> f32 {
        primitive_inner::segment_squared_distance_to_point(self, p)
    }

    /// 两条线段上的最近点对。
    ///
    /// 契约：API-006
    ///
    /// 约束：
    ///   - requires  端点为有限点
    ///   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
    ///   - 错误      无
    ///
    /// 参数：a — 第一条线段
    /// 参数：b — 第二条线段
    pub fn nearest_points_on_line_segments(a: &Segment, b: &Segment) -> (Point, Point) {
        primitive_inner::segment_nearest_points(a, b)
    }

    /// 点是否落在段内。
    ///
    /// 契约：API-006
    ///
    /// 约束：
    ///   - requires  端点为有限点
    ///   - ensures   distance_to_point 恒非负且不超过到任一端点的距离
    ///   - 错误      无
    ///
    /// 参数：p — 目标点
    pub fn contains_in_span(&self, p: Point) -> bool {
        primitive_inner::segment_contains_in_span(self, p)
    }
}

