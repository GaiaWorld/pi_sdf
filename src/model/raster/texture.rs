/**
 * 单位弧、数据纹理、索引纹理
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::ArcEndpoint;
use crate::model::raster::texture_inner;

/**
 * 单位弧：烘焙时归属某个格元的一条弧记录。
 *
 * 契约：API-019
 *
 * 约束：
 *   - requires  endpoints 非空；sdf_min 不大于 sdf_max
 *   - ensures   距离区间有序
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct UnitArc {
    /// 该单位弧包含的弧端点集合，非空
    pub endpoints: Vec<ArcEndpoint>,
    /// 距离区间下限
    pub sdf_min: f32,
    /// 距离区间上限
    pub sdf_max: f32,
    /// 是否参与渲染
    pub show: bool,
}

/**
 * 数据纹理：存放量化后的弧端点编码。
 *
 * 契约：API-020
 *
 * 约束：
 *   - requires  像素数据长度为宽乘高
 *   - ensures   每条记录可解码为弧端点
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct DataTexture {
    /// 像素数据，每像素 1 字节
    pub pixels: Vec<u8>,
    /// 纹理宽度
    pub width: u32,
    /// 纹理高度
    pub height: u32,
}

/**
 * 索引纹理：存放格元到数据纹理的映射与距离区间。
 *
 * 契约：API-021
 *
 * 约束：
 *   - requires  像素数据长度为宽乘高
 *   - ensures   索引指向有效数据纹理位置
 *   - 错误      无
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct IndexTexture {
    /// 像素数据，每像素 1 字节
    pub pixels: Vec<u8>,
    /// 纹理宽度
    pub width: u32,
    /// 纹理高度
    pub height: u32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl UnitArc {
    /// 距离区间是否有序。
    ///
    /// 契约：API-019
    ///
    /// 约束：
    ///   - requires  无
    ///   - ensures   sdf_min 不大于 sdf_max 时为真
    ///   - 错误      无
    pub fn is_ordered(&self) -> bool {
        texture_inner::unit_arc_is_ordered(self)
    }
}
