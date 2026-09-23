/**
 * 格元与近邻弧网格（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/grid/grid.rs
 */

use super::{Cell, CellGrid};

/**
 * 校验格元的弧索引是否落界。
 *
 * 契约：API-015
 *
 * 约束：
 *   - requires  arc_count 为全局弧数量
 *   - ensures   索引全部落界时为真
 *   - 错误      无
 *
 * 参数：cell — 格元
 * 参数：arc_count — 全局弧数量
 */
pub fn cell_is_valid(cell: &Cell, arc_count: usize) -> bool {
    // ①遍历 arc_indices ②逐个与 arc_count 比较 ③全落界返回真
    cell.arc_indices.iter().all(|&index| index < arc_count)
}

/**
 * 校验网格的索引一致性。
 *
 * 契约：API-016
 *
 * 约束：
 *   - requires  无
 *   - ensures   每个格元的索引均指向有效弧时为真
 *   - 错误      无
 *
 * 参数：grid — 近邻弧网格
 */
pub fn cell_grid_is_valid(grid: &CellGrid) -> bool {
    // ①取全局弧数量 ②逐格元调用校验 ③全部通过返回真
    let arc_count = grid.arcs.len();
    grid.cells.iter().all(|cell| cell_is_valid(cell, arc_count))
}
