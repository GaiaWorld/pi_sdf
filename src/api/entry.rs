/**
 * 跨语言导出入口（wasm 目标生效）
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::base::error::Result;
use crate::model::geom::aabb::Aabb;
use crate::model::outline::contour::Outline;
use crate::model::grid::grid::CellGrid;
use crate::model::raster::sdf::{RasterOptions, SdfTexture, TextureLayout};
use crate::api::entry_inner;

// ── 主链路入口（薄转发到 core）──

/**
 * 由轮廓计算近邻弧网格。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  入参满足各自约束
 *   - ensures   细分必然终止；结果与 core::grid 一致
 *   - 错误      InvalidParam —— scale 非正
 *
 * 参数：outline — 输入轮廓
 *
 * 参数：scale — 细分尺度，正有限值
 *
 * 参数：is_area — 是否按面积图元处理
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_cell_grid(outline: &Outline, scale: f32, is_area: bool) -> Result<CellGrid> {
    entry_inner::compute_cell_grid(outline, scale, is_area)
}

/**
 * 由轮廓范围与参数计算纹理布局。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  入参满足各自约束
 *   - ensures   结果稳定可复现
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_layout(
    extents: &Aabb,
    tex_size: u32,
    pxrange: u32,
    units_per_em: f32,
    offset: u32,
    is_svg: bool,
) -> Result<TextureLayout> {
    entry_inner::compute_layout(extents, tex_size, pxrange, units_per_em, offset, is_svg)
}

/**
 * 按格元将距离场光栅化为 SDF 纹理。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  入参满足各自约束
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价
 *   - 错误      InvalidParam —— 索引越界；Geometry —— 布局与网格不符
 *
 * 参数：grid — 近邻弧网格
 *
 * 参数：layout — 纹理布局
 *
 * 参数：opts — 光栅化选项
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture> {
    entry_inner::rasterize(grid, layout, opts)
}

// ── 字节通道与工具 ──

/**
 * 计算近邻弧网格（字节通道）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用 api → 编码」三步，不含算法；非法字节返回 Err，不 panic
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_near_arcs(bytes: &[u8]) -> Result<Vec<u8>> {
    entry_inner::compute_near_arcs(bytes)
}

/**
 * 计算 SDF 纹理（字节通道）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用 api → 编码」三步，不含算法；非法字节返回 Err，不 panic
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_sdf_tex(bytes: &[u8]) -> Result<Vec<u8>> {
    entry_inner::compute_sdf_tex(bytes)
}

/**
 * 调试：字符的轮廓与近邻弧。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用 api → 编码」三步，不含算法；非法字节返回 Err，不 panic
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn get_char_arc_debug(bytes: &[u8]) -> Result<Vec<u8>> {
    entry_inner::get_char_arc_debug(bytes)
}

/**
 * 调试：SVG 的 SDF 数据。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用 api → 编码」三步，不含算法；非法字节返回 Err，不 panic
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_svg_debug(bytes: &[u8]) -> Result<Vec<u8>> {
    entry_inner::compute_svg_debug(bytes)
}

/**
 * 解压 brotli 压缩流。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  data 为 brotli 压缩字节
 *   - ensures   返回解压后的字节；失败返回 Err，不 panic
 *   - 错误      Decode —— 压缩流非法
 *
 * 参数：data — brotli 压缩字节
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn brotli_decompressor(data: &[u8]) -> Result<Vec<u8>> {
    entry_inner::brotli_decompressor(data)
}
