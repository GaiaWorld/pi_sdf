/**
 * 纹理布局
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::geom::aabb::Aabb;
use crate::core::raster::layout_inner;

/**
 * 纹理布局：平面边界、图集边界、距离与纹理尺寸。
 *
 * 契约：API-022
 *
 * 约束：
 *   - requires  tex_size > 0；pxrange > 0；units_per_em > 0
 *   - ensures   结果稳定可复现；distance = 每像素距离 × pxrange
 *   - 错误      InvalidParam —— 参数非正
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureLayout {
    /// 平面坐标边界
    pub plane_bounds: Aabb,
    /// 图集坐标边界
    pub atlas_bounds: Aabb,
    /// 距离值 = 每像素距离 × 距离像素范围
    pub distance: f32,
    /// 纹理边长
    pub tex_size: u32,
}

/**
 * 由轮廓范围与参数计算纹理布局。
 *
 * 契约：API-022
 *
 * 约束：
 *   - requires  tex_size > 0；pxrange > 0；units_per_em > 0
 *   - ensures   结果稳定可复现；distance = 每像素距离 × pxrange
 *   - 错误      InvalidParam —— 参数非正
 *
 * 参数：extents — 轮廓包围盒
 * 参数：tex_size — 纹理边长，正整数
 * 参数：pxrange — 距离像素范围，正整数
 * 参数：units_per_em — 字体单位，正数
 * 参数：offset — 边距像素数
 * 参数：is_svg — 是否 SVG 路径（否则字形）
 */
pub fn compute_layout(
    extents: Aabb,
    tex_size: u32,
    pxrange: u32,
    units_per_em: f32,
    offset: u32,
    is_svg: bool,
) -> Result<TextureLayout> {
    layout_inner::compute_layout(extents, tex_size, pxrange, units_per_em, offset, is_svg)
}
