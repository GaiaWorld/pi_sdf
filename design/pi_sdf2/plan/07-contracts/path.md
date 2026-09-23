# 契约：路径

**模块：** MOD-005（数据）、MOD-013（场景）

## 接口

### API-026 PathVerb 路径动词

- 模块：MOD-005
- 对应需求：REQ-001.4、REQ-004.3
- 签名：

  ```
  #[repr(u8)]
  pub enum PathVerb { MoveTo = 1, /* …与现状一致的 19 个动词… */ Close = 19 }
  impl TryFrom<u8> for PathVerb { type Error = Error; }
  impl From<PathVerb> for u8 { /* … */ }
  ```

- requires：判别值落在 1..=19
- ensures：**非法判别值返回 Err**（替换现状 transmute UB，REQ-004.3）；判别值与 JS u8 契约一致
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidPathVerb | 判别值不在 1..=19 | 返回 Err，不 UB |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-053 | REQ-004.3 | 字节值 200 | PathVerb::try_from | 返回 Err(InvalidPathVerb) |
| CASE-054 | REQ-001.4 | 字节值 1..19 | try_from 后再转回 | 与输入一致 |

### API-027 Path 路径

- 模块：MOD-005
- 对应需求：REQ-001.4、REQ-004.3
- 签名：

  ```
  pub struct Path { pub verbs: Vec<PathVerb>, pub points: Vec<f32> }   // points 拍平存放，每点 2 个分量
  impl Path {
      pub fn new(verbs: Vec<u8>, points: Vec<f32>) -> Result<Self>;
      pub fn from_verbs(verbs: Vec<PathVerb>, points: Vec<f32>) -> Result<Self>;
      pub fn to_outline(&self) -> Result<Outline>;
      pub fn sdf_texture(&self, tex_size: u32, pxrange: u32) -> Result<SdfTexture>;
      pub fn extents(&self) -> Aabb;
      pub fn reverse(&mut self);
  }
  ```

- requires：points 长度与 verbs 所需的坐标数一致；tex_size、pxrange 为正
- ensures：非法动词或坐标不匹配返回 Err；sdf_texture 结果与 pi_sdf 等价（±1 灰度）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidPathVerb | 动词字节非法 | 返回 Err |
  | InvalidParam | 坐标数量不匹配或参数非正 | 返回 Err |

- 顺序约束：无
- 性能：同现状
- wasm 投影：verbs 与 points 字段被 skip，由同名 getter 暴露，JS 侧访问形式不变（ADR-007）

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-055 | REQ-001.4 | 合法路径 | sdf_texture | 与 pi_sdf 逐像素差 ≤ ±1 灰度 |
| CASE-056 | REQ-004.3 | 坐标数量不匹配 | Path::new | 返回 Err，不 panic |
| CASE-057 | REQ-001.4 | 同一路径 | extents | 与现状包围盒一致 |

### API-028 Primitives 图元

- 模块：MOD-005
- 对应需求：REQ-001.4、REQ-004.3
- 签名：

  ```
  pub struct Shape;                       // opaque 类；内部为 pub(crate) enum ShapeKind
  impl Shape {
      pub fn circle(cx: f32, cy: f32, r: f32) -> Self;
      pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Self;
      pub fn line(ax: f32, ay: f32, bx: f32, by: f32, step: Option<f32>) -> Self;
      pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self;
      pub fn polygon(points: Vec<f32>) -> Self;       // 扁平坐标，每点 2 个分量
      pub fn polyline(points: Vec<f32>, is_close: bool) -> Self;
      pub fn to_outline(&self) -> Result<Outline>;
      pub fn extents(&self) -> Aabb;
      pub fn is_area(&self) -> bool;
  }
  ```

- requires：半径与宽高为非负值；多边形至少 3 点
- ensures：六种图元轮廓与 pi_sdf 等价；非法尺寸返回 Err（消除现状 get_hash 样板与 unwrap）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 半径 / 宽高为负，或点数不足 | 返回 Err |

- 顺序约束：无
- 性能：圆 / 椭圆按分段离散，同现状

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-058 | REQ-001.4 | 六种图元 | to_outline | 各自轮廓与 pi_sdf 一致 |
| CASE-059 | REQ-004.3 | 半径为负的圆 | to_outline | 返回 Err |
| CASE-082 | REQ-004.3 | 点数不足 3 的多边形 | to_outline | 返回 Err(InvalidParam)，不 panic |

### API-029 Scene SVG 场景

- 模块：MOD-013
- 对应需求：REQ-001.4
- 签名：

  ```
  pub struct SvgScene { pub view_box: Aabb }
  impl SvgScene {
      pub fn new() -> Self;
      pub fn add(&mut self, key: u64, shape: Shape);
      pub fn has(&self, key: u64) -> bool;
      pub fn set_view_box(&mut self, vb: Aabb);
      pub fn layout(&self, tex_size: u32, pxrange: u32) -> Result<Vec<(u64, SdfTexture)>>;
  }
  ```

- requires：tex_size、pxrange 为正
- ensures：每个图元的 SDF 纹理与单独调用 Path::sdf_texture 一致；结果按 key 有序（可复现）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 参数非正 | 返回 Err |

- 顺序约束：无
- 性能：线性于图元数

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-060 | REQ-001.4 | 多个图元的场景 | layout | 每项纹理与单独生成一致 |
| CASE-061 | REQ-001.4 | 同一场景 | layout 两次 | 结果顺序与内容一致 |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/model/path/path.rs | API-026、API-027、API-028 | 是 | |
| src/model/path/path_inner.rs | API-026、API-027、API-028 | 否（实现） | |
| src/api/path/scene.rs | API-029 | 是 | |
| src/api/path/scene_inner.rs | API-029 | 否（实现） | |
