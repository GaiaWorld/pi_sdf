# 契约：光栅（数据与算法）

**模块：** MOD-004（数据）、MOD-011（算法）

## 接口

### API-019 UnitArc 单位弧

- 模块：MOD-004
- 对应需求：REQ-001.2
- 签名：

  ```
  pub struct UnitArc { pub endpoints: Vec<ArcEndpoint>, pub sdf_min: f32, pub sdf_max: f32, pub show: bool }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  ```

- requires：endpoints 非空；sdf_min 不大于 sdf_max
- ensures：距离区间有序（TERM-017）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | 无 | 纯数据结构，构造不失败 | — |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-036 | REQ-001.2 | 一条单位弧 | 检查区间 | sdf_min ≤ sdf_max |

### API-020 DataTexture 数据纹理

- 模块：MOD-004
- 对应需求：REQ-001.3
- 签名：

  ```
  pub struct DataTexture { pub pixels: Vec<u8>, pub width: u32, pub height: u32 }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  ```

- requires：像素数据长度为宽乘高
- ensures：每条记录可解码为弧端点（TERM-015）
- 错误模式：无（纯数据结构，不产生错误）
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-089 | REQ-001.3 | 一份数据纹理 | 检查尺寸 | pixels 长度等于 width × height |

### API-021 IndexTexture 索引纹理

- 模块：MOD-004
- 对应需求：REQ-001.3
- 签名：

  ```
  pub struct IndexTexture { pub pixels: Vec<u8>, pub width: u32, pub height: u32 }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  ```

- requires：像素数据长度为宽乘高
- ensures：索引指向有效数据纹理位置（TERM-016）
- 错误模式：无（纯数据结构，不产生错误）
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-090 | REQ-001.3 | 一份索引纹理 | 检查尺寸 | pixels 长度等于 width × height |

### API-022 TextureLayout 纹理布局

- 模块：MOD-004
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4
- 签名：

  ```
  pub struct TextureLayout { pub plane_bounds: Aabb, pub atlas_bounds: Aabb, pub distance: f32, pub tex_size: u32 }
  ```

- requires：tex_size > 0；pxrange > 0（构造由 API-035 保证）
- ensures：tex_size > 0、distance > 0（TERM-011、TERM-012）
- 错误模式：无（纯数据结构，不产生错误）
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-091 | REQ-001.2 | 一份纹理布局 | 检查字段 | tex_size > 0 且 distance > 0 |

### API-023 SdfTexture / TextureInfo / RasterOptions

- 模块：MOD-004
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4
- 签名：

  ```
  pub struct TextureInfo { pub plane_bounds: Aabb, pub atlas_bounds: Aabb, pub sdf_offset_x: f32, pub sdf_offset_y: f32 }
  pub struct SdfTexture { pub pixels: Vec<u8>, pub tex_size: u32, pub tex_info: TextureInfo }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  pub struct RasterOptions { pub is_outer_glow: bool, pub is_svg: bool, pub is_reverse: Option<bool> }
  ```

- requires：pixels 长度等于 tex_size 的平方
- ensures：pixels 长度 = tex_size × tex_size（TERM-010）
- 错误模式：

  无（纯数据结构，不产生错误）

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-044 | REQ-001.3 | 网格与布局 | 检查纹理 | pixels 长度等于 tex_size² |

### API-035 纹理布局计算

- 模块：MOD-011
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4
- 签名：

  ```
  pub fn compute_layout(extents: Aabb, tex_size: u32, pxrange: u32, units_per_em: f32, offset: u32, is_svg: bool) -> Result<TextureLayout>;
  ```

- requires：tex_size > 0；pxrange > 0；units_per_em > 0
- ensures：结果稳定可复现（消除现状因哈希表无序导致的不稳定）；distance = 每像素距离 × pxrange
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 参数非正 | 返回 Err，不 panic |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-041 | REQ-001.2 | 同一输入 | compute_layout 执行两次 | 两次结果完全一致 |
| CASE-042 | REQ-004.3 | tex_size = 0 | compute_layout | 返回 Err |

### API-036 光栅化

- 模块：MOD-011
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4、REQ-004.3
- 签名：

  ```
  pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture>;
  ```

- requires：layout 的 tex_size 与目标纹理一致；grid 的索引有效
- ensures：pixels 长度 = tex_size × tex_size；逐像素值与现状等价（差 ≤ ±1 灰度，REQ-001.3）；按「格元 × 覆盖像素 × 近邻弧」遍历，复杂度与现状同阶
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 网格索引越界 | 返回 Err，不 panic |
  | Geometry | 布局与网格范围不符 | 返回 Err，不 panic |

- 顺序约束：必须在 compute_cell_grid（API-017）与 compute_layout（API-035）之后调用
- 性能：O(格元覆盖像素 × 近邻弧数)，与现状同阶

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-043 | REQ-001.3 | 与 pi_sdf 同一轮廓与参数 | rasterize | 逐像素差 ≤ ±1 灰度 |
| CASE-045 | REQ-004.3 | 越界的网格索引 | rasterize | 返回 Err，不 panic |
| CASE-046 | REQ-001.4 | 同一 SVG 路径 | rasterize | 与 pi_sdf 逐像素差 ≤ ±1 灰度 |
| CASE-047 | REQ-001.2 | 同一输入 | rasterize 执行两次 | 两次结果完全一致 |
| CASE-085 | NFR-001 | 与 pi_sdf 同一输入的端到端链路 | 基准测试 | 耗时与峰值内存 ≤ 基线 100% |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/model/raster/texture.rs | API-019、API-020、API-021 | 是 | |
| src/model/raster/texture_inner.rs | API-019、API-020、API-021 | 否（实现） | |
| src/model/raster/sdf.rs | API-022、API-023 | 是 | |
| src/model/raster/sdf_inner.rs | API-022、API-023 | 否（实现） | |
| src/core/raster/layout.rs | API-035 | 是 | |
| src/core/raster/layout_inner.rs | API-035 | 否（实现） | |
| src/core/raster/raster.rs | API-036 | 是 | |
| src/core/raster/raster_inner.rs | API-036 | 否（实现） | |
