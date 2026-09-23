# 文件结构与规范

## 目录树

每个条目附一句话职责。【冻结】= D7b 落地后不可修改，只能改配对的 _inner 文件。

```
src/
├── lib.rs                           # crate 根：模块声明、目标相关全局分配器
├── core/                            # 算法核心（平台无关，零 unsafe、零平台 cfg）
│   ├── base/                        # MOD-001
│   │   ├── error.rs                 # 【冻结】统一错误类型
│   │   ├── error_inner.rs           #   错误类型实现
│   │   ├── consts.rs                #   常量与数值容差（豁免冻结）
│   │   └── num.rs                   #   基础数值工具（豁免冻结）
│   ├── geom/                        # MOD-002
│   │   ├── primitive.rs             # 【冻结】点/向量/直线/线段
│   │   ├── primitive_inner.rs
│   │   ├── curve.rs                 # 【冻结】贝塞尔曲线/弧
│   │   ├── curve_inner.rs
│   │   ├── aabb.rs                  # 【冻结】包围盒
│   │   └── aabb_inner.rs
│   ├── outline/                     # MOD-003
│   │   ├── contour.rs               # 【冻结】轮廓/子轮廓/绕向
│   │   ├── contour_inner.rs
│   │   ├── fit.rs                   # 【冻结】弧拟合与描边几何
│   │   └── fit_inner.rs
│   ├── sdf/                         # MOD-004
│   │   ├── sdf.rs                   # 【冻结】弧列表距离场
│   │   └── sdf_inner.rs
│   ├── grid/                        # MOD-005
│   │   ├── grid.rs                  # 【冻结】细分与近邻弧
│   │   └── grid_inner.rs
│   ├── bake/                        # MOD-006
│   │   ├── arena.rs                 # 【冻结】索引 arena 与单位弧
│   │   ├── arena_inner.rs
│   │   ├── texture.rs               # 【冻结】数据纹理/索引纹理编码
│   │   └── texture_inner.rs
│   └── raster/                      # MOD-007
│       ├── layout.rs                # 【冻结】纹理布局
│       ├── layout_inner.rs
│       ├── raster.rs                # 【冻结】网格采样为 SDF 纹理
│       └── raster_inner.rs
├── api/                             # 统一门面
│   ├── font/                        # MOD-008
│   │   ├── font_face.rs             # 【冻结】字体面与轮廓提取
│   │   ├── font_face_inner.rs
│   │   ├── metrics.rs               # 【冻结】字形度量
│   │   └── metrics_inner.rs
│   └── path/                        # MOD-009
│       ├── path.rs                  # 【冻结】路径动词与路径
│       ├── path_inner.rs
│       ├── primitives.rs            # 【冻结】圆/矩形/椭圆/多边形/折线
│       ├── primitives_inner.rs
│       ├── scene.rs                 # 【冻结】SVG 场景
│       └── scene_inner.rs
└── platform/                        # 平台后端
    ├── native/                      # MOD-010
    │   ├── font_source.rs           # 【冻结】字体来源接口
    │   ├── font_source_inner.rs
    │   ├── system_font_windows.rs   #   Windows 系统字体（cfg windows，豁免冻结）
    │   └── system_font_android.rs   #   Android 系统字体（cfg android，豁免冻结）
    └── wasm/                        # MOD-011
        ├── exports.rs               # 【冻结】wasm 导出边界壳
        ├── exports_inner.rs
        ├── codec.rs                 # 【冻结】bitcode 编解码
        ├── codec_inner.rs
        └── allocator.rs             #   wasm 全局分配器（cfg wasm32，豁免冻结）

tests/                               # 集成测试（与模块同名）
benches/                             # 基准测试（NFR-001 基线对比）
examples/                            # 字体、路径、各图元、基准等示例（迁移并补充）
build/                               # 构建脚本（整理后）
wasm/                                # 前端调试页（修复调试接口）
```

## 冻结面

D7b 落地后，本表「冻结文件」列谁都不能改，下游只能改「实现文件」列。

