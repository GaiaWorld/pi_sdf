/**
 * 格元与近邻弧网格
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::curve::Arc;
use crate::model::geom::aabb::Aabb;
use crate::model::grid::grid_inner;

/**
 * 格元：一个矩形区域及其近邻弧索引。
 *
 * 契约：API-015
 *
 * 约束：
 *   - requires  arc_indices 是全局弧列表的合法下标
 *   - ensures   bounds 非空盒；索引去重且升序
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Cell {
    /// 格元范围，非空盒
    pub bounds: Aabb,
    /// 近邻弧在全局弧列表中的下标，去重且升序
    pub arc_indices: Vec<usize>,
}

/**
 * 近邻弧网格：格元集合与全局弧集合。
 *
 * 契约：API-016
 *
 * 约束：
 *   - requires  cells 的索引均落在 arcs 范围内
 *   - ensures   cells 覆盖 extents；近邻弧对格元内任意点的距离计算充分
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct CellGrid {
    /// 网格整体范围
    pub extents: Aabb,
    /// 全局弧集合
    pub arcs: Vec<Arc>,
    /// 格元集合
    pub cells: Vec<Cell>,
    /// 该字形最小笔画宽度，用于纹理布局
    pub min_width: f32,
    /// 该字形最小笔画高度，用于纹理布局
    pub min_height: f32,
    /// 是否按面积图元处理
    pub is_area: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Cell {
    /// 校验格元的弧索引是否落界。
    ///
    /// 契约：API-015
    ///
    /// 约束：
    ///   - requires  arc_count 为全局弧数量
    ///   - ensures   索引全部落界时为真
    ///   - 错误      无
    ///
    /// 参数：arc_count — 全局弧数量
    pub fn is_valid(&self, arc_count: usize) -> bool {
        grid_inner::cell_is_valid(self, arc_count)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl CellGrid {
    /// 校验网格的索引一致性。
    ///
    /// 契约：API-016
    ///
    /// 约束：
    ///   - requires  无
    ///   - ensures   每个格元的索引均指向有效弧时为真
    ///   - 错误      无
    pub fn is_valid(&self) -> bool {
        grid_inner::cell_grid_is_valid(self)
    }
}
