# 文件结构与规范

## 目录树

每个条目附一句话职责。【冻结】= D7b 落地后不可修改，只能改配对的 _inner 文件。

```
src/
├── lib.rs                           # crate 根：模块声明、可见性、wasm 全局分配器
├── model/                           # 数据层：native 与 wasm 共用；允许导出注解
│   ├── base/                        # MOD-006
│   │   ├── error.rs                 # 【冻结】统一错误类型
│   │   ├── error_inner.rs
│   │   ├── consts.rs                #   常量与数值容差（豁免冻结）
│   │   └── num.rs                   #   基础数值工具（豁免冻结）
│   ├── geom/                        # MOD-001
│   │   ├── primitive.rs             # 【冻结】点/向量/有符号向量/直线/线段
│   │   ├── primitive_inner.rs
│   │   ├── curve.rs                 # 【冻结】弧端点/贝塞尔/弧
│   │   ├── curve_inner.rs
│   │   ├── aabb.rs                  # 【冻结】包围盒与方向
│   │   └── aabb_inner.rs
│   ├── outline/                     # MOD-002
│   │   ├── contour.rs               # 【冻结】子轮廓/轮廓/绕向
│   │   └── contour_inner.rs
│   ├── grid/                        # MOD-003
│   │   ├── grid.rs                  # 【冻结】格元/近邻弧网格
│   │   └── grid_inner.rs
│   ├── raster/                      # MOD-004
│   │   ├── texture.rs               # 【冻结】单位弧/数据纹理/索引纹理
│   │   ├── texture_inner.rs
│   │   ├── sdf.rs                   # 【冻结】纹理布局/定位信息/SDF 纹理/光栅化选项
│   │   └── sdf_inner.rs
│   └── path/                        # MOD-005
│       ├── path.rs                  # 【冻结】路径动词/路径/图形元
│       └── path_inner.rs
├── core/                            # 算法层：禁止导出注解与 unsafe
│   ├── outline/                     # MOD-007
│   │   ├── fit.rs                   # 【冻结】贝塞尔拟合弧/描边几何
│   │   └── fit_inner.rs
│   ├── sdf/                         # MOD-008
│   │   ├── sdf.rs                   # 【冻结】弧列表距离场
│   │   └── sdf_inner.rs
│   ├── grid/                        # MOD-009
│   │   ├── grid.rs                  # 【冻结】四叉细分
│   │   └── grid_inner.rs
│   ├── bake/                        # MOD-010
│   │   ├── arena.rs                 # 【冻结】索引 arena
│   │   ├── arena_inner.rs
│   │   ├── texture.rs               # 【冻结】数据/索引纹理编码
│   │   └── texture_inner.rs
│   └── raster/                      # MOD-011
│       ├── layout.rs                # 【冻结】纹理布局计算
│       ├── layout_inner.rs
│       ├── raster.rs                # 【冻结】SDF 纹理光栅化
│       └── raster_inner.rs
├── api/                             # 门面层：对外入口；允许导出注解
│   ├── font/                        # MOD-012
│   │   ├── font_face.rs             # 【冻结】字体面
│   │   ├── font_face_inner.rs
│   │   ├── metrics.rs               # 【冻结】字形度量
│   │   └── metrics_inner.rs
│   ├── path/                        # MOD-013
│   │   ├── scene.rs                 # 【冻结】SVG 场景
│   │   └── scene_inner.rs
│   ├── entry.rs                     # 【冻结】跨语言导出入口（8 函数）
│   ├── entry_inner.rs
│   ├── codec.rs                     # 【冻结】字节通道编解码
│   └── codec_inner.rs
└── platform/                        # 宿主服务层：不投影到 JS
    ├── native/                      # MOD-014
    │   ├── font_source.rs           # 【冻结】字体来源
    │   ├── font_source_inner.rs
    │   ├── system_font_windows.rs   #   平台实现（豁免冻结）
    │   └── system_font_android.rs   #   平台实现（豁免冻结）
    └── wasm/                        # MOD-015
        └── allocator.rs             #   wasm 分配器（豁免冻结）

tests/                               # 集成测试（与模块同名）
benches/                             # 基准测试（NFR-001 基线对比）
examples/                            # 示例（字体、路径、图元、基准）
build/                               # 构建脚本
wasm/                                # 前端调试页
```

