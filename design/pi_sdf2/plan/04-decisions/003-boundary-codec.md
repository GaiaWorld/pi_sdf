---
status: accepted
date: 2026-09-22
decision-makers: []
consulted: []
informed: []
---

# 保留 bitcode 作为平台边界编解码

## Context and Problem Statement

跨 wasm 边界需要传递复杂结构（轮廓、近邻弧网格）。现状以 bitcode 编解码并手写 Visitor，重复且易错。需要选定一种边界编解码方案。

## Decision Drivers

- REQ-002
- REQ-006

## Considered Options

- 保留 bitcode
- serde-wasm-bindgen（JSON）
- 手写 typed array 编解码

## Decision Outcome

Chosen option: "保留 bitcode"，because 它是紧凑二进制、与序列化框架兼容，可把手写 Visitor 换成自动派生，改动面最小且体积与速度优于文本编码。

### Consequences

- Good, because 二进制紧凑、编解码快，适合高频边界传递
- Good, because 可去除手写 Visitor，减少出错面
- Bad, because 需要管理跨版本的结构兼容
- Bad, because JS 侧仍需按字节数组处理

## Pros and Cons of the Options

### 保留 bitcode

- Good, because 体积小、速度快，现状已验证
- Bad, because 非自描述格式，跨版本兼容需自行约定

### serde-wasm-bindgen（JSON）

- Good, because JS 侧可读性最好
- Bad, because 体积大、编解码慢，边界高频调用不划算

### 手写 typed array

- Good, because 体积与速度最优
- Bad, because 维护成本高、易错

## Confirmation

- 边界往返数据与原实现等价
- 编解码耗时不超过现状
