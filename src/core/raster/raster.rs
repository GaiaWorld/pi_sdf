/**
 * SDF 纹理光栅化
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::geom::aabb::Aabb;
use crate::core::grid::grid::CellGrid;
use crate::core::raster::layout::TextureLayout;
use crate::core::raster::raster_inner;

/**
 * 纹理定位信息。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureInfo {
    /// 平面坐标边界
    pub plane_bounds: Aabb,
    /// 图集坐标边界
    pub atlas_bounds: Aabb,
    /// 水平偏移，用于渲染端定位
    pub sdf_offset_x: f32,
    /// 垂直偏移，用于渲染端定位
    pub sdf_offset_y: f32,
}

/**
 * SDF 纹理：像素数据、纹理尺寸与定位信息。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 */
#[derive(Debug, Clone)]
pub struct SdfTexture {
    /// 像素数据，每像素 1 字节，长度等于 tex_size 的平方
    pub pixels: Vec<u8>,
    /// 纹理边长
    pub tex_size: u32,
    /// 纹理定位信息
    pub tex_info: TextureInfo,
}

/**
 * 光栅化选项。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 */
#[derive(Debug, Clone, Copy)]
pub struct RasterOptions {
    /// 是否按外发光处理
    pub is_outer_glow: bool,
    /// 是否按 SVG 路径处理
    pub is_svg: bool,
    /// 是否反转内外，None 表示按默认
    pub is_reverse: Option<bool>,
}

/**
 * 按格元将距离场光栅化为 SDF 纹理。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 *
 * 参数：grid — 近邻弧网格
 * 参数：layout — 纹理布局
 * 参数：opts — 光栅化选项
 */
pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture> {
    raster_inner::rasterize(grid, layout, opts)
}