## 契约文件与目录的对应

`plan/07-contracts/` 下的契约文件按**领域**划分，与目录的对应关系如下（同域的数据与算法分列条目）。

| 契约文件 | 对应目录 | 说明 |
|---|---|---|
| `_shared.md` | 无独立目录 | 跨模块共享数据结构（D7 模板规定的例外） |
| `base.md` | src/model/base | |
| `geom.md` | src/model/geom | |
| `outline.md` | src/model/outline + src/core/outline | 数据条目归 model，算法条目归 core |
| `grid.md` | src/model/grid + src/core/grid | 同上 |
| `raster.md` | src/model/raster + src/core/raster | 同上 |
| `bake.md` | src/core/bake | |
| `sdf.md` | src/core/sdf | |
| `font.md` | src/api/font | |
| `path.md` | src/model/path + src/api/path | 数据条目归 model，场景归 api |
| `native.md` | src/platform/native | |
| `entry.md` | src/api/entry.rs、src/api/codec.rs | 文件级入口，无独立目录 |
| （无） | src/platform/wasm | 无接口（仅分配器，豁免冻结），故无契约文件 |

## 冻结面

D7b 落地后，本表「冻结文件」列谁都不能改，下游只能改「实现文件」列。共 22 对。

| 冻结文件（xx） | 实现文件（xx_inner） | 所属 MOD |
|---|---|---|
| src/model/base/error.rs | src/model/base/error_inner.rs | MOD-006 |
| src/model/geom/primitive.rs | src/model/geom/primitive_inner.rs | MOD-001 |
| src/model/geom/curve.rs | src/model/geom/curve_inner.rs | MOD-001 |
| src/model/geom/aabb.rs | src/model/geom/aabb_inner.rs | MOD-001 |
| src/model/outline/contour.rs | src/model/outline/contour_inner.rs | MOD-002 |
| src/model/grid/grid.rs | src/model/grid/grid_inner.rs | MOD-003 |
| src/model/raster/texture.rs | src/model/raster/texture_inner.rs | MOD-004 |
| src/model/raster/sdf.rs | src/model/raster/sdf_inner.rs | MOD-004 |
| src/model/path/path.rs | src/model/path/path_inner.rs | MOD-005 |
| src/core/outline/fit.rs | src/core/outline/fit_inner.rs | MOD-007 |
| src/core/sdf/sdf.rs | src/core/sdf/sdf_inner.rs | MOD-008 |
| src/core/grid/grid.rs | src/core/grid/grid_inner.rs | MOD-009 |
| src/core/bake/arena.rs | src/core/bake/arena_inner.rs | MOD-010 |
| src/core/bake/texture.rs | src/core/bake/texture_inner.rs | MOD-010 |
| src/core/raster/layout.rs | src/core/raster/layout_inner.rs | MOD-011 |
| src/core/raster/raster.rs | src/core/raster/raster_inner.rs | MOD-011 |
| src/api/font/font_face.rs | src/api/font/font_face_inner.rs | MOD-012 |
| src/api/font/metrics.rs | src/api/font/metrics_inner.rs | MOD-012 |
| src/api/path/scene.rs | src/api/path/scene_inner.rs | MOD-013 |
| src/api/entry.rs | src/api/entry_inner.rs | MOD-013 |
| src/api/codec.rs | src/api/codec_inner.rs | MOD-013 |
| src/platform/native/font_source.rs | src/platform/native/font_source_inner.rs | MOD-014 |

豁免冻结（用户批准，不落冻结对，下游可直接修改）：

