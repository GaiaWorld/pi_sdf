---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 以 stable Rust 为基准而非 nightly

## Context and Problem Statement

现状以 nightly 编译（使用了 trait_alias 这类 unstable feature），影响工具链与持续集成的可复现性。需要确定工具链基准。

## Decision Drivers

- NFR-003

## Considered Options

- 以 stable 为基准
- 保持 nightly

## Decision Outcome

Chosen option: "以 stable 为基准"，because stable 是 nightly 的能力子集，stable 能编译的代码 nightly 亦能编译，去掉 unstable feature 可提升可复现性且无功能损失（原 unstable 用法可用等价 stable 写法替代）。

### Consequences

- Good, because 工具链可复现，不锁定 nightly 版本
- Good, because 可在更多环境构建
- Bad, because 需要重写少量依赖 unstable feature 的代码

## Pros and Cons of the Options

### stable

- Good, because 可复现、生态兼容性最好
- Bad, because 需改写现有 unstable 用法

### nightly

- Good, because 无需改写
- Bad, because 工具链不可复现，CI 与协作成本高

## Confirmation

- 三目标用 stable 工具链编译通过
