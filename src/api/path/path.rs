/**
 * 路径
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::outline::contour::Outline;
use crate::core::geom::aabb::Aabb;
use crate::core::raster::raster::SdfTexture;
use crate::api::path::primitives::PathVerb;
use crate::api::path::path_inner;

/**
 * 矢量路径：动词序列与坐标序列。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
 *   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// 路径动词序列
    pub verbs: Vec<PathVerb>,
    /// 路径坐标序列，每点 2 个分量
    pub points: Vec<[f32; 2]>,
}

impl Path {
    /// 由动词字节与扁平坐标构造路径。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    ///
    /// 参数：verbs — 路径动词判别值序列
    /// 参数：points — 扁平坐标序列，每点 2 个分量
    pub fn new(verbs: Vec<u8>, points: Vec<f32>) -> Result<Self> {
        path_inner::path_new(verbs, points)
    }

    /// 由路径动词与扁平坐标构造路径。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    ///
    /// 参数：verbs — 路径动词序列
    /// 参数：points — 扁平坐标序列，每点 2 个分量
    pub fn from_verbs(verbs: Vec<PathVerb>, points: Vec<f32>) -> Result<Self> {
        path_inner::path_from_verbs(verbs, points)
    }

    /// 转为轮廓。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    pub fn to_outline(&self) -> Result<Outline> {
        path_inner::path_to_outline(self)
    }

    /// 生成 SDF 纹理。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    ///
    /// 参数：tex_size — 纹理边长，正整数
    /// 参数：pxrange — 距离像素范围，正整数
    pub fn sdf_texture(&self, tex_size: u32, pxrange: u32) -> Result<SdfTexture> {
        path_inner::path_sdf_texture(self, tex_size, pxrange)
    }

    /// 路径包围盒。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    pub fn extents(&self) -> Aabb {
        path_inner::path_extents(self)
    }

    /// 反转路径方向。
    ///
    /// 契约：API-027
    ///
    /// 约束：
    ///   - requires  points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
    ///   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
    ///   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
    pub fn reverse(&mut self) {
        path_inner::path_reverse(self)
    }
}
