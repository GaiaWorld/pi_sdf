# 契约：几何

**模块：** MOD-001

## 接口

### API-003 Point 点

- 模块：MOD-001
- 对应需求：REQ-001.1、REQ-003.1
- 签名：

  ```
  pub struct Point { pub x: f32, pub y: f32 }
  impl Point {
      pub fn new(x: f32, y: f32) -> Self;
      pub fn distance_to(self, other: Point) -> f32;
      pub fn squared_distance_to(self, other: Point) -> f32;
      pub fn midpoint(self, other: Point) -> Point;
      pub fn to_vector(self) -> Vector;
      pub fn add_vector(self, v: Vector) -> Point;
      pub fn shortest_distance_to_line(self, line: &Line) -> SignedVector;
  }
  ```

- requires：坐标有限；distance_to 的 another 为有限点
- ensures：distance_to 恒非负；midpoint 位于两点连线中点
- 错误模式：无（坐标非法由上游构造点保证）
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-006 | REQ-001.1 | 两点 (0,0)、(3,4) | 求距离 | 返回 5.0（容差 1e-6） |
| CASE-007 | REQ-003.1 | 算法核心源码 | 检索 Point 实现 | 无平台 cfg、无 unsafe |

### API-004 Vector 向量与有符号向量

- 模块：MOD-001
- 对应需求：REQ-001.1、REQ-003.1
- 签名：

  ```
  pub struct Vector { pub x: f32, pub y: f32 }
  pub struct SignedVector { pub vec2: Vector, pub negative: bool }
  impl Vector {
      pub fn new(x: f32, y: f32) -> Self;
      pub fn dot(self, other: Vector) -> f32;
      pub fn cross(self, other: Vector) -> f32;
      pub fn norm_squared(self) -> f32;
      pub fn norm(self) -> f32;
      pub fn normalized(self) -> Vector;
      pub fn ortho(self) -> Vector;
      pub fn angle(self) -> f32;
      pub fn rebase(self, bx: f32, by: f32) -> Vector;
  }
  impl SignedVector {
      pub fn new(x: f32, y: f32, negative: bool) -> Self;
      pub fn from_vector(v: Vector, negative: bool) -> Self;
      pub fn neg(self) -> Self;
  }
  ```

- requires：normalized 的向量模长不为 0
- ensures：normalized 结果模长为 1（容差内）；cross 满足反对称
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Geometry | normalized 输入模长为 0 | 返回零向量，不 panic；调用方需自行判定退化 |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-008 | REQ-001.1 | 向量 (3,4) | normalized | 模长为 1，方向不变 |
| CASE-009 | REQ-001.1 | 零向量 | normalized | 返回零向量，不 panic |

### API-005 Line 直线

- 模块：MOD-001
- 对应需求：REQ-001.1
- 签名：

  ```
  pub struct Line { pub n: Vector, pub c: f32 }   // 直线方程 n·x = c
  impl Line {
      pub fn new(a: f32, b: f32, c: f32) -> Self;
      pub fn from_points(p0: Point, p1: Point) -> Self;
      pub fn normalized(self) -> Self;
      pub fn normal(&self) -> &Vector;                                   // wasm 不导出（借用返回，ADR-007）
      pub fn intersect(&self, other: &Line) -> Option<Point>;
      pub fn sub(&self, p: &Point) -> SignedVector;
  }
  ```

- requires：from_points 的两点不重合
- ensures：normalized 后法向量模长为 1；sub 的符号表示点在直线哪一侧
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Geometry | from_points 两点重合 | 返回 None 语义需调用方处理（见 intersect 返回 Option） |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-010 | REQ-001.1 | 两条相交直线 | intersect | 返回交点坐标（容差内） |
| CASE-011 | REQ-001.1 | 两条平行直线 | intersect | 返回 None |
| CASE-073 | REQ-001.1 | 两点重合 | from_points | 返回退化直线（零法向量），不 panic |

### API-006 Segment 线段

- 模块：MOD-001
- 对应需求：REQ-001.1、REQ-005.4
- 签名：

  ```
  pub struct Segment { pub a: Point, pub b: Point }
  impl Segment {
      pub fn new(a: Point, b: Point) -> Self;
      pub fn distance_to_point(&self, p: Point) -> f32;
      pub fn squared_distance_to_point(&self, p: Point) -> f32;
      pub fn nearest_points_on_line_segments(a: &Segment, b: &Segment) -> (Point, Point);   // wasm 不导出（元组返回，ADR-007）
      pub fn contains_in_span(&self, p: Point) -> bool;
  }
  ```

- requires：端点为有限点
- ensures：distance_to_point 恒非负且不超过到任一端点的距离；最近点落在段内
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-012 | REQ-005.4 | 点在段外一侧 | squared_distance_to_point | 非负且等于几何最短距离平方 |
| CASE-013 | REQ-001.1 | 两条线段 | nearest_points_on_line_segments | 两点分别落在各自段内 |

### API-007 Bezier 三次贝塞尔曲线

