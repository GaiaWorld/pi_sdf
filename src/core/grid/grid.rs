/**
 * 四叉细分与近邻弧网格
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::geom::curve::Arc;
use crate::core::geom::aabb::Aabb;
use crate::core::outline::contour::Outline;
use crate::core::grid::grid_inner;

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

/**
 * 由轮廓计算近邻弧网格。
 *
 * 契约：API-017
 *
 * 约束：
 *   - requires  scale 为正有限值
 *   - ensures   细分必然终止；结果与现状格元划分一致
 *   - 错误      InvalidParam —— scale 非正
 *
 * 参数：outline — 输入轮廓
 * 参数：scale — 细分尺度，正有限值
 * 参数：is_area — 是否按面积图元处理
 */
pub fn compute_cell_grid(outline: &Outline, scale: f32, is_area: bool) -> Result<CellGrid> {
    grid_inner::compute_cell_grid(outline, scale, is_area)
}
