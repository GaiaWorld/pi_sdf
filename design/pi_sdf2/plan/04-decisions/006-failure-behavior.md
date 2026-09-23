---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 对外入口返回错误而非 panic

## Context and Problem Statement

现状热路径以断言与解包驱动流程，不可信输入（平台边界字节、字体字节、非法参数）会导致 panic 或模块 trap，宿主无法恢复。

## Decision Drivers

- REQ-004
- NFR-002

## Considered Options

- 对外入口返回错误类型
- 保持现状 panic
- 全路径禁止 panic

## Decision Outcome

Chosen option: "对外入口返回错误类型、内部不变量仅在 debug 断言"，because 调用方能捕获并降级，release 不在宿主崩溃，且不必为内部不变量付出运行时检查成本。

### Consequences

- Good, because 不可信输入不再崩宿主
- Good, because release 无额外运行时开销
- Bad, because 需要为公开接口统一错误类型
- Bad, because 内部不变量违背在 release 下退化为逻辑错误（靠 debug 断言暴露）

## Pros and Cons of the Options

### 对外返回错误类型

- Good, because 调用方可控，边界安全
- Bad, because 需要统一错误类型并改造签名

### 保持 panic

- Good, because 零改动
- Bad, because 宿主崩溃，违反 REQ-004

### 全路径禁止 panic

- Good, because 最彻底
- Bad, because 实现成本高，内部不变量检查拖累热路径

## Confirmation

- 对非法平台字节、损坏字体、越界参数分别注入，均返回错误且模块不 trap
