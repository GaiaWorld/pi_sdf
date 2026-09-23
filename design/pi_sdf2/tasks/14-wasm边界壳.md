# 14: wasm 边界壳

**做什么：** 实现 bitcode 编解码与四个 wasm 导出，并修复前端调试页。

**关联：** REQ-002.2、REQ-004.1、REQ-006.1

**触及：** `src/platform/wasm/exports_inner.rs`、`src/platform/wasm/codec_inner.rs`、`wasm/index.html`、`wasm/app.js`、`wasm/draw_text.js`、`tests/platform_wasm.rs`

**阻塞于：** TASK-10、TASK-11

**状态：** open

- [ ] CASE-065 / 066 / 067 / 068 / 069 / 070 通过
- [ ] 非法字节返回 Err，wasm 模块不 trap（替换 deserialize().unwrap()）
- [ ] get_char_arc_debug 与 compute_svg_debug 可被 JS 调用并返回数据
