/**
 * native 字体来源
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use std::path::PathBuf;
use crate::model::base::error::Result;
use crate::platform::native::font_source_inner;

/**
 * 字体来源：文件路径或系统字体族。
 *
 * 契约：API-030
 *
 * 约束：
 *   - requires  File 变体路径存在；SystemFamily 变体在平台上可解析
 *   - ensures   平台相关代码只在此模块内；无实现的平台返回 Err 而不导致编译失败；读到的字节可直接交给 API-024
 *   - 错误      InvalidFont —— 路径不存在或系统族解析失败
 */
#[derive(Debug, Clone)]
pub enum FontSource {
    /// 字体文件路径
    File(PathBuf),
    /// 系统字体族名
    SystemFamily(String),
}

impl FontSource {
    /// 加载字体字节。
    ///
    /// 契约：API-030
    ///
    /// 约束：
    ///   - requires  File 变体路径存在；SystemFamily 变体在平台上可解析
    ///   - ensures   平台相关代码只在此模块内；无实现的平台返回 Err 而不导致编译失败；读到的字节可直接交给 API-024
    ///   - 错误      InvalidFont —— 路径不存在或系统族解析失败
    pub fn load(&self) -> Result<Vec<u8>> {
        font_source_inner::font_source_load(self)
    }
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
    font_source_inner::system_font_available()
}
