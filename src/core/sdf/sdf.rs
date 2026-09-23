/**
 * 弧列表距离场
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::model::base::error::Result;
use crate::model::geom::primitive::Point;
use crate::model::geom::curve::Arc;
use crate::core::sdf::sdf_inner;

/**
 * 距离场采样结果。
 *
 * 契约：API-014
 *
 * 约束：
 *   - requires  indices 中每个值均小于 all 的长度
 *   - ensures   distance 符号正确；arc_index 指向产生最小距离的弧；distance 非 NaN
 *   - 错误      InvalidParam —— indices 越界
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SdfSample {
    /// 有符号距离，符号表示点在轮廓内外
    pub distance: f32,
    /// 产生最小距离的弧在输入集合中的下标；集合为空时为 None
    pub arc_index: Option<usize>,
}

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
    sdf_inner::sdf_from_arcs(arcs, p)
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
    sdf_inner::sdf_from_indices(all, indices, p)
}
