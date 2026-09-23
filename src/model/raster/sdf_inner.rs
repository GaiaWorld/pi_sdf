/**
 * 纹理布局与 SDF 纹理数据（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/raster/sdf.rs
 */

use super::SdfTexture;

/**
 * 像素数量是否与纹理尺寸一致。
 *
 * 契约：API-023
 *
 * 约束：
 *   - requires  无
 *   - ensures   长度等于边长平方时为真
 *   - 错误      无
 *
 * 参数：tex — SDF 纹理
 */
pub fn sdf_texture_is_consistent(tex: &SdfTexture) -> bool {
    // 用 u64 比较避免大边长平方在 32 位目标上溢出（对外路径不得 panic）
    let expected = (tex.tex_size as u64) * (tex.tex_size as u64);
    tex.pixels.len() as u64 == expected
}
