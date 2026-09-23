/**
 * 字形度量
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::base::error::Result;
use crate::model::geom::aabb::Aabb;
use crate::api::font::font_face::FontFace;
use crate::api::font::metrics_inner;

/**
 * 字形度量：步进、包围盒与绕向。
 *
 * 契约：API-025
 *
 * 约束：
 *   - requires  字符存在于字体中
 *   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
 *   - 错误      InvalidParam —— 字符无对应字形
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct GlyphMetrics {
    /// 水平步进，非负
    pub advance: f32,
    /// 字形轮廓包围盒
    pub extents: Aabb,
    /// 轮廓是否为顺时针绕向
    pub is_clockwise: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl FontFace {
    /// 取字符的字形度量。
    ///
    /// 契约：API-025
    ///
    /// 约束：
    ///   - requires  字符存在于字体中
    ///   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
    ///   - 错误      InvalidParam —— 字符无对应字形
    ///
    /// 参数：ch — 目标字符
    pub fn glyph_metrics(&self, ch: char) -> Result<GlyphMetrics> {
        metrics_inner::font_glyph_metrics(self, ch)
    }

    /// 字体上升部高度。
    ///
    /// 契约：API-025
    ///
    /// 约束：
    ///   - requires  字符存在于字体中
    ///   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
    ///   - 错误      InvalidParam —— 字符无对应字形
    pub fn ascender(&self) -> f32 {
        metrics_inner::font_ascender(self)
    }

    /// 字体下降部高度。
    ///
    /// 契约：API-025
    ///
    /// 约束：
    ///   - requires  字符存在于字体中
    ///   - ensures   advance 非负；units_per_em 大于 0；extents 为该字形轮廓包围盒
    ///   - 错误      InvalidParam —— 字符无对应字形
    pub fn descender(&self) -> f32 {
        metrics_inner::font_descender(self)
    }
}
