# 契约：距离场

**模块：** MOD-004

## 接口

### API-014 Sdf 弧列表距离场

- 模块：MOD-004
- 对应需求：REQ-001.1、REQ-001.3、REQ-005.4
- 签名：

  ```
  pub struct SdfSample { pub distance: f32, pub arc_index: Option<usize> }
  pub fn sdf_from_arcs(arcs: &[Arc], p: Point) -> SdfSample;
  pub fn sdf_from_indices(all: &[Arc], indices: &[usize], p: Point) -> Result<SdfSample>;
  ```

- requires：indices 中每个值均小于 all 的长度
- ensures：distance **符号正确**（内部为负或按约定，与现状一致且消除符号缺陷，REQ-005.4）；arc_index 指向产生最小距离的弧；distance 非 NaN
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | indices 越界 | 返回 Err；不得 panic |

- 顺序约束：无
- 性能：线性于弧数（与现状同阶）

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-027 | REQ-005.4 | 轮廓内一点 | sdf_from_arcs | distance 符号表示在内部 |
| CASE-028 | REQ-001.3 | 轮廓外一点 | sdf_from_arcs | distance 符号与内点相反，绝对值等于几何最短距离（容差内） |
| CASE-078 | REQ-004.3 | 含越界下标的 indices | sdf_from_indices | 返回 Err(InvalidParam)，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/core/sdf/sdf.rs | API-014 | 是 | |
| src/core/sdf/sdf_inner.rs | API-014 | 否（实现） | |