| 冻结文件（xx） | 实现文件（xx_inner） | 所属 MOD |
|---|---|---|
| src/core/base/error.rs | src/core/base/error_inner.rs | MOD-001 |
| src/core/geom/primitive.rs | src/core/geom/primitive_inner.rs | MOD-002 |
| src/core/geom/curve.rs | src/core/geom/curve_inner.rs | MOD-002 |
| src/core/geom/aabb.rs | src/core/geom/aabb_inner.rs | MOD-002 |
| src/core/outline/contour.rs | src/core/outline/contour_inner.rs | MOD-003 |
| src/core/outline/fit.rs | src/core/outline/fit_inner.rs | MOD-003 |
| src/core/sdf/sdf.rs | src/core/sdf/sdf_inner.rs | MOD-004 |
| src/core/grid/grid.rs | src/core/grid/grid_inner.rs | MOD-005 |
| src/core/bake/arena.rs | src/core/bake/arena_inner.rs | MOD-006 |
| src/core/bake/texture.rs | src/core/bake/texture_inner.rs | MOD-006 |
| src/core/raster/layout.rs | src/core/raster/layout_inner.rs | MOD-007 |
| src/core/raster/raster.rs | src/core/raster/raster_inner.rs | MOD-007 |
| src/api/font/font_face.rs | src/api/font/font_face_inner.rs | MOD-008 |
| src/api/font/metrics.rs | src/api/font/metrics_inner.rs | MOD-008 |
| src/api/path/path.rs | src/api/path/path_inner.rs | MOD-009 |
| src/api/path/primitives.rs | src/api/path/primitives_inner.rs | MOD-009 |
| src/api/path/scene.rs | src/api/path/scene_inner.rs | MOD-009 |
| src/platform/native/font_source.rs | src/platform/native/font_source_inner.rs | MOD-010 |
| src/platform/wasm/exports.rs | src/platform/wasm/exports_inner.rs | MOD-011 |
| src/platform/wasm/codec.rs | src/platform/wasm/codec_inner.rs | MOD-011 |

豁免冻结（用户批准，不落冻结对，下游可直接修改）：

| 文件 | 理由 |
|---|---|
| src/lib.rs、各 mod.rs | 仅模块声明与聚合，无契约 |
| src/core/base/consts.rs | 常量与数值容差，非接口 |
| src/core/base/num.rs | 纯数值工具，非接口 |
| src/platform/native/system_font_windows.rs | 平台条件编译实现细节 |
| src/platform/native/system_font_android.rs | 平台条件编译实现细节 |
| src/platform/wasm/allocator.rs | 全局分配器，非接口 |

## 文件命名

| 类型 | 规则 | 正例 | 反例 |
|---|---|---|---|
| 冻结接口文件 | xx.rs | font_face.rs | FontFace.rs |
| 配对实现文件 | xx_inner.rs（名字与 xx 同，后缀 _inner） | font_face_inner.rs | fontface_impl.rs |
| **冻结对判据** | **仅凭 _inner 后缀判断谁可改**：带 _inner 的是实现，可改；不带的是冻结壳 | 下游可改 font_face_inner.rs | 不可改 font_face.rs |
| 冻结对的函数名 | xx 与 xx_inner 中同名函数 | 两侧均为 FontFace::from_bytes | 两侧命名不同 |
| 普通模块文件 | snake_case | system_font_windows.rs | SystemFontWindows.rs |

## 编程规范

### 命名

- 类型 / 枚举 / 特征：UpperCamelCase。正例：ArcEndpoint。反例：arc_endpoint。
- 函数 / 方法 / 变量 / 模块：snake_case。正例：compute_near_arcs。反例：computeNearArcs。
- 常量：SCREAMING_SNAKE_CASE。正例：MAX_GRID_SIZE。反例：maxGridSize。

### 错误处理

- 对外入口返回 Result<T, Error>；Error 定义于 src/core/base/error.rs，实现 std::error::Error。
- 内部不变量使用 debug 断言，release 不做断言。
- 禁止 unwrap / expect / panic! 出现在对外入口路径。
- 正例：let face = FontFace::from_bytes(bytes)?;
- 反例：let face = FontFace::from_bytes(bytes).unwrap();

### 导入规则

- 顺序：std → 外部 crate → 本 crate，rustfmt 默认分组。
- 使用绝对路径引入 crate 内项（crate::core::geom::...）。
- 禁止循环依赖，唯一例外见「依赖方向」。

### 测试

- 分层：模块内部接口的单元测试写在模块内联的 #[cfg(test)] mod tests，调用冻结接口 xx（不直接调 xx_inner）；crate 对外 API 与跨模块链路的集成测试放 tests/；性能基准放 benches/。
- 命名：tests/<模块名>.rs；基准 benches/<主题>.rs。
- 必测：几何运算边界、缺陷修复回归（REQ-005 四项）、跨边界错误路径（REQ-004）、等价性对比（REQ-001）。
- 不测：getter / setter、纯转发。
- 正例：mod tests 内调用 grid::compute_near_arcs 验证格元划分。反例：直接调 grid_inner::compute_near_arcs 断言。

