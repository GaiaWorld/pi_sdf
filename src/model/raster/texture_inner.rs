/**
 * 单位弧、数据纹理、索引纹理（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/raster/texture.rs
 */

use super::UnitArc;

/**
 * 距离区间是否有序。
 *
 * 契约：API-019
 *
 * 约束：
 *   - requires  无
 *   - ensures   sdf_min 不大于 sdf_max 时为真
 *   - 错误      无
 *
 * 参数：unit — 单位弧
 */
pub fn unit_arc_is_ordered(unit: &UnitArc) -> bool {
    // TODO-DECL —— 实现逻辑：①比较 sdf_min 与 sdf_max ②返回比较结果
    todo!("TODO-DECL")
}
