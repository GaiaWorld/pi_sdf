/**
 * 贝塞尔拟合弧与描边几何
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::geom::curve::{Arc, Bezier};
use crate::core::outline::fit_inner;

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
    fit_inner::bezier_to_arcs(b, max_deviation)
}

/**
 * 描边网格：位置、纹理坐标与索引。
 *
 * 契约：API-013
 *
 * 约束：
 *   - requires  thickness 为正有限值；arcs 非空
 *   - ensures   每条弧生成 4 个顶点、2 个三角形（索引数 = 弧数 × 6）；法线按节点邻接计算
 *   - 错误      InvalidParam —— thickness 非正
 */
#[derive(Debug, Clone)]
pub struct StrokeMesh {
    /// 顶点位置坐标，每顶点 2 个分量
    pub positions: Vec<f32>,
    /// 顶点纹理坐标，每顶点 2 个分量
    pub uvs: Vec<f32>,
    /// 三角形索引，每三角形 3 个
    pub indices: Vec<u16>,
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
    fit_inner::stroke_mesh(arcs, thickness)
}