- 模块：MOD-001
- 对应需求：REQ-001.1、REQ-005.3
- 签名：

  ```
  pub struct Bezier { pub p0: Point, pub p1: Point, pub p2: Point, pub p3: Point }
  impl Bezier {
      pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self;
      pub fn point(&self, t: f32) -> Point;
      pub fn tangent(&self, t: f32) -> Vector;
      pub fn derivative_tangent(&self, t: f32) -> Vector;
      pub fn curvature(&self, t: f32) -> f32;
      pub fn split(&self, t: f32) -> (Bezier, Bezier);                    // wasm 不导出（元组返回，ADR-007）
      pub fn segment(&self, t0: f32, t1: f32) -> Bezier;
      pub fn midpoint(&self) -> Point;
  }
  ```

- requires：t 落在 [0,1]；segment 的 t1 不为 0、t0 不为 1
- ensures：split 的两段拼接后与原曲线一致；point(0)=p0、point(1)=p3
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Geometry | segment 分母退化 | 返回退化曲线，调用方按需求判断是否重采样 |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-014 | REQ-001.1 | 一条贝塞尔 | split(0.5) 后各自取端点 | 拼接端点与原曲线中点一致 |
| CASE-015 | REQ-005.3 | 一条贝塞尔 | segment(0.25,0.75) | 端点与 point(0.25)/point(0.75) 一致 |
| CASE-074 | REQ-001.1 | t0=1 或 t1=0 的退化区间 | segment | 返回退化曲线，不 panic |

### API-008 Arc 弧

- 模块：MOD-001
- 对应需求：REQ-001.1、REQ-005.2
- 签名：

  ```
  pub struct Arc { pub p0: Point, pub p1: Point, pub d: f32 }
  impl Arc {
      pub fn new(p0: Point, p1: Point, d: f32) -> Self;
      pub fn radius(&self) -> f32;
      pub fn center(&self) -> Point;
      pub fn angle(&self) -> f32;
      pub fn len(&self) -> f32;
      pub fn distance_to_point(&self, p: Point) -> f32;
      pub fn squared_distance_to_point(&self, p: Point) -> f32;
      pub fn tangents(&self) -> (Vector, Vector);                        // wasm 不导出（元组返回，ADR-007）
      pub fn extents(&self) -> Aabb;
      pub fn wedge_contains_point(&self, p: Point) -> bool;
      pub fn approximate_bezier(&self) -> Bezier;
      pub fn to_endpoint(&self) -> ArcEndpoint;
      pub fn from_endpoint(e: &ArcEndpoint) -> Self;
  }
  ```

- requires：p0 与 p1 不重合；d 为有限值且不为裸 NaN
- ensures：d=0 时等价线段；wedge_contains_point 对大弧（|d|>1）与小弧均与几何定义一致（REQ-005.2）；radius/center 对 d=0 返回线段语义
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Geometry | p0 与 p1 重合 | 构造返回退化弧；调用方应视为线段 |

- 顺序约束：无
- 性能：radius/center/len 为按需计算，无隐藏缓存

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-016 | REQ-005.2 | 大圆弧（d 绝对值 > 1）与其外侧一点 | wedge_contains_point | 按几何定义判定为不包含 |
| CASE-017 | REQ-001.1 | 弧与其端点 | from_endpoint(to_endpoint()) | 与原弧逐位一致 |
| CASE-075 | REQ-001.1 | p0 与 p1 重合 | Arc::new | 返回退化弧，不 panic |

### API-009 Aabb 包围盒

- 模块：MOD-001
- 对应需求：REQ-001.2、REQ-001.4
- 签名：

  ```
  pub struct Aabb { pub mins: Point, pub maxs: Point }
  impl Aabb {
      pub fn new(mins: Point, maxs: Point) -> Self;
      pub fn new_invalid() -> Self;                 // 空盒表示
      pub fn is_empty(&self) -> bool;
      pub fn extend(&mut self, p: Point);
      pub fn extend_by(&mut self, other: &Aabb);
      pub fn includes(&self, other: &Aabb) -> bool;
      pub fn width(&self) -> f32;
      pub fn height(&self) -> f32;
      pub fn scale(&mut self, s: f32);
      pub fn half(&self) -> (Aabb, Aabb);                            // wasm 不导出（元组返回，ADR-007）
      pub fn collision(&self, other: &Aabb) -> Option<Aabb>;
      pub fn bound(&self, dir: Direction) -> Segment;
      }
  pub enum Direction { Top, Bottom, Left, Right }
  ```

- requires：非空盒满足 mins 分量不大于 maxs 分量
- ensures：is_empty 对 mins 与 maxs 全为无穷的空盒返回真（修正现状只判单分量的缺陷）；extend 后仍满足顺序
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-018 | REQ-001.2 | 空盒 | is_empty | 返回真（两个分量都判定） |
| CASE-019 | REQ-001.2 | 一个盒与盒内一点 | includes | 返回真 |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/model/geom/primitive.rs | API-003、API-004、API-005、API-006 | 是 | |
| src/model/geom/primitive_inner.rs | API-003、API-004、API-005、API-006 | 否（实现） | |
| src/model/geom/curve.rs | API-001、API-007、API-008 | 是 | |
| src/model/geom/curve_inner.rs | API-001、API-007、API-008 | 否（实现） | |
| src/model/geom/aabb.rs | API-009 | 是 | |
| src/model/geom/aabb_inner.rs | API-009 | 否（实现） | |
