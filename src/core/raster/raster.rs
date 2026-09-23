/**
 * SDF 纹理光栅化
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::model::base::error::Result;
use crate::model::grid::grid::CellGrid;
use crate::model::raster::sdf::{RasterOptions, SdfTexture, TextureLayout};
use crate::core::raster::raster_inner;

/**
 * 按格元将距离场光栅化为 SDF 纹理。
 *
 * 契约：API-036
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 *
 * 参数：grid — 近邻弧网格
 *
 * 参数：layout — 纹理布局
 *
 * 参数：opts — 光栅化选项
 */
pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture> {
    raster_inner::rasterize(grid, layout, opts)
}
