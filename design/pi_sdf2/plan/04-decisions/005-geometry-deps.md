---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 不引入替代几何库，评估整体去除 parry2d / nalgebra

## Context and Problem Statement

parry2d 与 nalgebra 属重量级依赖，现状用于点、向量、包围盒与矩阵类型。用户明确不允许替换为其它几何库，但允许在可以去掉时整体去除。

## Decision Drivers

- NFR-001
- NFR-003

## Considered Options

- 保留现状
- 整体去除（自研最小几何类型）
- 替换为轻量几何库

## Decision Outcome

Chosen option: "不引入替代库；能整体去除则去除，否则保留"，because 用户不允许替换为别的库；若这些依赖仅用于基础几何与矩阵，可用自研最小类型替代从而缩小依赖树。

### Consequences

- Good, because 依赖树更小、供应链风险更低
- Good, because 不再为少量基础类型背负重量级依赖
- Bad, because 去除需要自研类型并迁移调用点
- Bad, because 若存在难以自研的能力，则仍需保留该依赖

## Pros and Cons of the Options

### 保留现状

- Good, because 零迁移成本
- Bad, because 重量级依赖长期存在

### 整体去除（自研最小类型）

- Good, because 依赖最小、可控
- Bad, because 需要自研与迁移，且要覆盖原有能力

### 替换为轻量几何库

- Good, because 兼顾轻量与现成
- Bad, because 用户明确不允许替换

## Confirmation

- 编译产物依赖列表中原有重量级依赖是否消失
- 基准测试不低于基线
