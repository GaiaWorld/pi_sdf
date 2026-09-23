---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 用单 crate 三层分层而非多 crate workspace 组织 native 与 wasm

## Context and Problem Statement

现状把同一功能写成 native 与 wasm 两套方法（后缀 _of_wasm），并在边界手工序列化，逻辑重复、边界模糊。需要在保持单一发布物的前提下，让平台相关代码与算法代码分离。

## Decision Drivers

- REQ-002
- REQ-003
- NFR-004
- 来自 D0 的硬约束：crate-type 为 cdylib + rlib，需同时产出 wasm 与 native 库

## Considered Options

- 单 crate 三层：算法核心 / 统一接口 / 平台后端（含边界壳）
- workspace 多 crate：core / native / wasm 各一个
- 单层 + 条件编译收敛

## Decision Outcome

Chosen option: "单 crate 三层分层"，because 它在不增加构建与发布复杂度的前提下，把平台相关代码收敛到一个后端目录，且算法核心的纯净性可由检索机械验证。

### Consequences

- Good, because 发布物仍是单个 crate，现有 cargo 用法不变
- Good, because 算法核心可被脚本验证「无平台条件编译、无 unsafe」
- Bad, because 平台无关性靠约定与检索保证，编译期不强制
- Bad, because 需要建立并长期维持一条分层纪律

## Pros and Cons of the Options

### 单 crate 三层

- Good, because 构建与发布最简单，三目标共用一份 Cargo.toml
- Good, because 边界壳集中在一处，重复方法可整体消除
- Bad, because core 的纯净性是约定而非编译期约束

### workspace 多 crate

- Good, because core 不含任何平台代码可由编译器强制
- Bad, because 需要多份 Cargo.toml 与版本管理，发布复杂度上升
- Bad, because 现有单 crate 用法需拆解

### 单层 + 条件编译收敛

- Good, because 改动最小
- Bad, because 内部边界仍模糊，不满足 REQ-003

## Confirmation

- 对算法核心检索平台条件编译、unsafe、wasm_bindgen，应零命中
- Windows / Android / wasm32 三目标编译通过
