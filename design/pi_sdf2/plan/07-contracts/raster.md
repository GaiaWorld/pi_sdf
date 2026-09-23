# 契约：光栅

**模块：** MOD-007

## 接口

### API-022 TextureLayout 纹理布局

- 模块：MOD-007
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4
- 签名：

  ```
  pub struct TextureLayout {
      pub plane_bounds: Aabb,
      pub atlas_bounds: Aabb,
      pub distance: f32,
      pub tex_size: u32,
  }
  pub fn compute_layout(
      extents: Aabb,
      tex_size: u32,
      pxrange: u32,
      units_per_em: f32,
      offset: u32,
      is_svg: bool,
  ) -> Result<TextureLayout>
  ```

- requires：tex_size > 0；pxrange > 0；units_per_em > 0
- ensures：**结果稳定可复现**（消除现状因哈希表遍历无序导致的不稳定）；distance = 每像素距离 × pxrange；tex_size > 0、pxrange > 0、distance > 0（TERM-011、TERM-012）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | tex_size / pxrange / units_per_em 非正 | 返回 Err，不 panic |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-041 | REQ-001.2 | 同一输入 | compute_layout 执行两次 | 两次结果完全一致 |
| CASE-042 | REQ-004.3 | tex_size = 0 | compute_layout | 返回 Err |

### API-023 Raster SDF 纹理光栅化

- 模块：MOD-007
- 对应需求：REQ-001.2、REQ-001.3、REQ-001.4、REQ-004.3
- 签名：

  ```
  pub struct TextureInfo { pub plane_bounds: Aabb, pub atlas_bounds: Aabb, pub sdf_offset_x: f32, pub sdf_offset_y: f32 }
  pub struct SdfTexture { pub pixels: Vec<u8>, pub tex_size: u32, pub tex_info: TextureInfo }
  pub struct RasterOptions { pub is_outer_glow: bool, pub is_svg: bool, pub is_reverse: Option<bool> }
  pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture>
  ```

- requires：layout 的 tex_size 与目标纹理一致；grid 的索引有效
- ensures：pixels 长度 = tex_size × tex_size；逐像素值与现状等价（差 ≤ ±1 灰度，REQ-001.3）；按「格元 × 覆盖像素 × 近邻弧」遍历，复杂度与现状同阶
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 网格索引越界 | 返回 Err，不 panic |
  | Geometry | 布局与网格范围不符 | 返回 Err，不 panic |

- 顺序约束：必须在 compute_cell_grid 与 compute_layout 之后调用
- 性能：O(格元覆盖像素 × 近邻弧数)，与现状同阶

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-043 | REQ-001.3 | 与 pi_sdf 同一轮廓与参数 | rasterize | 逐像素差 ≤ ±1 灰度 |
| CASE-044 | REQ-001.3 | 网格与布局 | rasterize | pixels 长度等于 tex_size² |
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
| src/core/raster/layout.rs | API-022 | 是 | |
| src/core/raster/layout_inner.rs | API-022 | 否（实现） | |
| src/core/raster/raster.rs | API-023 | 是 | |
| src/core/raster/raster_inner.rs | API-023 | 否（实现） | |
