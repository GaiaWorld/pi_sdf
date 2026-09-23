# 上下文

> 仅记录观察到的事实。依据列一律给文件路径或命令输出。

## 技术栈

| 项 | 值 | 依据 |
|---|---|---|
| 语言 / 版本 | Rust，edition 2021，crate 版本 0.1.34 | pi_sdf/Cargo.toml:4-5 |
| 编译器特性 | nightly（feature trait_alias） | pi_sdf/src/lib.rs:1 |
| 运行时目标 | native（Windows / Android）+ wasm32-unknown-unknown | Cargo.toml:35-47；build/*.bat |
| 包管理器 | cargo；Cargo.lock 存在（374 个包条目） | Cargo.lock；命令 Select-String 计数 |
| 产物类型 | cdylib + rlib | Cargo.toml:8 |
| 外部库 | parry2d/nalgebra（几何）、allsorts + ttf-parser（字体）、kurbo/lyon_geom、pi_hash、bitcode、talc、serde | Cargo.toml:11-33 |
| SDF 算法 | 自研 glyphy 模块（弧列表 SDF） | pi_sdf/src/glyphy/ |

## 目录布局

当前工作目录 pi_sdf2（E:\app_new_gui\pi_sdf2）只有 .opencode/，无源码、无 git。
参考源 pi_sdf（E:\app_new_gui\pi_sdf）布局：

```
pi_sdf/
├── src/
│   ├── lib.rs            模块声明、wasm 胶水导出、wasm 全局分配器
│   ├── shape.rs          SVG 图元与 SvgInfo、描边顶点（72KB）
│   ├── utils.rs          OutlineInfo / CellInfo / SDF 光栅化（61KB）
│   ├── font.rs           FontFace 字体解析与轮廓提取
│   ├── svg.rs            Svg 壳，new 忽略入参（死代码）
│   ├── blur.rs           模糊 / 外发光（仅 native）
│   ├── system_font.rs    系统字体（windows / android 各一份）
│   ├── glyphy/           几何 + 弧列表 SDF + 纹理烘焙（blob.rs 49KB、geometry/arc.rs 33KB 等）
│   └── sdf/mod.rs        空文件（0 字节）
├── build/                9 个构建脚本（7 bat + 2 js）
├── wasm/                 前端 index.html / app.js / draw_text.js
├── examples/             11 个示例
├── source/               字体（ttf/woff2）与 GLSL 着色器（fs/vs）素材
├── pkg/ pkg_profiling/   构建产物
└── target/               构建产物
```

## 质量工具

| 工具 | 配置位置 | 现有门槛 |
|---|---|---|
| lint | 无 | 未发现 clippy.toml / .cargo/config |
| 格式 | 无 | 未发现 rustfmt.toml / .rustfmt.toml |
| 类型检查 | cargo check（无 CI 配置） | 未发现 .github/workflows 或 *.yml |
| 测试 | 内联 #[test]，无 tests/ 目录 | grep 命中 5 处有效 #[test]（shape.rs、glyphy/sdf.rs、glyphy/geometry/{arc,aabb}.rs）、4 处被注释；无覆盖率门槛 |
| 工具链固定 | 无 | 未发现 rust-toolchain / rust-toolchain.toml |

## 可复用资产

| 资产 | 位置 | 能做什么 |
|---|---|---|
| pi_sdf 现有实现 | E:\app_new_gui\pi_sdf\src\ | 几何、轮廓、SDF、纹理烘焙的参考实现 |
| glyphy 几何库 | pi_sdf/src/glyphy/ | Arc/Bezier/Aabb/Line/Segment 与弧列表 SDF |
| 字体素材 | pi_sdf/source/*.ttf、*.woff2 | 测试字体（msyh、hwxw、Rubik、SourceHanSans、WenQuanYi） |
| GLSL 着色器素材 | pi_sdf/source/*.fs/*.vs | 与库配套的渲染着色器（不属于本库编译产物） |

## 来源边界

| 来源 | 可达性 | 入口 | 读到了什么 |
|---|---|---|---|
| 内部知识库 / 文档站 | 无 | — | 用户答复：无，使用 crates.io 公开资源 |
| 别的代码库 | 无 | — | 用户答复：无 |
| 内部规范 | 无 | — | 用户答复：无 |

**因不可达而不确定的结论：** 无——用户明确答复不存在工作目录之外的内部来源。

## 组织内已有的方案

| 类别 | 有什么 | 对本次设计的影响 |
|---|---|---|
| 内部库 / 私有制品源 | 有：私有 cargo registry（Cargo.toml 中 registry = yn，指向 ser.yinengyun.com），提供 pi_egl / pi_glow / pi_wgpu / pi_assets | 用户答复：pi_sdf2 保留私有 registry |
| 已有基建 | 无 | — |
| 团队技能 | 用户答复：无 | — |
| 采购 / 供应商约束 | 无 | — |
| 踩过的坑 | 未提供 | — |

## 边界与禁区

- pi_sdf 是已发布 crate（name = pi_sdf，version = 0.1.34；README 标注 crates.io 与 MIT）。既有调用方依赖其 Rust pub API 与 wasm_bindgen 导出的 JS 契约。依据：Cargo.toml:1-5、README.md。
- 其他设计的 targets：Glob **/design/*/00-index.md 无匹配 → 无地盘重叠。
- 构建产物目录：target/、pkg/、pkg_profiling/。依据：目录列举。
- .gitignore 忽略 **/target、**/*.rs.bk、/.cargo、/Cargo.lock；工作树中 Cargo.lock 实际存在。依据：pi_sdf/.gitignore。
- 私有 registry 约定：dev-dependencies 中 winit 与 pi_wgpu 使用 registry = yn。依据：Cargo.toml:56-60。
- 版本控制：pi_sdf 是 git 仓库（git rev-parse 返回 true；最近提交 34cc43d，远端 github.com/GaiaWorld/pi_sdf，分支 master）。pi_sdf2 目录不是 git 仓库（git rev-parse 报 fatal: not a git repository）。用户答复：重构产物将作为 pi_sdf 仓库的新分支纳入 git 管理。

## 惯例

- commit message 中英混合、带 Emoji 式前缀：feat: / # fix:。依据：git log 34cc43d..7507aa6。
- 注释以中文为主，掺杂 JS 风格注释残留（如 // #[test]）。依据：src/glyphy/geometry/vector.rs:1、segment.rs:435。
- 无 lint / format 配置，注释密度高。依据：目录列举 + 质量工具表。
- 默认分支 master。依据：git log 输出。

## 观察到的但未决的事

> 看到苗头、属于后续阶段的问题，D1 起带走。此处只记录，不做判断。

- 范围：用户本轮选择覆盖核心 SDF 管线、system_font、构建脚本 + wasm 前端、examples + 测试；排除 blur.rs 与 svg.rs。
- 对外兼容目标：内部自由重构，对外尽量兼容。
- native 目标平台：Windows + Android（不含 Linux / macOS）。
- 双版本接口组织：用户已选「单 crate 分层 + 边界壳」（core / api / platform）。
- unsafe 处理：全部安全化，但用户要求最终做基准性能测试，性能须与现状一致或更好。
- wasm 调试接口 get_char_arc_debug / compute_svg_debug 在源码中不存在，用户指定需修复（用于调试本库 SDF）。
- pi_sdf2 目录无 git：D7b 落地门禁需先确定冻结基线方案。
- system_font 当前仅 windows / android 有实现；font.rs 无条件调用 FontLoader::new()，在其它 native 平台会编译失败。
- 已确认的功能缺陷候选：blob.rs encode_index_tex 死循环、arc.rs wedge_contains_point 大弧分支失效、outline.rs winding 写入副本、glyphy/sdf.rs 符号 TODO。

## 自检

- [x] 每一条事实都附了依据（文件路径或命令输出）
- [x] 没有出现「应该 / 建议 / 最好」这类表述
- [x] 找不到的项目明确记了「无」
- [x] 已检查其他设计的 targets，无重叠
- [x] 来源边界的询问确实发出去了（不以工作区猜测代替）
- [x] 「来源边界」三类来源的可达性都写明了
- [x] 无不可达来源；已显式记录「无」
- [x] 可达的外部来源：无，未发生整站通读
- [x] 「组织内已有的方案」已单独问过一遍
- [x] 该节未与「可复用资产」混写
- [x] 「观察到的但未决的事」已列出，供 D1 带走