| 文件 | 理由 |
|---|---|
| src/lib.rs、全部 mod.rs | 仅模块声明与聚合，无契约 |
| src/model/base/consts.rs | 常量与数值容差，非接口 |
| src/model/base/num.rs | 纯数值工具，非接口 |
| src/platform/native/system_font_windows.rs | 平台条件编译实现细节 |
| src/platform/native/system_font_android.rs | 平台条件编译实现细节 |
| src/platform/wasm/allocator.rs | 全局分配器，非接口 |

## 文件命名

| 类型 | 规则 | 正例 | 反例 |
|---|---|---|---|
| 冻结接口文件 | xx.rs | font_face.rs | FontFace.rs |
| 配对实现文件 | xx_inner.rs（模块声明为私有 mod） | font_face_inner.rs | fontface_impl.rs |
| **冻结对判据** | **仅凭 _inner 后缀判断谁可改**；带 _inner 的由 `mod xx_inner;` 私有声明 | 下游可改 font_face_inner.rs | 不可改 font_face.rs |
| 冻结对的函数名 | xx 与 xx_inner 中同名自由函数 | 两侧均为 compute_layout | 两侧命名不同 |
| 普通模块文件 | snake_case | font_source.rs | FontSource.rs |

## 编程规范

### 命名

- 类型 / 枚举 / 特征：UpperCamelCase。正例：ArcEndpoint。反例：arc_endpoint。
- 函数 / 方法 / 变量 / 模块：snake_case。正例：compute_cell_grid。反例：computeCellGrid。
- 常量：SCREAMING_SNAKE_CASE。正例：MAX_GRID_SIZE。反例：maxGridSize。

### 可见性

- 对外只有 `model` 与 `api` 两个 `pub mod`；`core` 与 `platform` 为 `pub(crate) mod`。
- 每个 `xx_inner` 以私有 `mod xx_inner;` 声明，仅同父模块的壳可见。
- 正例：lib.rs 写 `pub mod model; pub mod api; pub(crate) mod core;`。反例：把 core 写成 `pub mod core;`。

### 导出注解

- `#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]` **只允许**出现在 `model` 与 `api` 两层的类型与函数上。
- `core` 与 `platform` **禁止**出现该注解，也禁止 `wasm_bindgen` 依赖。
- 成员级投影（ADR-007）：不可导出的成员（元组返回、借用返回）移入**无注解**的同名 impl 块；`Vec<T>` 字段加 `wasm_bindgen(skip)` 并另加仅 wasm32 定义的同名 `#[wasm_bindgen(getter)]` 方法返回克隆；带数据的 enum 改为 opaque 类 + 静态构造器。
- 正例：src/model/geom/curve.rs 的 `pub struct Arc` 带注解。反例：src/core/sdf/sdf.rs 的函数带注解。

### 错误处理

- 对外入口返回 Result<T, Error>；Error 定义于 src/model/base/error.rs，实现 std::error::Error 与 Into<JsValue>。
- 内部不变量使用 debug 断言，release 不做断言。
- 禁止 unwrap / expect / panic! 出现在对外入口路径。
- 正例：let face = FontFace::from_bytes(bytes)?;
- 反例：let face = FontFace::from_bytes(bytes).unwrap();

### 导入规则

- 顺序：std → 外部 crate → 本 crate，rustfmt 默认分组。
- 使用绝对路径引入 crate 内项（crate::model::...）。
- 禁止循环依赖，唯一例外见「依赖方向」。

### 测试

- 分层：模块内部接口的单元测试写在模块内联的 #[cfg(test)] mod tests，调用冻结接口 xx（不直接调 xx_inner）；crate 对外 API 与跨模块链路的集成测试放 tests/；性能基准放 benches/。
- 命名：tests/<模块名>.rs；基准 benches/<主题>.rs。
- 必测：几何运算边界、缺陷修复回归（REQ-005 四项）、跨边界错误路径（REQ-004）、等价性对比（REQ-001）。
- 不测：getter / setter、纯转发。
- 正例：mod tests 内调用 api::compute_cell_grid 验证网格。反例：直接调 core::grid::grid_inner::compute_cell_grid。

### 注释

