/**
 * 四叉细分：由轮廓计算近邻弧网格
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::model::base::error::Result;
use crate::model::outline::contour::Outline;
use crate::model::grid::grid::CellGrid;
use crate::core::grid::grid_inner;

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
