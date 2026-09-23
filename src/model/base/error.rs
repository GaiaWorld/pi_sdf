/**
 * 统一错误类型
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::base::error_inner;

/**
 * 全库统一的错误类型。
 *
 * 契约：API-002
 *
 * 约束：
 *   - requires  无
 *   - ensures   Display 输出可读短语；构造与格式化均不 panic
 *   - 错误      不适用（Error 是错误载体本身）
 */
#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum Error {
    /// 字体字节非法或损坏
    InvalidFont(&'static str),
    /// 路径动词判别值非法
    InvalidPathVerb(u8),
    /// 参数越界（如纹理尺寸为 0）
    InvalidParam(&'static str),
    /// 边界字节解码失败
    Decode(&'static str),
    /// 编码失败
    Encode(&'static str),
    /// 几何前提不成立（如弧端点为 NaN）
    Geometry(&'static str),
}

/**
 * 全库统一结果类型。
 *
 * 契约：API-002
 *
 * 约束：
 *   - requires  无
 *   - ensures   与错误类型配套，成功为 Ok、失败为 Err
 *   - 错误      不适用（类型别名）
 */
pub type Result<T> = std::result::Result<T, Error>;

/**
 * 错误的可读格式化。
 *
 * 契约：API-002
 *
 * 约束：
 *   - requires  无
 *   - ensures   输出可读短语，不 panic
 *   - 错误      不适用
 */
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_inner::display(self, f)
    }
}

/**
 * 错误的标准错误实现。
 *
 * 契约：API-002
 *
 * 约束：
 *   - requires  无
 *   - ensures   可作为标准错误传播
 *   - 错误      不适用
 */
impl std::error::Error for Error {}