- 语言：中文。
- 对外契约：按 D7b 落地格式写契约注释。
- 禁止历史式注释；解释「为什么」而非「做了什么」。

### 文档跳转形式

- 写不写：写。D7b 在每个落地文件头写一行 设计：design/pi_sdf2/00-index.md。
- 形式：裸路径。
- 理由：本项目不产出 rustdoc，源码在编辑器与 GitHub 中按代码渲染，裸路径对两者都有效。

### 日志

- 统一使用 log crate；热路径禁止 println!。
- 级别：error（不可恢复）、warn（降级）、debug（内部状态）。
- 正例：log::debug!("near arcs: {}", n)。反例：println!("{:?}", near_arcs)。

### 依赖方向

| 目录 | 不得 import | 检查方式 |
|---|---|---|
| src/model/** | crate::core、crate::api、crate::platform | CI 脚本检索 |
| src/core/** | crate::api、crate::platform、wasm_bindgen，且不得出现平台 cfg | CI 脚本检索（与 NFR-004 同一脚本） |
| src/api/** | crate::platform | CI 脚本检索 |
| src/platform/native/** | crate::platform::wasm | CI 脚本检索 |
| src/platform/wasm/** | crate::platform::native | CI 脚本检索 |

例外（唯一一条）：

> 同一对 xx / xx_inner 之间可以互相 import——xx_inner 要用 xx 里声明的类型与返回结构，所以这一对本来就有来有回。这是双向例外，只到这一对为止。

## 与现有规范的差异

| 项 | 现状 | 本设计 | 为什么 |
|---|---|---|---|
| 工具链 | nightly（feature trait_alias） | stable 兼容，构建沿用当前环境 | ADR-004 |
| lint / 格式化 | 无任何配置 | cargo fmt + cargo clippy -D warnings | 大规模重构需要可机械检查的底线 |
| 分层 | 几何/烘焙/光栅混在 glyphy、utils、shape | 四层：model / core / api / platform | REQ-003 |
| 对外暴露 | 全部模块 pub | 只有 model 与 api 两个 pub mod | 收敛对外接口 |
| 接口与实现是否分文件 | 同文件 | 每个接口文件 xx 配对 xx_inner（私有 mod），下游只能改 xx_inner | 冻结设计，让「有没有遵守设计」变成 diff 可查 |
| 冻结对命名 | 无 | xx.rs / xx_inner.rs | 仅凭 _inner 后缀判断可改性 |
| 导出注解位置 | 散落在各文件与 lib.rs | 只允许 model 与 api | 保持 core 平台无关（REQ-003.1） |
| 测试 | 仅散落内联 #[test] | 内联单元测试 + tests/ 集成 + benches/ 基准 | NFR-001 需要基准对比 |
| 文档跳转 | 无 | 文件头 设计：design/pi_sdf2/00-index.md，裸路径 | 追溯；本项目不产出 rustdoc |
| 热路径日志 | 存在 println! | 统一 log crate，热路径禁止 println! | 性能与输出污染 |

## 自检

- [x] 目录树里每个文件/目录都有一句话职责说明
- [x] 目录树与 D5 的 MOD 表一一对应（15 个 MOD）
- [x] 目录树路径与 00-index 的 targets 一致
- [x] 每个冻结文件 xx 都有配对的 xx_inner，树里可见（22 对）
- [x] 「冻结面」表已落盘，22 对 + 6 类豁免均写明理由
- [x] 冻结对命名规则已写进「文件命名」表，仅凭 _inner 后缀可判
- [x] 目录树、冻结面表、文件命名表三处命名写法一致
- [x] 依赖方向表写了冻结对双向例外，且限定在这一对之内
- [x] **新增「导出注解」规范**：只允许 model 与 api
- [x] **新增「可见性」规范**：对外只有 model 与 api
- [x] 每条规范可检查且有明确争议点
- [x] 每条规范配有正例与反例
- [x] 文档跳转形式已明确写下
- [x] 依赖方向已落成可检查形式，每条写了检查方式
- [x] 与 D0 现状的差异列在差异表并给了理由
