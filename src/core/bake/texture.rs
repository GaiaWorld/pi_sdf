/**
 * 纹理编码（数据纹理与索引纹理）
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::model::base::error::Result;
use crate::model::grid::grid::CellGrid;
use crate::model::raster::texture::{DataTexture, IndexTexture};
use crate::core::bake::arena::ArcArena;
use crate::core::bake::texture_inner;

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
    texture_inner::encode_data_texture(arena)
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
    texture_inner::encode_index_texture(grid, data)
}
