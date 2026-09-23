/**
 * 弧列表距离场（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/sdf/sdf.rs
 */

use super::SdfSample;
use crate::model::base::error::{Error, Result};
use crate::model::geom::primitive::Point;
use crate::model::geom::curve::Arc;

/**
 * 由完整弧集合计算某点的距离场采样。
 *
 * 契约：API-014
 *
 * 约束：
 *   - requires  indices 中每个值均小于 all 的长度
 *   - ensures   distance 符号正确；arc_index 指向产生最小距离的弧；distance 非 NaN
 *   - 错误      InvalidParam —— indices 越界
 *
 * 参数：arcs — 完整弧集合
 * 参数：p — 目标点
 */
pub fn sdf_from_arcs(arcs: &[Arc], p: Point) -> SdfSample {
    // TODO-DECL —— 实现逻辑：①逐弧求最短距离 ②取最小值与下标 ③按内外定符号返回
    todo!("TODO-DECL")
}

/**
 * 由索引子集计算某点的距离场采样。
 *
 * 契约：API-014
 *
 * 约束：
 *   - requires  indices 中每个值均小于 all 的长度
 *   - ensures   distance 符号正确；arc_index 指向产生最小距离的弧；distance 非 NaN
 *   - 错误      InvalidParam —— indices 越界
 *
 * 参数：all — 完整弧集合
 * 参数：indices — 参与计算的弧下标
 * 参数：p — 目标点
 */
pub fn sdf_from_indices(all: &[Arc], indices: &[usize], p: Point) -> Result<SdfSample> {
    // TODO-DECL —— 实现逻辑：①校验下标范围 ②按下标遍历求最短距离 ③按内外定符号返回
    todo!("TODO-DECL")
}
