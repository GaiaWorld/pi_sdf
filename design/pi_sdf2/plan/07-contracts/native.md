# 契约：native 后端

**模块：** MOD-014

## 接口

### API-030 FontSource 字体来源

- 模块：MOD-014
- 对应需求：REQ-002.1、REQ-003.1、REQ-004.2
- 签名：

  ```
  pub enum FontSource {
      File(std::path::PathBuf),
      SystemFamily(String),
  }
  impl FontSource {
      pub fn load(&self) -> Result<Vec<u8>>;
  }
  pub fn system_font_available() -> bool;
  ```

- requires：File 变体路径存在；SystemFamily 变体在平台上可解析
- ensures：平台相关代码只在此模块内（Windows 用 dwrote、Android 用 ndk，各自隔离）；在无实现的平台返回 Err，而不导致编译失败（修正现状其它平台编译失败）；读到的字体字节可直接交给 API-024
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidFont | 路径不存在或系统族解析失败 | 返回 Err，调用方回退到内置字体 |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-062 | REQ-002.1 | Windows 环境 | FontSource::SystemFamily 加载 | 返回可用字体字节 |
| CASE-063 | REQ-004.2 | 不存在的文件路径 | load | 返回 Err(InvalidFont) |
| CASE-064 | REQ-003.1 | 算法核心源码 | 检索 platform 引用 | 无 |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/platform/native/font_source.rs | API-030 | 是 | |
| src/platform/native/font_source_inner.rs | API-030 | 否（实现） | |
