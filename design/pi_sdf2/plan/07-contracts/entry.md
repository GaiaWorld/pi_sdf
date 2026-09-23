# 契约：导出入口与边界编解码

**模块：** MOD-013

## 接口

### API-031 Exports 跨语言导出入口

- 模块：MOD-013
- 对应需求：REQ-002.2、REQ-004.1、REQ-006.1
- 签名：

  ```
  // 主链路（类型投影）：薄转发到 core
  pub fn compute_cell_grid(outline: &Outline, scale: f32, is_area: bool) -> Result<CellGrid>;
  pub fn compute_layout(extents: &Aabb, tex_size: u32, pxrange: u32, units_per_em: f32, offset: u32, is_svg: bool) -> Result<TextureLayout>;
  pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture>;
  // 字节通道与工具
  pub fn compute_near_arcs(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn compute_sdf_tex(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn get_char_arc_debug(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn compute_svg_debug(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn brotli_decompressor(data: &[u8]) -> Result<Vec<u8>>;
  ```

  导出注解：每个函数带 `#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]`（native 下不生效）。

- requires：类型投影入口的入参满足各自约束；字节通道入口的 bytes 由 Codec 编码
- ensures：类型投影入口只做转发不含算法；字节通道入口只做「解码 → 调用 → 编码」；非法字节返回 Err，wasm 不 trap（REQ-004.1）；get_char_arc_debug 与 compute_svg_debug 可用（REQ-006.1）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Decode | 边界字节无法解析 | 返回 Err（映射为 JS 异常），不 trap |
  | InvalidParam | 载荷或参数内字段越界 | 返回 Err |

- 顺序约束：主链路入口须按「轮廓 → 网格 → 布局 → 光栅化」顺序调用
- 性能：类型投影入口零编解码；字节通道入口开销线性于载荷大小

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-065 | REQ-004.1 | 一段乱码字节 | compute_near_arcs | 返回 Err，模块不 trap |
| CASE-066 | REQ-006.1 | 合法字符载荷 | get_char_arc_debug | 返回可展示的轮廓/近邻弧数据 |
| CASE-067 | REQ-006.1 | 合法 SVG 载荷 | compute_svg_debug | 返回可展示的 SDF 数据 |
| CASE-068 | REQ-002.2 | wasm32 目标 | 编译并调用导出 | 核心链路接口可用 |
| CASE-087 | REQ-002.2 | 已取到 Outline | compute_cell_grid(outline, scale, is_area) | 返回 CellGrid，与 core::grid 结果一致 |
| CASE-088 | REQ-002.2 | 已取到 CellGrid | compute_layout + rasterize | 返回 SdfTexture，与 core::raster 结果一致 |

### API-032 Codec 边界编解码

- 模块：MOD-013
- 对应需求：REQ-002.2、REQ-004.1
- 签名：

  ```
  pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>>;
  pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>;
  ```

  说明：泛型不可导出，本条目不挂 wasm 注解；仅供 native 与字节通道内部使用（ADR-003 保持）。

- requires：T 实现序列化框架的对应特征
- ensures：编解码只出现在字节通道；解码失败返回 Err（替换现状 deserialize().unwrap()）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Decode | 字节不合法或结构不符 | 返回 Err |
  | Encode | 序列化失败 | 返回 Err |

- 顺序约束：无
- 性能：线性于载荷大小

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-069 | REQ-002.2 | 任一可序列化结构 | encode → decode | 与输入一致 |
| CASE-070 | REQ-004.1 | 随机字节 | decode | 返回 Err，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/api/entry.rs | API-031 | 是 | |
| src/api/entry_inner.rs | API-031 | 否（实现） | |
| src/api/codec.rs | API-032 | 是 | |
| src/api/codec_inner.rs | API-032 | 否（实现） | |
