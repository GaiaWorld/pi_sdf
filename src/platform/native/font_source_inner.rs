/**
 * native 字体来源（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/platform/native/font_source.rs
 */

use super::FontSource;
use crate::model::base::error::{Error, Result};

/**
 * 加载字体字节。
 *
 * 契约：API-030
 *
 * 约束：
 *   - requires  File 变体路径存在；SystemFamily 变体在平台上可解析
 *   - ensures   平台相关代码只在此模块内；无实现的平台返回 Err 而不导致编译失败；读到的字节可直接交给 API-024
 *   - 错误      InvalidFont —— 路径不存在或系统族解析失败
 *
 * 参数：source — 字体来源
 */
pub fn font_source_load(source: &FontSource) -> Result<Vec<u8>> {
    // TODO-DECL —— 实现逻辑：①按变体分派 ②文件走读盘 ③系统族按平台实现查询 ④失败返回 InvalidFont
    todo!("TODO-DECL")
}

/// 当前平台是否有系统字体实现。
///
/// 契约：API-030
///
/// 约束：
///   - requires  File 变体路径存在；SystemFamily 变体在平台上可解析
///   - ensures   平台相关代码只在此模块内；无实现的平台返回 Err 而不导致编译失败；读到的字节可直接交给 API-024
///   - 错误      InvalidFont —— 路径不存在或系统族解析失败
pub fn system_font_available() -> bool {
    // TODO-DECL —— 实现逻辑：①按 cfg 平台判定 ②Windows 与 Android 返回真 ③其余返回假
    todo!("TODO-DECL")
}
