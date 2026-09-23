# 契约：基础

**模块：** MOD-006

## 接口

### API-002 Error 与 ErrorKind（统一错误类型）

- 模块：MOD-006
- 对应需求：REQ-004.1、REQ-004.2、REQ-004.3、REQ-005.1
- 签名：

  ```
  pub enum Error {
      InvalidFont(&'static str),   // 字体字节非法或损坏
      InvalidPathVerb(u8),         // 路径动词判别值非法
      InvalidParam(&'static str),  // 参数越界（如纹理尺寸为 0）
      Decode(&'static str),        // 边界字节解码失败
      Encode(&'static str),        // 编码失败
      Geometry(&'static str),      // 几何前提不成立（如弧端点为 NaN）
  }
  impl std::error::Error for Error {}
  impl std::fmt::Display for Error {}
  #[cfg(target_arch = "wasm32")] impl From<Error> for wasm_bindgen::JsValue {}   // 转为 JS 字符串，作为 Result 的 Err
  pub type Result<T> = std::result::Result<T, Error>;
  ```

- requires：无
- ensures：Display 输出可读的中文或英文短语；构造与格式化均不 panic（REQ-004.3）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | （不适用） | Error 是错误载体本身，不产生新错误 | — |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-003 | REQ-004.2 | 一段损坏的字体字节 | 调用字体构造 | 返回 Err(InvalidFont)，宿主不 panic |
| CASE-004 | REQ-004.1 | 一段非法边界字节 | 调用边界解码 | 返回 Err(Decode)，wasm 不 trap |
| CASE-005 | REQ-004.3 | 纹理尺寸为 0 | 调用 SDF 生成 | 返回 Err(InvalidParam)，不进入 panic 路径 |
| CASE-083 | REQ-003.2 | 已完成的核心层源码 | 依赖图检查 | 模块依赖单向无环 |
| CASE-084 | REQ-003.3 | 已完成的核心层源码 | 职责检查 | 每个源文件只承担一个职责，职责混杂已拆分 |
| CASE-086 | NFR-003 | Windows / Android / wasm32 三目标 | 逐目标 cargo build | 三目标全部编译成功 |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/model/base/error.rs | API-002 | 是 | |
| src/model/base/error_inner.rs | API-002 | 否（实现） | |