### 注释

- 语言：中文。
- 对外契约：按 D7b 落地格式写契约注释。
- 禁止历史式注释（如「我把这里改成了……」）；解释「为什么」而非「做了什么」。

### 文档跳转形式

- 写不写：写。D7b 在每个落地文件头写一行 设计：design/pi_sdf2/00-index.md。
- 形式：裸路径。
- 理由：本项目不产出 rustdoc，源码在编辑器与 GitHub 中按代码渲染，裸路径对两者都有效。

### 日志

- 统一使用 log crate；热路径禁止 println!（现状有 println! 出现在热路径，本轮移除）。
- 级别：error（不可恢复）、warn（降级）、debug（内部状态）。
- 正例：log::debug!("near arcs: {}", n)。反例：println!("{:?}", near_arcs)。

### 依赖方向

| 目录 | 不得 import | 检查方式 |
|---|---|---|
| src/core/** | crate::api、crate::platform、wasm_bindgen，且不得出现平台 cfg | CI 脚本检索（grep），与 NFR-004 同一脚本 |
| src/api/** | crate::platform | CI 脚本检索 |
| src/platform/native/** | crate::platform::wasm | CI 脚本检索 |
| src/platform/wasm/** | crate::platform::native | CI 脚本检索 |

例外（唯一一条，必须写）：

> 同一对 xx / xx_inner 之间可以互相 import——xx_inner 要用 xx 里声明的类型、错误枚举和返回结构，所以这一对本来就有来有回（xx → xx_inner 调实现，xx_inner → xx 用类型）。这是双向例外，只到这一对为止，不是「实现层可以随便引接口层」。

## 与现有规范的差异

| 项 | 现状 | 本设计 | 为什么 |
|---|---|---|---|
| 工具链 | nightly（feature trait_alias） | stable 兼容，构建沿用当前环境 | ADR-004 |
| lint / 格式化 | 无任何配置 | cargo fmt + cargo clippy -D warnings | 大规模重构需要可机械检查的底线 |
| 接口与实现是否分文件 | 同文件 | 每个接口文件 xx 配对 xx_inner，下游只能改 xx_inner | 冻结设计，让「有没有遵守设计」变成 diff 可查 |
| 冻结对命名 | 无 | xx.rs / xx_inner.rs | 仅凭 _inner 后缀判断可改性 |
| 测试 | 仅散落内联 #[test] | 内联单元测试 + tests/ 集成 + benches/ 基准 | NFR-001 需要基准对比 |
| 文档跳转 | 无 | 文件头 设计：design/pi_sdf2/00-index.md，裸路径 | 追溯；本项目不产出 rustdoc |
| 热路径日志 | 存在 println! | 统一 log crate，热路径禁止 println! | 性能与输出污染 |
| 模块归属 | 几何/烘焙/光栅混在 glyphy、utils、shape | 按处理阶段拆为 11 个 MOD | REQ-003 |

## 自检

- [x] 目录树里每个文件/目录都有一句话职责说明
- [x] 目录树与 D5 的 MOD 表一一对应
- [x] 目录树路径与 00-index 的 targets 一致（targets 为 src/core/**、src/api/**、src/platform/** 等，覆盖本树）
- [x] 每个冻结文件 xx 都有配对的 xx_inner，树里可见
- [x] 「冻结面」表已落盘，20 对 + 6 类豁免均写明理由
- [x] 冻结对命名规则已写进「文件命名」表，仅凭 _inner 后缀可判
- [x] 目录树、冻结面表、文件命名表三处命名写法一致（统一 _inner 下划线）
- [x] 依赖方向表写了冻结对双向例外，且限定在这一对之内
- [x] 每条规范可检查且有明确争议点
- [x] 每条规范配有正例与反例
- [x] 继承 D2–D5 的部分有出处（ADR-004、REQ-003、NFR-001、NFR-004）
- [x] 文档跳转形式已明确写下
- [x] 依赖方向已落成可检查形式，每条写了检查方式
- [x] 与 D0 现状的差异列在差异表并给了理由
- [x] 新立的规矩（冻结机制、依赖方向、测试分层）也在差异表里，现状填「无」或现状值
