---
status: accepted
date: 2026-09-23
decision-makers: []
consulted: []
informed: []
---

# wasm 边界投影按成员级规则裁剪，native 接口保持全量

## Context and Problem Statement

`wasm_bindgen` 的表达能力有硬边界：不支持带数据的 enum、元组返回值与借用返回；`pub` 字段的 getter 要求字段类型 `Copy`，故 `Vec<T>` 字段无法直接导出。返工形成的「边界类型清单」（plan/05-architecture.md）只到**类型级**（"导出为 JS 类"），不足以落地——骨架按其加注解后，wasm32 目标产生 41 个编译错误。需要一条可机械执行的**成员级**规则，在保持 native 接口全量的同时让 wasm32 可编译。

## Decision Drivers

- REQ-002.1、REQ-002.2、REQ-002.3
- REQ-003.1（导出注解仅允许出现在 model 与 api）
- NFR-003（三目标编译）

## Considered Options

- 按成员级规则裁剪导出面（拆 impl 块 + 字段 skip + getter）
- 把类型定义统一改成两边都能表达的形态（如全部 opaque 类）
- 为 wasm 单独维护一套并行类型与实现

## Decision Outcome

Chosen option: "按成员级规则裁剪导出面"，because 它不为两种目标维护两份定义（满足 REQ-002.3），也不牺牲 native 的 Rust 接口（元组返回、借用返回、带数据 enum 在 native 全部保留），代价仅是 JS 侧看不到个别非核心链路成员。

### 成员级规则

| wasm_bindgen 限制 | 落地规则 |
|---|---|
| 不支持带数据的 enum | 改为 opaque 类：`pub(crate)` 内部 enum + 静态构造器（如 `Shape`） |
| 不支持元组返回 | 该成员移入**无导出注解**的 impl 块；native 保留，JS 不导出 |
| 不支持借用返回 | 同上 |
| `pub` 字段 getter 要求 `Copy`（`Vec<T>` 不满足） | 字段加 `wasm_bindgen(skip)`；同名 `#[wasm_bindgen(getter)]` 方法返回克隆，JS 侧访问形式不变 |
| 带数据的错误 enum 不能作 `Result::Err` | `impl From<Error> for JsValue`（转字符串），不导出 enum 本身 |
| 核心链路之外的几何工具方法 | 允许 JS 不导出 |

判定基准是 REQ-002.1 的核心链路：字体 → 轮廓 → 近邻弧 → SDF 纹理。当前允许 JS 不导出的成员为 `Aabb::half`、`Bezier::split`、`Arc::tangents`、`Segment::nearest_points_on_line_segments`、`Line::normal`。

### Consequences

- Good, because wasm32 与 native 共用同一份类型定义即可编译（REQ-002.3）
- Good, because native 的 Rust 接口不打折
- Bad, because 壳文件按「可导出 / 不可导出」拆成多个 impl 块，冻结面结构变复杂
- Bad, because JS 侧看不到若干几何工具方法（已在边界类型清单登记）

## Pros and Cons of the Options

### 按成员级规则裁剪导出面

- Good, because 改动集中在 model 与 api 的注解层，数据定义与算法不变
- Bad, because 壳里出现两个 impl 块，需要约定与检索

### 全部改为 opaque 类

- Good, because 单一形态，注解简单
- Bad, because native 侧丢失模式匹配与元组返回，Rust 调用方大面积破坏（违背"老调用默认不改"）

### 为 wasm 单独维护一套并行定义

- Good, because 两侧各自最优
- Bad, because 违反 REQ-002.3（不得维护两份并行等价实现）

## Confirmation

- `cargo build`（x86_64-pc-windows-msvc）与 `cargo build --target wasm32-unknown-unknown` 均退出码 0
- 导出注解只出现在 model 与 api（REQ-003.1 检索）
