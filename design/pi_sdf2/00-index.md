# pi_sdf2 设计索引

## 状态

| 项 | 值 |
|---|---|
| 模式 | 完整模式 |
| 当前阶段 | 已完成 |
| 最后更新 | 2026-09-22 |
| 跳过阶段 | |

## targets

- src/core/**
- src/api/**
- src/platform/**
- src/lib.rs
- Cargo.toml
- .gitignore
- tests/**
- benches/**
- examples/**
- build/**
- wasm/**

## 批次进度（仅拆分形态）

| 批次 | 模块文件（批内） | 接口数 | 依赖前批 | 契约确认 | 落地 | 落地提交 |
|---|---|---|---|---|---|---|
| 0 | _shared.md base.md geom.md | 9 | — | ✔ | ✔ | d2773ef |
| 1 | outline.md sdf.md grid.md | 8 | 0 | ✔ | ✔ | d2773ef |
| 2 | bake.md raster.md | 6 | 0、1 | ✔ | ✔ | d2773ef |
| 3 | font.md path.md | 6 | 0、1 | ✔ | ✔ | d2773ef |
| 4 | native.md wasm.md | 3 | 3 | ✔ | ✔ | d2773ef |

## 术语引用

| 来源设计 | 引用的术语 | 快照日期 |
|---|---|---|

## ID 总表

### 需求

| ID | 标题 | 定义于 | 状态 |
|---|---|---|---|
| REQ-001 | 功能等价 | spec/02-requirements.md | open |
| REQ-002 | 统一接口同时适配 native 与 wasm | spec/02-requirements.md | open |
| REQ-003 | 模块边界清晰且可机械验证 | spec/02-requirements.md | open |
| REQ-004 | 不可信输入不导致崩溃 | spec/02-requirements.md | open |
| REQ-005 | 已确认缺陷修复 | spec/02-requirements.md | open |
| REQ-006 | wasm 调试链路可用 | spec/02-requirements.md | open |

### 非功能需求

| ID | 标题 | 定义于 | 状态 |
|---|---|---|---|
| NFR-001 | 性能不退化 | spec/02-requirements.md | open |
| NFR-002 | 内存安全 | spec/02-requirements.md | open |
| NFR-003 | 平台覆盖 | spec/02-requirements.md | open |
| NFR-004 | 边界可机械验证 | spec/02-requirements.md | open |

### 词汇

| ID | 术语 | 定义于 |
|---|---|---|
| TERM-001 | 轮廓 Outline | spec/03-glossary.md |
| TERM-002 | 子轮廓 Contour | spec/03-glossary.md |
| TERM-003 | 弧 Arc | spec/03-glossary.md |
| TERM-004 | 弧端点 ArcEndpoint | spec/03-glossary.md |
| TERM-005 | 包围盒 Aabb | spec/03-glossary.md |
| TERM-006 | 有符号距离场 SDF | spec/03-glossary.md |
| TERM-007 | 格元 Cell | spec/03-glossary.md |
| TERM-008 | 近邻弧 Near Arcs | spec/03-glossary.md |
| TERM-009 | 近邻弧网格 Cell Grid | spec/03-glossary.md |
| TERM-010 | SDF 纹理 SDF Texture | spec/03-glossary.md |
| TERM-011 | 距离像素范围 PxRange | spec/03-glossary.md |
| TERM-012 | 纹理尺寸 TexSize | spec/03-glossary.md |
| TERM-013 | 量化 Quantize | spec/03-glossary.md |
| TERM-014 | 烘焙 Bake | spec/03-glossary.md |
| TERM-015 | 数据纹理 Data Texture | spec/03-glossary.md |
| TERM-016 | 索引纹理 Index Texture | spec/03-glossary.md |
| TERM-017 | 单位弧 Unit Arc | spec/03-glossary.md |
| TERM-018 | 字形 Glyph | spec/03-glossary.md |
| TERM-019 | 字形步进 Advance | spec/03-glossary.md |
| TERM-020 | 字体单位 UnitsPerEm | spec/03-glossary.md |
| TERM-021 | 平台后端 Platform Backend | spec/03-glossary.md |
| TERM-022 | 算法核心 Algorithm Core | spec/03-glossary.md |
| TERM-023 | 边界壳 Boundary Shell | spec/03-glossary.md |
| TERM-024 | ArcArena 弧数据索引 arena | spec/03-glossary.md |
| TERM-025 | FontFace 字体面 | spec/03-glossary.md |
| TERM-026 | GlyphMetrics 字形度量 | spec/03-glossary.md |
| TERM-027 | Path 路径 | spec/03-glossary.md |
| TERM-028 | PathVerb 路径动词 | spec/03-glossary.md |
| TERM-029 | Shape 图形元 | spec/03-glossary.md |
| TERM-030 | SvgScene SVG 场景 | spec/03-glossary.md |
| TERM-031 | FontSource 字体来源 | spec/03-glossary.md |
| TERM-032 | RasterOptions 光栅化选项 | spec/03-glossary.md |
| TERM-033 | TextureInfo 纹理定位信息 | spec/03-glossary.md |
| TERM-034 | TextureLayout 纹理布局 | spec/03-glossary.md |
| TERM-035 | StrokeMesh 描边网格 | spec/03-glossary.md |

### 决策

| ID | 标题 | 文件 | 状态 |
|---|---|---|---|
| ADR-001 | 用单 crate 三层分层而非多 crate workspace | plan/04-decisions/001-platform-layering.md | accepted |
| ADR-002 | 用索引 arena 而非裸指针共享弧数据 | plan/04-decisions/002-arc-arena.md | accepted |
| ADR-003 | 保留 bitcode 作为平台边界编解码 | plan/04-decisions/003-boundary-codec.md | accepted |
| ADR-004 | 以 stable Rust 为基准而非 nightly | plan/04-decisions/004-toolchain-stable.md | accepted |
| ADR-005 | 不引入替代几何库，评估整体去除 parry2d/nalgebra | plan/04-decisions/005-geometry-deps.md | accepted |
| ADR-006 | 对外入口返回错误而非 panic | plan/04-decisions/006-failure-behavior.md | accepted |

### 模块

| ID | 名称 | 目录 | 定义于 |
|---|---|---|---|
| MOD-001 | 基础 | src/core/base/ | plan/05-architecture.md |
| MOD-002 | 几何 | src/core/geom/ | plan/05-architecture.md |
| MOD-003 | 轮廓 | src/core/outline/ | plan/05-architecture.md |
| MOD-004 | 距离场 | src/core/sdf/ | plan/05-architecture.md |
| MOD-005 | 网格 | src/core/grid/ | plan/05-architecture.md |
| MOD-006 | 烘焙 | src/core/bake/ | plan/05-architecture.md |
| MOD-007 | 光栅 | src/core/raster/ | plan/05-architecture.md |
| MOD-008 | 字体 | src/api/font/ | plan/05-architecture.md |
| MOD-009 | 路径 | src/api/path/ | plan/05-architecture.md |
| MOD-010 | native 后端 | src/platform/native/ | plan/05-architecture.md |
| MOD-011 | wasm 后端 | src/platform/wasm/ | plan/05-architecture.md |

### 接口

| ID | 名称 | 模块 | 对应需求 | 定义于 | 实现文件（冻结 + 实现） |
|---|---|---|---|---|---|
| API-001 | ArcEndpoint 弧端点 | MOD-002 | REQ-001.1、REQ-002.3 | plan/07-contracts/_shared.md | src/core/geom/curve.rs + src/core/geom/curve_inner.rs |
| API-002 | Error 统一错误类型 | MOD-001 | REQ-003.2、REQ-003.3、REQ-004.1、REQ-004.2、REQ-004.3、REQ-005.1 | plan/07-contracts/base.md | src/core/base/error.rs + src/core/base/error_inner.rs |
| API-003 | Point 点 | MOD-002 | REQ-001.1、REQ-003.1、NFR-004 | plan/07-contracts/geom.md | src/core/geom/primitive.rs + src/core/geom/primitive_inner.rs |
| API-004 | Vector 向量与有符号向量 | MOD-002 | REQ-001.1、REQ-003.1 | plan/07-contracts/geom.md | src/core/geom/primitive.rs + src/core/geom/primitive_inner.rs |
| API-005 | Line 直线 | MOD-002 | REQ-001.1 | plan/07-contracts/geom.md | src/core/geom/primitive.rs + src/core/geom/primitive_inner.rs |
| API-006 | Segment 线段 | MOD-002 | REQ-001.1、REQ-005.4 | plan/07-contracts/geom.md | src/core/geom/primitive.rs + src/core/geom/primitive_inner.rs |
| API-007 | Bezier 三次贝塞尔曲线 | MOD-002 | REQ-001.1、REQ-005.3 | plan/07-contracts/geom.md | src/core/geom/curve.rs + src/core/geom/curve_inner.rs |
| API-008 | Arc 弧 | MOD-002 | REQ-001.1、REQ-005.2 | plan/07-contracts/geom.md | src/core/geom/curve.rs + src/core/geom/curve_inner.rs |
| API-009 | Aabb 包围盒 | MOD-002 | REQ-001.2、REQ-001.4 | plan/07-contracts/geom.md | src/core/geom/aabb.rs + src/core/geom/aabb_inner.rs |
| API-010 | Outline / Contour 轮廓与子轮廓 | MOD-003 | REQ-001.1、REQ-005.3 | plan/07-contracts/outline.md | src/core/outline/contour.rs + src/core/outline/contour_inner.rs |
| API-011 | Winding 绕向判定 | MOD-003 | REQ-005.3 | plan/07-contracts/outline.md | src/core/outline/contour.rs + src/core/outline/contour_inner.rs |
| API-012 | ArcFit 贝塞尔拟合弧 | MOD-003 | REQ-001.1 | plan/07-contracts/outline.md | src/core/outline/fit.rs + src/core/outline/fit_inner.rs |
| API-013 | Stroke 描边几何 | MOD-003 | REQ-001.1 | plan/07-contracts/outline.md | src/core/outline/fit.rs + src/core/outline/fit_inner.rs |
| API-014 | Sdf 弧列表距离场 | MOD-004 | REQ-001.1、REQ-001.3、REQ-005.4 | plan/07-contracts/sdf.md | src/core/sdf/sdf.rs + src/core/sdf/sdf_inner.rs |
| API-015 | Cell 格元 | MOD-005 | REQ-001.2 | plan/07-contracts/grid.md | src/core/grid/grid.rs + src/core/grid/grid_inner.rs |
| API-016 | CellGrid 近邻弧网格 | MOD-005 | REQ-001.2、REQ-002.3 | plan/07-contracts/grid.md | src/core/grid/grid.rs + src/core/grid/grid_inner.rs |
| API-017 | Subdivision 细分 | MOD-005 | REQ-001.2、REQ-005.1 | plan/07-contracts/grid.md | src/core/grid/grid.rs + src/core/grid/grid_inner.rs |
| API-018 | ArcArena 弧数据索引 arena | MOD-006 | REQ-001.2、REQ-004.3、NFR-002 | plan/07-contracts/bake.md | src/core/bake/arena.rs + src/core/bake/arena_inner.rs |
| API-019 | UnitArc 单位弧 | MOD-006 | REQ-001.2 | plan/07-contracts/bake.md | src/core/bake/arena.rs + src/core/bake/arena_inner.rs |
| API-020 | DataTexture 数据纹理 | MOD-006 | REQ-001.3 | plan/07-contracts/bake.md | src/core/bake/texture.rs + src/core/bake/texture_inner.rs |
| API-021 | IndexTexture 索引纹理 | MOD-006 | REQ-001.3、REQ-005.1 | plan/07-contracts/bake.md | src/core/bake/texture.rs + src/core/bake/texture_inner.rs |
| API-022 | TextureLayout 纹理布局 | MOD-007 | REQ-001.2、REQ-001.3、REQ-001.4 | plan/07-contracts/raster.md | src/core/raster/layout.rs + src/core/raster/layout_inner.rs |
| API-023 | Raster SDF 纹理光栅化 | MOD-007 | REQ-001.2、REQ-001.3、REQ-001.4、REQ-004.3 | plan/07-contracts/raster.md | src/core/raster/raster.rs + src/core/raster/raster_inner.rs |
| API-024 | FontFace 字体面与轮廓提取 | MOD-008 | REQ-001.1、REQ-002.1、REQ-002.2、REQ-002.3、REQ-004.2 | plan/07-contracts/font.md | src/api/font/font_face.rs + src/api/font/font_face_inner.rs |
| API-025 | Metrics 字形度量 | MOD-008 | REQ-001.1 | plan/07-contracts/font.md | src/api/font/metrics.rs + src/api/font/metrics_inner.rs |
| API-026 | PathVerb 路径动词 | MOD-009 | REQ-001.4、REQ-004.3 | plan/07-contracts/path.md | src/api/path/primitives.rs + src/api/path/primitives_inner.rs |
| API-027 | Path 路径 | MOD-009 | REQ-001.4、REQ-004.3 | plan/07-contracts/path.md | src/api/path/path.rs + src/api/path/path_inner.rs |
| API-028 | Primitives 图元 | MOD-009 | REQ-001.4、REQ-004.3 | plan/07-contracts/path.md | src/api/path/primitives.rs + src/api/path/primitives_inner.rs |
| API-029 | Scene SVG 场景 | MOD-009 | REQ-001.4 | plan/07-contracts/path.md | src/api/path/scene.rs + src/api/path/scene_inner.rs |
| API-030 | FontSource 字体来源 | MOD-010 | REQ-002.1、REQ-003.1、REQ-004.2 | plan/07-contracts/native.md | src/platform/native/font_source.rs + src/platform/native/font_source_inner.rs |
| API-031 | Exports wasm 导出边界壳 | MOD-011 | REQ-002.2、REQ-004.1、REQ-006.1 | plan/07-contracts/wasm.md | src/platform/wasm/exports.rs + src/platform/wasm/exports_inner.rs |
| API-032 | Codec 边界编解码 | MOD-011 | REQ-002.2、REQ-004.1 | plan/07-contracts/wasm.md | src/platform/wasm/codec.rs + src/platform/wasm/codec_inner.rs |

### 用例

| ID | 名称 | 对应需求 | 定义于 |
|---|---|---|---|
| CASE-001 | ArcEndpoint 往返一致 | REQ-001.1 | plan/07-contracts/_shared.md |
| CASE-002 | ArcEndpoint 跨平台一致 | REQ-002.3 | plan/07-contracts/_shared.md |
| CASE-003 | 损坏字体返回错误 | REQ-004.2 | plan/07-contracts/base.md |
| CASE-004 | 非法边界字节返回错误 | REQ-004.1 | plan/07-contracts/base.md |
| CASE-005 | 参数越界返回错误 | REQ-004.3 | plan/07-contracts/base.md |
| CASE-006 | 点距离 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-007 | 算法核心无平台代码 | REQ-003.1、NFR-004 | plan/07-contracts/geom.md |
| CASE-008 | 向量归一化 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-009 | 零向量归一化 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-010 | 直线相交 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-011 | 平行直线 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-012 | 点到线段距离 | REQ-005.4 | plan/07-contracts/geom.md |
| CASE-013 | 线段最近点 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-014 | 贝塞尔分割 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-015 | 贝塞尔子段 | REQ-005.3 | plan/07-contracts/geom.md |
| CASE-016 | 大弧包含判定 | REQ-005.2 | plan/07-contracts/geom.md |
| CASE-017 | 弧端点往返 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-018 | 空盒判定 | REQ-001.2 | plan/07-contracts/geom.md |
| CASE-019 | 盒包含点 | REQ-001.2 | plan/07-contracts/geom.md |
| CASE-020 | reverse 原地生效 | REQ-005.3 | plan/07-contracts/outline.md |
| CASE-021 | 端点往返一致 | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-022 | outline_winding 改写 | REQ-005.3 | plan/07-contracts/outline.md |
| CASE-023 | even_odd 内点 | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-024 | 贝塞尔拟合弧 | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-025 | 退化贝塞尔 | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-026 | 描边网格 | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-027 | 内点距离符号 | REQ-005.4 | plan/07-contracts/sdf.md |
| CASE-028 | 外点距离符号 | REQ-001.3 | plan/07-contracts/sdf.md |
| CASE-029 | 格元索引有效 | REQ-001.2 | plan/07-contracts/grid.md |
| CASE-030 | 网格覆盖范围 | REQ-001.2 | plan/07-contracts/grid.md |
| CASE-031 | 网格跨平台一致 | REQ-002.3 | plan/07-contracts/grid.md |
| CASE-032 | 细分终止 | REQ-005.1 | plan/07-contracts/grid.md |
| CASE-033 | 细分与现状一致 | REQ-001.2 | plan/07-contracts/grid.md |
| CASE-034 | arena 同键去重 | REQ-001.2 | plan/07-contracts/bake.md |
| CASE-035 | arena 越界返回 None | REQ-004.3 | plan/07-contracts/bake.md |
| CASE-036 | 单位弧区间有序 | REQ-001.2 | plan/07-contracts/bake.md |
| CASE-037 | 数据纹理往返 | REQ-001.3 | plan/07-contracts/bake.md |
| CASE-038 | 量化溢出返回 Err | REQ-001.3 | plan/07-contracts/bake.md |
| CASE-039 | 索引纹理终止 | REQ-005.1 | plan/07-contracts/bake.md |
| CASE-040 | 索引指向有效 | REQ-001.3 | plan/07-contracts/bake.md |
| CASE-041 | 布局稳定可复现 | REQ-001.2 | plan/07-contracts/raster.md |
| CASE-042 | 布局参数校验 | REQ-004.3 | plan/07-contracts/raster.md |
| CASE-043 | 光栅与现状等价 | REQ-001.3 | plan/07-contracts/raster.md |
| CASE-044 | 纹理尺寸 | REQ-001.3 | plan/07-contracts/raster.md |
| CASE-045 | 越界索引返回 Err | REQ-004.3 | plan/07-contracts/raster.md |
| CASE-046 | SVG 光栅等价 | REQ-001.4 | plan/07-contracts/raster.md |
| CASE-047 | 光栅可复现 | REQ-001.2 | plan/07-contracts/raster.md |
| CASE-048 | 损坏字体返回 Err | REQ-004.2 | plan/07-contracts/font.md |
| CASE-049 | 字形轮廓等价 | REQ-001.1 | plan/07-contracts/font.md |
| CASE-050 | 字体跨平台一致 | REQ-002.3 | plan/07-contracts/font.md |
| CASE-051 | native 接口可用 | REQ-002.1 | plan/07-contracts/font.md |
| CASE-052 | 字形度量 | REQ-001.1 | plan/07-contracts/font.md |
| CASE-053 | 非法动词返回 Err | REQ-004.3 | plan/07-contracts/path.md |
| CASE-054 | 动词往返 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-055 | 路径 SDF 等价 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-056 | 坐标不匹配返回 Err | REQ-004.3 | plan/07-contracts/path.md |
| CASE-057 | 路径包围盒 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-058 | 六图元轮廓 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-059 | 负半径返回 Err | REQ-004.3 | plan/07-contracts/path.md |
| CASE-060 | 场景布局 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-061 | 场景可复现 | REQ-001.4 | plan/07-contracts/path.md |
| CASE-062 | Windows 系统字体加载 | REQ-002.1 | plan/07-contracts/native.md |
| CASE-063 | 不存在路径返回 Err | REQ-004.2 | plan/07-contracts/native.md |
| CASE-064 | 算法核心无平台引用 | REQ-003.1、NFR-004 | plan/07-contracts/native.md |
| CASE-065 | 乱码字节不 trap | REQ-004.1 | plan/07-contracts/wasm.md |
| CASE-066 | 字符调试接口可用 | REQ-006.1 | plan/07-contracts/wasm.md |
| CASE-067 | SVG 调试接口可用 | REQ-006.1 | plan/07-contracts/wasm.md |
| CASE-068 | wasm 接口可用 | REQ-002.2 | plan/07-contracts/wasm.md |
| CASE-069 | 编解码往返 | REQ-002.2 | plan/07-contracts/wasm.md |
| CASE-070 | 随机字节解码返回 Err | REQ-004.1 | plan/07-contracts/wasm.md |
| CASE-073 | 直线退化不崩溃 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-074 | 贝塞尔退化不崩溃 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-075 | 弧退化不崩溃 | REQ-001.1 | plan/07-contracts/geom.md |
| CASE-076 | 非正容差返回 Err | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-077 | 非正厚度返回 Err | REQ-001.1 | plan/07-contracts/outline.md |
| CASE-078 | 越界下标返回 Err | REQ-004.3 | plan/07-contracts/sdf.md |
| CASE-079 | 非正 scale 返回 Err | REQ-004.3 | plan/07-contracts/grid.md |
| CASE-080 | 偏移超位宽返回 Err | REQ-001.3 | plan/07-contracts/bake.md |
| CASE-081 | 无字形返回 Err | REQ-004.3 | plan/07-contracts/font.md |
| CASE-082 | 点数不足返回 Err | REQ-004.3 | plan/07-contracts/path.md |
| CASE-083 | 依赖单向无环 | REQ-003.2 | plan/07-contracts/base.md |
| CASE-084 | 算法层按职责拆分 | REQ-003.3 | plan/07-contracts/base.md |
| CASE-085 | 基准不劣于基线 | NFR-001 | plan/07-contracts/raster.md |
| CASE-086 | 三目标编译通过 | NFR-003 | plan/07-contracts/base.md |

### 任务

| ID | 标题 | 阻塞于 | 状态 |
|---|---|---|---|
| TASK-01 | 工程骨架与构建 | 无 | open |
| TASK-02 | 基础实现 | TASK-01 | open |
| TASK-03 | 几何基础 | TASK-02 | open |
| TASK-04 | 轮廓与绕向 | TASK-03 | open |
| TASK-05 | 弧拟合与描边 | TASK-03 | open |
| TASK-06 | 距离场 | TASK-03 | open |
| TASK-07 | 网格细分 | TASK-03 | open |
| TASK-08 | 烘焙 | TASK-07 | open |
| TASK-09 | 布局与光栅化 | TASK-06、TASK-07 | open |
| TASK-10 | 字体 | TASK-04 | open |
| TASK-11 | 路径与图元 | TASK-04、TASK-09 | open |
| TASK-12 | 场景 | TASK-11 | open |
| TASK-13 | native 字体来源 | TASK-10 | open |
| TASK-14 | wasm 边界壳 | TASK-10、TASK-11 | open |
| TASK-15 | 基准与等价回归 | TASK-02..TASK-14 | open |

## 未决项

| 标签 | 位置 | 归属 | 何时解决 |
|---|---|---|---|
| Deferred | tasks/task-map.md 尚未成形（wasm 调试页交互形态） | TASK-14 | 实现阶段前端跑起来后定 |

## 变更日志

| 日期 | 阶段 | 变更 | 影响的产物 |
|---|---|---|---|
| 2026-09-22 | D0 | 绑定 slug pi_sdf2，创建索引 | 无 |
| 2026-09-22 | D1 | 需求落盘；模式=完整模式；门禁确认 5 项默认（提问 1 轮） | 无 |
| 2026-09-22 | D2 | 本轮提问 3 次（SVG 链路纳入等价、依赖不可替换可去除、stable 优先） | 无 |
| 2026-09-22 | D4 | 6 条 ADR 落盘（all accepted）；沿用清单照 D0 技术栈；D4 候选完整性复问 1 次 | 无 |
| 2026-09-22 | D5 | 架构落盘：11 个 MOD 三层结构；描边并入 core/outline；硬约束=核心算法流程不变 | 无 |
| 2026-09-22 | D6 | 布局与规范落盘：20 对冻结 + 6 类豁免；stable 兼容 + fmt/clippy；测试分三层；examples 补充覆盖 | 无 |
| 2026-09-22 | D7 | 生产计划确认：5 批；接口粒度=一个类型/一族函数一条 | 无 |
| 2026-09-22 | D7 | 5 批契约全部确认；11 个契约文件、32 个 API、70 个用例；D7 自检通过 | 无 |
| 2026-09-22 | D7b | 批0落地 8 文件；auditBatch 14 项全绿；projectBatch 不可用改手写；提交待用户执行 | 无 |
| 2026-09-22 | D7b | 批1落地 8 文件；sdf 契约签名修正为 Result（越界返回 InvalidParam） | 无 |
| 2026-09-22 | D7b | 批2落地 8 文件（含 ADR-002 的 ArcArena） | 无 |
| 2026-09-22 | D7b | 批3落地 10 文件 | 无 |
| 2026-09-22 | D7b | 批4落地 6 文件；5 批全部落地完成 | 无 |
| 2026-09-22 | D7b | 全批落地收尾：20 对 / 40 文件与 D6 冻结面逐行一致；各批审计 14 项全绿；提交待用户执行 | 无 |
| 2026-09-22 | D8 | 任务图落盘：15 个垂直切片任务；前沿=TASK-01；前沿触及无交集；每条 REQ 均被覆盖 | 无 |
| 2026-09-22 | D9 | 审查完成：不通过。9 项 ❌（C3/C5/C8/C9/C14/C15/C16/C32/C43/C48/C50 计）+ 覆盖表缺 REQ-003.2/003.3 + 反向孤儿 5 项；C12 未判；C45 无基线。报告已落 09-review.md | 无 |
| 2026-09-22 | D9 | 回溯改 D3 词汇表（补 TERM-024..035 + 通用词豁免）以消解 C9；C3/C14 复核为 ddt 工具误报，C12 补判为 ✅。按三个区规则，plan/ 与 tasks/ 整体失效待重审；D9 须全量重跑 | 无 |
| 2026-09-22 | D9 | 修复轮：D1/D2 去待澄清标签字面量、D4 补 NFR 选型列、D5 结构不变量改判、D7 补 ensures 与 12 个错误用例、D7b 补锚点、D8 修任务切片与目录表、索引补孤儿关系与未决项 | 无 |
| 2026-09-22 | D9 | 重审完成：设计层 ❌ 清零（C5/C8/C9/C12/C15/C16/C32/C43/C50 均已修复并经子代理复核）；余 C45 未验证（无 git 基线）、C48 ❌（测试待实现阶段） | 无 |
| 2026-09-23 | D9 | D9 收口：设计层通过，登记完成。已知遗留——C45 冻结无基线（待建 git 基线）、C48 测试待实现阶段写入 | 无 |
| 2026-09-23 | D9 | 建立 git 基线并推送远端新分支 pi_sdf2（commit d2773ef）；冻结文件自落地 diff 为空，C45 核销 | 无 |
