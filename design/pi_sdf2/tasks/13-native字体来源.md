# 13: native 字体来源

**做什么：** 实现文件与系统字体来源，平台相关代码只在 native 后端内。

**关联：** REQ-002.1、REQ-003.1、REQ-004.2

**触及：** `src/platform/native/font_source_inner.rs`、`src/platform/native/system_font_windows.rs`、`src/platform/native/system_font_android.rs`、`tests/platform_native.rs`

**阻塞于：** TASK-10

**状态：** open

- [ ] CASE-062 / 063 / 064 通过
- [ ] 无系统字体实现的平台返回 Err，不导致编译失败
