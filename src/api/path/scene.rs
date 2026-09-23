/**
 * SVG 场景
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::geom::aabb::Aabb;
use crate::core::raster::raster::SdfTexture;
use crate::api::path::primitives::Shape;
use crate::api::path::scene_inner;

/**
 * SVG 场景：图元集合与视口。
 *
 * 契约：API-029
 *
 * 约束：
 *   - requires  tex_size、pxrange 为正
 *   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
 *   - 错误      InvalidParam —— 参数非正
 */
#[derive(Debug, Clone)]
pub struct SvgScene {
    /// 视口包围盒
    pub view_box: Aabb,
    shapes: Vec<(u64, Shape)>,
}

impl SvgScene {
    /// 构造空场景。
    ///
    /// 契约：API-029
    ///
    /// 约束：
    ///   - requires  tex_size、pxrange 为正
    ///   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
    ///   - 错误      InvalidParam —— 参数非正
    pub fn new() -> Self {
        scene_inner::scene_new()
    }

    /// 加入一个图元。
    ///
    /// 契约：API-029
    ///
    /// 约束：
    ///   - requires  tex_size、pxrange 为正
    ///   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
    ///   - 错误      InvalidParam —— 参数非正
    ///
    /// 参数：key — 图元标识
    /// 参数：shape — 图形元
    pub fn add(&mut self, key: u64, shape: Shape) {
        scene_inner::scene_add(self, key, shape)
    }

    /// 是否含某标识的图元。
    ///
    /// 契约：API-029
    ///
    /// 约束：
    ///   - requires  tex_size、pxrange 为正
    ///   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
    ///   - 错误      InvalidParam —— 参数非正
    ///
    /// 参数：key — 图元标识
    pub fn has(&self, key: u64) -> bool {
        scene_inner::scene_has(self, key)
    }

    /// 设置视口。
    ///
    /// 契约：API-029
    ///
    /// 约束：
    ///   - requires  tex_size、pxrange 为正
    ///   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
    ///   - 错误      InvalidParam —— 参数非正
    ///
    /// 参数：vb — 视口包围盒
    pub fn set_view_box(&mut self, vb: Aabb) {
        scene_inner::scene_set_view_box(self, vb)
    }

    /// 生成场景内所有图元的 SDF 纹理。
    ///
    /// 契约：API-029
    ///
    /// 约束：
    ///   - requires  tex_size、pxrange 为正
    ///   - ensures   每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序
    ///   - 错误      InvalidParam —— 参数非正
    ///
    /// 参数：tex_size — 纹理边长，正整数
    /// 参数：pxrange — 距离像素范围，正整数
    pub fn layout(&self, tex_size: u32, pxrange: u32) -> Result<Vec<(u64, SdfTexture)>> {
        scene_inner::scene_layout(self, tex_size, pxrange)
    }
}
