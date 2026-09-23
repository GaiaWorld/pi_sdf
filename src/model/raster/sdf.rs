/**
 * 纹理布局与 SDF 纹理数据
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::aabb::Aabb;
use crate::model::raster::sdf_inner;

/**
 * 纹理布局：平面边界、图集边界、距离与纹理尺寸。
 *
 * 契约：API-022
 *
 * 约束：
 *   - requires  tex_size > 0；pxrange > 0；units_per_em > 0
 *   - ensures   结果稳定可复现；distance = 每像素距离 × pxrange
 *   - 错误      无（构造由 core::raster::compute_layout 保证）
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
 * 纹理定位信息。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  pixels 长度等于 tex_size 的平方
 *   - ensures   pixels 长度 = tex_size × tex_size
 *   - 错误      无
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
 *   - requires  pixels 长度等于 tex_size 的平方
 *   - ensures   pixels 长度 = tex_size × tex_size
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
 *   - requires  无
 *   - ensures   仅作为参数载体，不产生不变量
 *   - 错误      无
 */
#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct RasterOptions {
    /// 是否按外发光处理
    pub is_outer_glow: bool,
    /// 是否按 SVG 路径处理
    pub is_svg: bool,
    /// 是否反转内外，None 表示按默认
    pub is_reverse: Option<bool>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl SdfTexture {
    /// 像素数量是否与纹理尺寸一致。
    ///
    /// 契约：API-023
    ///
    /// 约束：
    ///   - requires  无
    ///   - ensures   长度等于边长平方时为真
    ///   - 错误      无
    pub fn is_consistent(&self) -> bool {
        sdf_inner::sdf_texture_is_consistent(self)
    }
}
