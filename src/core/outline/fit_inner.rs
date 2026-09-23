/**
 * 贝塞尔拟合弧与描边几何（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/outline/fit.rs
 */

use super::StrokeMesh;
use crate::model::base::error::{Error, Result};
use crate::model::geom::curve::{Arc, Bezier};

/**
 * 把三次贝塞尔拟合为弧序列。
 *
 * 契约：API-012
 *
 * 约束：
 *   - requires  max_deviation 为正有限值
 *   - ensures   生成弧的首尾端点与贝塞尔一致；每弧中点相对贝塞尔的偏差不超过 max_deviation
 *   - 错误      InvalidParam —— max_deviation 非正
 *
 * 参数：b — 待拟合的贝塞尔曲线
 * 参数：max_deviation — 允许的最大中点偏差，正有限值
 */
pub fn bezier_to_arcs(b: &Bezier, max_deviation: f32) -> Result<Vec<Arc>> {
    // TODO-DECL —— 实现逻辑：①校验容差为正 ②按中点偏差递归二分 ③在容差内求最优弧 ④收集返回
    todo!("TODO-DECL")
}

/**
 * 由轮廓弧序列生成描边网格。
 *
 * 契约：API-013
 *
 * 约束：
 *   - requires  thickness 为正有限值；arcs 非空
 *   - ensures   每条弧生成 4 个顶点、2 个三角形（索引数 = 弧数 × 6）；法线按节点邻接计算
 *   - 错误      InvalidParam —— thickness 非正
 *
 * 参数：arcs — 轮廓弧序列
 * 参数：thickness — 描边厚度，正有限值
 */
pub fn stroke_mesh(arcs: &[Arc], thickness: f32) -> Result<StrokeMesh> {
    // TODO-DECL —— 实现逻辑：①校验厚度为正 ②逐节点求邻接法向 ③按法向扩张四点两三角 ④填充位置/UV/索引
    todo!("TODO-DECL")
}
