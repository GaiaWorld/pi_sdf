//! 宿主服务层：只提供宿主能力，不含语言绑定

pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
