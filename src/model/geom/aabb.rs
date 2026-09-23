/**
 * 包围盒
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::aabb_inner;
use crate::model::geom::primitive::{Point, Segment};

/**
 * 边的方向。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum Direction {
    /// 上边
    Top,
    /// 下边
    Bottom,
    /// 左边
    Left,
    /// 右边
    Right,
}

/**
 * 与坐标轴对齐的矩形范围。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Aabb {
    /// 最小点
    pub mins: Point,
    /// 最大点
    pub maxs: Point,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Aabb {
    /// 由两点构造盒。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：mins — 最小点
    /// 参数：maxs — 最大点
    pub fn new(mins: Point, maxs: Point) -> Self {
        aabb_inner::aabb_new(mins, maxs)
    }

    /// 构造空盒。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    pub fn new_invalid() -> Self {
        aabb_inner::aabb_new_invalid()
    }

    /// 是否为空盒。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    pub fn is_empty(&self) -> bool {
        aabb_inner::aabb_is_empty(self)
    }

    /// 扩张以包含一个点。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：p — 待包含的点
    pub fn extend(&mut self, p: Point) {
        aabb_inner::aabb_extend(self, p)
    }

    /// 扩张以包含另一个盒。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：other — 另一个盒
    pub fn extend_by(&mut self, other: &Aabb) {
        aabb_inner::aabb_extend_by(self, other)
    }

    /// 是否包含另一个盒。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：other — 待判定的盒
    pub fn includes(&self, other: &Aabb) -> bool {
        aabb_inner::aabb_includes(self, other)
    }

    /// 宽度。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    pub fn width(&self) -> f32 {
        aabb_inner::aabb_width(self)
    }

    /// 高度。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    pub fn height(&self) -> f32 {
        aabb_inner::aabb_height(self)
    }

    /// 按比例缩放。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：s — 缩放比例，正数
    pub fn scale(&mut self, s: f32) {
        aabb_inner::aabb_scale(self, s)
    }

    /// 沿长边一分为二。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    pub fn half(&self) -> (Aabb, Aabb) {
        aabb_inner::aabb_half(self)
    }

    /// 与另一个盒的交集。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：other — 另一个盒
    pub fn collision(&self, other: &Aabb) -> Option<Aabb> {
        aabb_inner::aabb_collision(self, other)
    }

    /// 取某条边。
    ///
    /// 契约：API-009
    ///
    /// 约束：
    ///   - requires  非空盒满足 mins 分量不大于 maxs 分量
    ///   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
    ///   - 错误      无
    ///
    /// 参数：dir — 边的方向
    pub fn bound(&self, dir: Direction) -> Segment {
        aabb_inner::aabb_bound(self, dir)
    }

}
