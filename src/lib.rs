//! pi_sdf2：基于有符号距离场（SDF）的矢量图形库
//!
//! 分层与对外暴露：
//!
//! ```text
//! model   数据层   ── 对外公开（跨语言共用的类型）
//! api     门面层   ── 对外公开（入口函数与有状态类型）
//! core    算法层   ── crate 内部（不对外）
//! platform 宿主层  ── crate 内部（不对外）
//! ```
//!
//! 对外接口只有 **model** 与 **api** 两个模块；core 与 platform 为 `pub(crate)`。
//! 每个 `xx.rs`（冻结壳）都配有私有 `xx_inner.rs`（实现），下游只改后者。

pub mod model;
pub mod api;

pub(crate) mod core;
pub(crate) mod platform;
