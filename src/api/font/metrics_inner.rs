/**
 * 字形度量（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/api/font/metrics.rs
 */

use super::GlyphMetrics;
use crate::model::base::error::{Error, Result};
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
    // ①校验字形存在：cmap 未命中（索引 0）即无对应字形
    let index = face.glyph_index(ch);
    if index == 0 {
        return Err(Error::InvalidParam("字符无对应字形"));
    }
    // ②求轮廓，直接以其包围盒与绕向作为度量（extents 即轮廓包围盒）
    let outline = face.glyph_outline(ch)?;
    let extents = outline.extents();
    let is_clockwise = outline.is_clockwise();
    // ③求非负步进（字体单位；解析异常时兜底为 0）
    let advance = super::font_face_inner::font_horizontal_advance(face, index);
    debug_assert!(advance >= 0.0);
    Ok(GlyphMetrics {
        advance,
        extents,
        is_clockwise,
    })
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
    // ①读 hhea 表 ②返回上升部（字体单位）
    super::font_face_inner::font_ascender_value(face)
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
    // ①读 hhea 表 ②返回下降部（字体单位，通常为负）
    super::font_face_inner::font_descender_value(face)
}
