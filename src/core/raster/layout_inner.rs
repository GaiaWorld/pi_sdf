/**
 * 纹理布局计算（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/raster/layout.rs
 */

use crate::model::base::error::{Error, Result};
use crate::model::geom::aabb::Aabb;
use crate::model::raster::sdf::TextureLayout;

/**
 * 由轮廓范围与参数计算纹理布局。
 *
 * 契约：API-035
 *
 * 约束：
 *   - requires  tex_size > 0；pxrange > 0；units_per_em > 0
 *   - ensures   结果稳定可复现；distance = 每像素距离 × pxrange；tex_size > 0、pxrange > 0、distance > 0（TERM-011、TERM-012）
 *   - 错误      InvalidParam —— 参数非正
 *
 * 参数：extents — 轮廓包围盒
 *
 * 参数：tex_size — 纹理边长，正整数
 *
 * 参数：pxrange — 距离像素范围，正整数
 *
 * 参数：units_per_em — 字体单位，正数
 *
 * 参数：offset — 边距像素数
 *
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
    // TODO-DECL —— 实现逻辑：①校验三个正参数 ②按字形或路径求补方外扩 ③求平面与图集边界 ④算每像素距离与 distance
    todo!("TODO-DECL")
}
