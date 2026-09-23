/**
 * 纹理编码（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/bake/texture.rs
 */

use crate::model::base::error::{Error, Result};
use crate::model::grid::grid::CellGrid;
use crate::model::raster::texture::{DataTexture, IndexTexture};
use crate::core::bake::arena::ArcArena;

/**
 * 把 arena 中的单位弧编码为数据纹理。
 *
 * 契约：API-033
 *
 * 约束：
 *   - requires  arena 中的端点坐标落在量化范围内
 *   - ensures   每条记录可解码为弧端点；不足 3 端点的记录补终止符；量化误差不超过半个量化步长（TERM-013）
 *   - 错误      Encode —— 量化溢出
 *
 * 参数：arena — 单位弧来源
 */
pub fn encode_data_texture(arena: &ArcArena) -> Result<DataTexture> {
    // TODO-DECL —— 实现逻辑：①逐单位弧量化端点 ②编码为像素并按内容去重 ③不足 3 端点补终止符 ④返回纹理
    todo!("TODO-DECL")
}

/**
 * 把近邻弧网格编码为索引纹理。
 *
 * 契约：API-034
 *
 * 约束：
 *   - requires  grid 的每个单位弧已写入 data
 *   - ensures   索引指向有效数据纹理位置；填充循环保证终止
 *   - 错误      Encode —— 偏移超出位宽
 *
 * 参数：grid — 近邻弧网格
 *
 * 参数：data — 已生成的数据纹理
 */
pub fn encode_index_texture(grid: &CellGrid, data: &DataTexture) -> Result<IndexTexture> {
    // TODO-DECL —— 实现逻辑：①打包端点数与距离区间到位域 ②写入格元偏移 ③按固定上界填充（不得用不递减下标）④返回纹理
    todo!("TODO-DECL")
}
