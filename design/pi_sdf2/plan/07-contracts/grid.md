# 契约：网格

**模块：** MOD-005

## 接口

### API-015 Cell 格元

- 模块：MOD-005
- 对应需求：REQ-001.2
- 签名：

  ```
  pub struct Cell { pub bounds: Aabb, pub arc_indices: Vec<usize> }
  ```

- requires：arc_indices 是全局弧列表的合法下标
- ensures：bounds 非空盒；索引去重且升序
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-029 | REQ-001.2 | 一个格元 | 检查 arc_indices | 无重复、均指向有效弧 |

### API-016 CellGrid 近邻弧网格

- 模块：MOD-005
- 对应需求：REQ-001.2、REQ-002.3
- 签名：

  ```
  pub struct CellGrid {
      pub extents: Aabb,
      pub arcs: Vec<Arc>,
      pub cells: Vec<Cell>,
      pub min_width: f32,
      pub min_height: f32,
      pub is_area: bool,
  }
  ```

- requires：cells 的索引均落在 arcs 范围内
- ensures：cells 覆盖 extents；near-arc 对格元内任意点的距离计算充分
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-030 | REQ-001.2 | 一份近邻弧网格 | 遍历 cells | 所有索引有效，覆盖 extents |
| CASE-031 | REQ-002.3 | 同一轮廓 | 分别经 native 与 wasm 计算网格 | 两侧格元划分与索引集合一致 |

### API-017 Subdivision 细分

- 模块：MOD-005
- 对应需求：REQ-001.2、REQ-005.1
- 签名：

  ```
  pub fn compute_cell_grid(outline: &Outline, scale: f32, is_area: bool) -> Result<CellGrid>
  ```

- requires：scale 为正有限值
- ensures：**终止性**——细分必然结束，不得出现无终止的填充循环（修正现状死循环缺陷，REQ-005.1）；结果与现状格元划分一致
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | scale 非正 | 返回 Err |

- 顺序约束：无
- 性能：与现状同阶（四叉细分线性于叶格元数）

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-032 | REQ-005.1 | 任一合法轮廓 | compute_cell_grid | 有限时间内返回，无死循环 |
| CASE-033 | REQ-001.2 | 与 pi_sdf 同一输入 | compute_cell_grid | 格元划分与弧索引集合一致 |
| CASE-079 | REQ-004.3 | scale 为 0 或负 | compute_cell_grid | 返回 Err(InvalidParam)，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/core/grid/grid.rs | API-015、API-016、API-017 | 是 | |
| src/core/grid/grid_inner.rs | API-015、API-016、API-017 | 否（实现） | |
