//! 基础：错误、常量、数值工具

pub mod error;
mod error_inner;
pub mod consts;
pub mod num;

pub use error::Error;
// consts / num 由 TASK-01 补齐
