/**
 * 统一错误类型（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/base/error.rs
 */

use super::Error;

/**
 * 把错误格式化为可读短语。
 *
 * 契约：API-002
 *
 * 约束：
 *   - requires  无
 *   - ensures   Display 输出可读短语；构造与格式化均不 panic
 *   - 错误      不适用（Error 是错误载体本身）
 *
 * 参数：e — 待格式化的错误
 * 参数：f — 目标格式化器
 */
pub fn display(e: &Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match e {
        Error::InvalidFont(msg) => write!(f, "invalid font: {}", msg),
        Error::InvalidPathVerb(value) => write!(f, "invalid path verb: {}", value),
        Error::InvalidParam(msg) => write!(f, "invalid param: {}", msg),
        Error::Decode(msg) => write!(f, "decode failed: {}", msg),
        Error::Encode(msg) => write!(f, "encode failed: {}", msg),
        Error::Geometry(msg) => write!(f, "geometry error: {}", msg),
    }
}
