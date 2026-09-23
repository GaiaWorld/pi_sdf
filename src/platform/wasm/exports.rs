/**
 * wasm 导出边界壳
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::core::base::error::Result;
use crate::platform::wasm::exports_inner;

/**
 * 计算近邻弧网格（wasm 边界）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用接口层 → 编码」三步，不含算法；非法字节返回 Err，wasm 不 trap
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_near_arcs_of_wasm(bytes: &[u8]) -> Result<Vec<u8>> {
    exports_inner::compute_near_arcs_of_wasm(bytes)
}

/**
 * 计算 SDF 纹理（wasm 边界）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用接口层 → 编码」三步，不含算法；非法字节返回 Err，wasm 不 trap
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_sdf_tex_of_wasm(bytes: &[u8]) -> Result<Vec<u8>> {
    exports_inner::compute_sdf_tex_of_wasm(bytes)
}

/**
 * 调试：字符的轮廓与近邻弧（wasm 边界）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用接口层 → 编码」三步，不含算法；非法字节返回 Err，wasm 不 trap
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn get_char_arc_debug(bytes: &[u8]) -> Result<Vec<u8>> {
    exports_inner::get_char_arc_debug(bytes)
}

/**
 * 调试：SVG 的 SDF 数据（wasm 边界）。
 *
 * 契约：API-031
 *
 * 约束：
 *   - requires  bytes 是由 Codec 编码的合法载荷
 *   - ensures   只做「解码 → 调用接口层 → 编码」三步，不含算法；非法字节返回 Err，wasm 不 trap
 *   - 错误      Decode —— 边界字节无法解析；InvalidParam —— 载荷内字段越界
 *
 * 参数：bytes — 编码后的输入载荷
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_svg_debug(bytes: &[u8]) -> Result<Vec<u8>> {
    exports_inner::compute_svg_debug(bytes)
}
