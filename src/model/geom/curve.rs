/**
 * 几何曲线：弧端点、三次贝塞尔曲线、弧
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::curve_inner;
use crate::model::base::error::Result;
use crate::model::geom::primitive::{Point, Vector};
use crate::model::geom::aabb::Aabb;

/**
 * 弧端点：弧的可传递表示。
 *
 * 契约：API-001
 *
 * 约束：
 *   - requires  p、d 均为有限值
 *   - ensures   可无损转换为弧；tag 仅作去重用途，不影响几何
 *   - 错误      无（纯数据，构造不失败）
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct ArcEndpoint {
    /// 端点 x 坐标，有限值
    pub px: f32,
    /// 端点 y 坐标，有限值
    pub py: f32,
    /// 曲率参数 = tan(圆心角/4)，有限值；0 表示线段
    pub d: f32,
    /// 线段共享键，仅线段使用；其余为 None（JS 侧为 BigInt，无损）
    pub tag: Option<u64>,
}

/**
 * 三次贝塞尔曲线。
 *
 * 契约：API-007
 *
 * 约束：
 *   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
 *   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
 *   - 错误      Geometry —— segment 分母退化时返回退化曲线
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Bezier {
    /// 起点
    pub p0: Point,
    /// 第一控制点
    pub p1: Point,
    /// 第二控制点
    pub p2: Point,
    /// 终点
    pub p3: Point,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Bezier {
    /// 由四个控制点构造。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：p0 — 起点
    /// 参数：p1 — 第一控制点
    /// 参数：p2 — 第二控制点
    /// 参数：p3 — 终点
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        curve_inner::bezier_new(p0, p1, p2, p3)
    }

    /// 曲线上的点。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t — 参数，范围 [0,1]
    pub fn point(&self, t: f32) -> Point {
        curve_inner::bezier_point(self, t)
    }

    /// 曲线切线。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t — 参数，范围 [0,1]
    pub fn tangent(&self, t: f32) -> Vector {
        curve_inner::bezier_tangent(self, t)
    }

    /// 曲线切线的导数。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t — 参数，范围 [0,1]
    pub fn derivative_tangent(&self, t: f32) -> Vector {
        curve_inner::bezier_derivative_tangent(self, t)
    }

    /// 曲线曲率。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t — 参数，范围 [0,1]
    pub fn curvature(&self, t: f32) -> f32 {
        curve_inner::bezier_curvature(self, t)
    }

    /// 取子段 [t0, t1]。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t0 — 起始参数
    /// 参数：t1 — 结束参数
    pub fn segment(&self, t0: f32, t1: f32) -> Bezier {
        curve_inner::bezier_segment(self, t0, t1)
    }

    /// 曲线中点（t = 0.5）。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    pub fn midpoint(&self) -> Point {
        curve_inner::bezier_midpoint(self)
    }
}

// wasm_bindgen 不支持元组返回，故本块不加导出注解（native 目标仍全量可用）。
impl Bezier {
    /// 在 t 处分割为两段。
    ///
    /// 契约：API-007
    ///
    /// 约束：
    ///   - requires  t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
    ///   - ensures   split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
    ///   - 错误      Geometry —— segment 分母退化时返回退化曲线
    ///
    /// 参数：t — 分割参数，范围 [0,1]
    pub fn split(&self, t: f32) -> (Bezier, Bezier) {
        curve_inner::bezier_split(self, t)
    }
}

/**
 * 圆弧。
 *
 * 契约：API-008
 *
 * 约束：
 *   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
 *   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
 *   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Arc {
    /// 起点
    pub p0: Point,
    /// 终点
    pub p1: Point,
    /// 曲率参数 = tan(圆心角/4)；0 表示线段
    pub d: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Arc {
    /// 由端点与曲率参数构造弧。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    ///
    /// 参数：p0 — 起点
    /// 参数：p1 — 终点
    /// 参数：d — 曲率参数 = tan(圆心角/4)；0 表示线段
    pub fn new(p0: Point, p1: Point, d: f32) -> Self {
        curve_inner::arc_new(p0, p1, d)
    }

    /// 圆周半径。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn radius(&self) -> f32 {
        curve_inner::arc_radius(self)
    }

    /// 圆心。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn center(&self) -> Point {
        curve_inner::arc_center(self)
    }

    /// 圆心角。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn angle(&self) -> f32 {
        curve_inner::arc_angle(self)
    }

    /// 弧长。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn len(&self) -> f32 {
        curve_inner::arc_len(self)
    }

    /// 点到弧的最短距离。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    ///
    /// 参数：p — 目标点
    pub fn distance_to_point(&self, p: Point) -> f32 {
        curve_inner::arc_distance_to_point(self, p)
    }

    /// 点到弧的平方距离。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    ///
    /// 参数：p — 目标点
    pub fn squared_distance_to_point(&self, p: Point) -> f32 {
        curve_inner::arc_squared_distance_to_point(self, p)
    }

    /// 弧的包围盒。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn extents(&self) -> Aabb {
        curve_inner::arc_extents(self)
    }

    /// 判定点是否落在弧的楔形区域。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    ///
    /// 参数：p — 目标点
    pub fn wedge_contains_point(&self, p: Point) -> bool {
        curve_inner::arc_wedge_contains_point(self, p)
    }

    /// 以三次贝塞尔近似本弧。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn approximate_bezier(&self) -> Bezier {
        curve_inner::arc_approximate_bezier(self)
    }

    /// 转为可传递的弧端点对（起点、终点）。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   返回 [起点端点, 终点端点]，d 均为本弧曲率参数；tag 一律 None
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn endpoints(&self) -> Vec<ArcEndpoint> {
        curve_inner::arc_endpoints(self)
    }

    /// 由弧端点对（起点、终点）还原弧。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  eps 长度为 2；d 为有限值且不为裸 NaN
    ///   - ensures   以 eps[0] 为起点、eps[1] 为终点、eps[0].d 为曲率参数构造弧
    ///   - 错误      InvalidParam —— eps 长度不为 2
    ///
    /// 参数：eps — 弧端点对（起、终）
    pub fn from_endpoints(eps: Vec<ArcEndpoint>) -> Result<Self> {
        curve_inner::arc_from_endpoints(eps)
    }
}

// wasm_bindgen 不支持元组返回，故本块不加导出注解（native 目标仍全量可用）。
impl Arc {
    /// 两端点切线。
    ///
    /// 契约：API-008
    ///
    /// 约束：
    ///   - requires  p0 与 p1 不重合；d 为有限值且不为裸 NaN
    ///   - ensures   d=0 时等价线段；大弧与小弧的包含判定均与几何定义一致
    ///   - 错误      Geometry —— p0 与 p1 重合时返回退化弧
    pub fn tangents(&self) -> (Vector, Vector) {
        curve_inner::arc_tangents(self)
    }
}
