# 契约：轮廓

**模块：** MOD-002（数据）、MOD-007（算法）

## 接口

### API-010 Outline / Contour 轮廓与子轮廓

- 模块：MOD-002
- 对应需求：REQ-001.1、REQ-005.3
- 签名：

  ```
  pub struct Contour { pub arcs: Vec<Arc>, pub is_closed: bool }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  pub struct Outline { pub contours: Vec<Contour> }   // wasm 投影：字段 skip + 同名 getter，JS 访问形式不变（ADR-007）
  impl Contour {
      pub fn new(arcs: Vec<Arc>, is_closed: bool) -> Self;
      pub fn from_endpoints(eps: Vec<ArcEndpoint>) -> Self;
      pub fn endpoints(&self) -> Vec<ArcEndpoint>;
      pub fn is_clockwise(&self) -> bool;
      pub fn reverse(&mut self);
      pub fn extents(&self) -> Aabb;
  }
  impl Outline {
      pub fn new(contours: Vec<Contour>) -> Self;
      pub fn extents(&self) -> Aabb;
      pub fn reverse(&mut self);
      pub fn is_clockwise(&self) -> bool;
  }
  ```

- requires：Contour 闭合时首尾弧端点相接
- ensures：reverse 结果**原地作用**于自身（修正现状对副本取可变引用、改动丢失的缺陷，REQ-005.3）；from_endpoints 与 endpoints 往返一致；闭合子轮廓的首尾弧端点相接（TERM-001），reverse 不改变闭合性
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Geometry | 端点序列首尾不接 | 返回未闭合轮廓，调用方按 is_closed 判定 |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-020 | REQ-005.3 | 一条顺时针子轮廓 | 调用 reverse，再读取自身 | 绕向变为逆时针（非副本） |
| CASE-021 | REQ-001.1 | 一组端点 | endpoints → from_endpoints → endpoints | 两次端点集合一致 |

### API-011 Winding 绕向判定

- 模块：MOD-002
- 对应需求：REQ-005.3
- 签名：

  ```
  pub fn contour_winding(contour: &mut Contour, inverse: bool);
  pub fn outline_winding(outline: &mut Outline, inverse: bool);
  pub fn even_odd(contour: &Contour, p: Point) -> bool;
  ```

- requires：contour 至少含一条弧
- ensures：winding 决定（该反转时）真正改写传入对象；even_odd 对闭合轮廓给出内外判定
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-022 | REQ-005.3 | 需要反转的轮廓 | outline_winding(outline, true) | 传入 outline 的绕向被改写，返回后仍可见 |
| CASE-023 | REQ-001.1 | 闭合轮廓内一点 | even_odd | 返回真 |

### API-012 ArcFit 贝塞尔拟合弧

- 模块：MOD-007
- 对应需求：REQ-001.1
- 签名：

  ```
  pub fn bezier_to_arcs(b: &Bezier, max_deviation: f32) -> Result<Vec<Arc>>;
  ```

- requires：max_deviation 为正有限值
- ensures：生成弧的首尾端点与贝塞尔一致；每弧中点相对贝塞尔的偏差不超过 max_deviation
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | max_deviation 非正 | 返回 Err，调用方改用正容差 |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-024 | REQ-001.1 | 一条贝塞尔与容差 | bezier_to_arcs | 首尾端点一致，最大偏差 ≤ 容差 |
| CASE-025 | REQ-001.1 | 退化为直线的贝塞尔 | bezier_to_arcs | 返回单条线段语义弧（d=0） |
| CASE-076 | REQ-001.1 | max_deviation 为 0 或负 | bezier_to_arcs | 返回 Err(InvalidParam)，不 panic |

### API-013 Stroke 描边几何

- 模块：MOD-007
- 对应需求：REQ-001.1
- 签名：

  ```
  pub struct StrokeMesh { pub positions: Vec<f32>, pub uvs: Vec<f32>, pub indices: Vec<u16> }
  pub fn stroke_mesh(arcs: &[Arc], thickness: f32) -> Result<StrokeMesh>
  ```

- requires：thickness 为正有限值；arcs 非空
- ensures：每条弧生成 4 个顶点、2 个三角形（索引数 = 弧数 × 6）；法线按节点邻接计算
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | thickness 非正 | 返回 Err |

- 顺序约束：无
- 性能：线性于弧数

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-026 | REQ-001.1 | 一条含 3 弧的轮廓 | stroke_mesh | 12 个顶点（位置 24 个分量、UV 24 个分量）、索引 18 个 |
| CASE-077 | REQ-001.1 | thickness 为 0 或负 | stroke_mesh | 返回 Err(InvalidParam)，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/model/outline/contour.rs | API-010、API-011 | 是 | |
| src/model/outline/contour_inner.rs | API-010、API-011 | 否（实现） | |
| src/core/outline/fit.rs | API-012、API-013 | 是 | |
| src/core/outline/fit_inner.rs | API-012、API-013 | 否（实现） | |
