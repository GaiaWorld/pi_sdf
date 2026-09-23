/**
 * 字体面与字形轮廓提取
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::base::error::Result;
use crate::model::outline::contour::Outline;
use crate::api::font::font_face_inner::FontState;

/**
 * 字体面：解析后的字体与其轮廓提取能力。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct FontFace {
    pub(crate) data: Vec<u8>,
    pub(crate) state: FontState,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl FontFace {
    /// 由字体字节构造字体面。
    ///
    /// 契约：API-024
    ///
    /// 约束：
    ///   - requires  data 为字体文件字节（ttf / otf / ttc）
    ///   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
    ///   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
    ///
    /// 参数：data — 字体文件字节
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        crate::api::font::font_face_inner::font_face_from_bytes(data)
    }

    /// 查字符的字形索引。
    ///
    /// 契约：API-024
    ///
    /// 约束：
    ///   - requires  data 为字体文件字节（ttf / otf / ttc）
    ///   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
    ///   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
    ///
    /// 参数：ch — 目标字符
    pub fn glyph_index(&self, ch: char) -> u32 {
        crate::api::font::font_face_inner::font_face_glyph_index(self, ch)
    }

    /// 取字符的字形轮廓。
    ///
    /// 契约：API-024
    ///
    /// 约束：
    ///   - requires  data 为字体文件字节（ttf / otf / ttc）
    ///   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
    ///   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
    ///
    /// 参数：ch — 目标字符
    pub fn glyph_outline(&self, ch: char) -> Result<Outline> {
        crate::api::font::font_face_inner::font_face_glyph_outline(self, ch)
    }

    /// 按字形索引取轮廓。
    ///
    /// 契约：API-024
    ///
    /// 约束：
    ///   - requires  data 为字体文件字节（ttf / otf / ttc）
    ///   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
    ///   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
    ///
    /// 参数：index — 字形索引
    pub fn outline_of_glyph_index(&self, index: u32) -> Result<Outline> {
        crate::api::font::font_face_inner::font_face_outline_of_glyph_index(self, index)
    }

    /// 每 em 的字体单位数。
    ///
    /// 契约：API-024
    ///
    /// 约束：
    ///   - requires  data 为字体文件字节（ttf / otf / ttc）
    ///   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
    ///   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
    pub fn units_per_em(&self) -> f32 {
        crate::api::font::font_face_inner::font_face_units_per_em(self)
    }
}
