# 契约：wasm 后端

**模块：** MOD-011

## 接口

### API-031 Exports wasm 导出边界壳

- 模块：MOD-011
- 对应需求：REQ-002.2、REQ-004.1、REQ-006.1
- 签名：

  ```
  // 均以字节数组进出；Err 映射到 JS 异常通道
  pub fn compute_near_arcs_of_wasm(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn compute_sdf_tex_of_wasm(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn get_char_arc_debug(bytes: &[u8]) -> Result<Vec<u8>>;
  pub fn compute_svg_debug(bytes: &[u8]) -> Result<Vec<u8>>;
  ```

- requires：bytes 是由 Codec 编码的合法载荷
- ensures：每个导出只做「解码 → 调用接口层 → 编码」三步，不含任何算法（TERM-023）；非法字节返回 Err，wasm 模块不 trap（REQ-004.1）；get_char_arc_debug 与 compute_svg_debug 可用并返回可展示数据（修复现状引用不存在的导出，REQ-006.1）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Decode | 边界字节无法解析 | 返回 Err（映射为 JS 异常），不 trap |
  | InvalidParam | 载荷内字段越界 | 返回 Err |

- 顺序约束：无（各导出相互独立）
- 性能：编解码开销线性于载荷大小

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-065 | REQ-004.1 | 一段乱码字节 | compute_near_arcs_of_wasm | 返回 Err，模块不 trap |
| CASE-066 | REQ-006.1 | 合法字符载荷 | get_char_arc_debug | 返回可展示的轮廓/近邻弧数据 |
| CASE-067 | REQ-006.1 | 合法 SVG 载荷 | compute_svg_debug | 返回可展示的 SDF 数据 |
| CASE-068 | REQ-002.2 | wasm32 目标 | 编译并调用导出 | 核心链路接口可用 |

### API-032 Codec 边界编解码

- 模块：MOD-011
- 对应需求：REQ-002.2、REQ-004.1
- 签名：

  ```
  pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>>;
  pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>;
  ```

- requires：T 实现序列化框架的对应特征
- ensures：bitcode 编解码只出现在边界壳（ADR-003），核心层与接口层不含编解码；解码失败返回 Err（替换现状 deserialize().unwrap()）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Decode | 字节不合法或结构不符 | 返回 Err |
  | Encode | 序列化失败 | 返回 Err |

- 顺序约束：无
- 性能：线性于载荷大小

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-069 | REQ-002.2 | 任一可序列化结构 | encode → decode | 与输入一致 |
| CASE-070 | REQ-004.1 | 随机字节 | decode | 返回 Err，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/platform/wasm/exports.rs | API-031 | 是 | |
| src/platform/wasm/exports_inner.rs | API-031 | 否（实现） | |
| src/platform/wasm/codec.rs | API-032 | 是 | |
| src/platform/wasm/codec_inner.rs | API-032 | 否（实现） | |
