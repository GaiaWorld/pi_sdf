//! wasm 目标的全局分配器。
//!
//! 由 `platform/wasm/mod.rs` 声明为私有模块；lib.rs 不再内联定义。

// talc 的 ClaimOnOom::new 是 unsafe fn，无法避免此处 unsafe；
// 它隔离在 platform/wasm 内，core 层零 unsafe（REQ-003.1）。
#[global_allocator]
static ALLOCATOR: talc::Talck<talc::locking::AssumeUnlockable, talc::ClaimOnOom> = unsafe {
    static mut MEMORY: [u8; 128 * 1024 * 1024] = [0; 128 * 1024 * 1024];
    let span = talc::Span::from_const_array(std::ptr::addr_of!(MEMORY));
    talc::Talc::new(talc::ClaimOnOom::new(span)).lock()
};
