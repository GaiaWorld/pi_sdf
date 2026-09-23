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
    // TODO-DECL —— 实现逻辑：①匹配错误变体 ②写入变体名与内部消息 ③返回 Ok(())
    todo!("TODO-DECL")
}
