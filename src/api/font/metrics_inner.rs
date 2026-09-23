/**
 * 字形度量（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/api/font/metrics.rs
 */

use super::GlyphMetrics;
use crate::core::base::error::{Error, Result};
use crate::api::font::font_face::FontFace;

/**
 * 取字符的字形度量。
 *
 * 契约：API-025
 *
 * 约束：
 *   - requires  字符存在于字体中
 *   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
 *   - 错误      InvalidParam —— 字符无对应字形
 *
 * 参数：face — 字体面
 * 参数：ch — 目标字符
 */
pub fn font_glyph_metrics(face: &FontFace, ch: char) -> Result<GlyphMetrics> {
    // TODO-DECL —— 实现逻辑：①校验字形存在 ②求步进 ③求轮廓包围盒与绕向 ④返回度量
    todo!("TODO-DECL")
}

/**
 * 字体上升部高度。
 *
 * 契约：API-025
 *
 * 约束：
 *   - requires  字符存在于字体中
 *   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
 *   - 错误      InvalidParam —— 字符无对应字形
 *
 * 参数：face — 字体面
 */
pub fn font_ascender(face: &FontFace) -> f32 {
    // TODO-DECL —— 实现逻辑：①读 hhea 或 os2 表 ②返回上升部
    todo!("TODO-DECL")
}

/**
 * 字体下降部高度。
 *
 * 契约：API-025
 *
 * 约束：
 *   - requires  字符存在于字体中
 *   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
 *   - 错误      InvalidParam —— 字符无对应字形
 *
 * 参数：face — 字体面
 */
pub fn font_descender(face: &FontFace) -> f32 {
    // TODO-DECL —— 实现逻辑：①读 hhea 或 os2 表 ②返回下降部
    todo!("TODO-DECL")
}
