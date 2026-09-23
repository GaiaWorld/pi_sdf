---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 用索引 arena 而非裸指针共享弧数据

## Context and Problem Statement

现状把弧单元以裸指针存入去重表并跨函数伪造成静态生命周期，存在多处同时可变别名；底层容器一旦重分配指针即悬垂。需要在不增加空间复杂度的前提下消除未定义行为。

## Decision Drivers

- NFR-002
- REQ-004
- NFR-001

## Considered Options

- 索引 arena：弧存平坦池，去重表存「键 → 下标」
- Rc<RefCell> 共享
- 保留裸指针并补充文档契约
- 每次深拷贝弧数据

## Decision Outcome

Chosen option: "索引 arena"，because 键到下标的映射与裸指针同为常数时间查找且零克隆，同时天然可序列化、缓存更友好。

### Consequences

- Good, because 彻底消除悬垂与别名 UB，且查找不慢于裸指针
- Good, because 结构可整体序列化，便于跨边界传递
- Bad, because 需要一次性重排数据布局

## Pros and Cons of the Options

### 索引 arena

- Good, because 零克隆、常数时间查找，空间复杂度不变
- Good, because 生命期由容器拥有，无跨函数所有权问题
- Bad, because 需要改造现有的数据组织方式

### Rc<RefCell>

- Good, because 语义安全、改动相对直接
- Bad, because 运行时借用检查与引用计数带来开销，热路径不划算

### 保留裸指针 + 文档契约

- Good, because 完全不动
- Bad, because UB 依旧存在，且不可序列化

### 深拷贝

- Good, because 实现最简单
- Bad, because 空间与时间开销随弧数线性上升，可能劣化 NFR-001

## Confirmation

- 算法核心检索不到 unsafe
- 去重前后纹理数据一致，基准测试不低于基线
