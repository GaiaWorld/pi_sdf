# 契约：字体

**模块：** MOD-008

## 接口

### API-024 FontFace 字体面与轮廓提取

- 模块：MOD-008
- 对应需求：REQ-001.1、REQ-002.1、REQ-002.2、REQ-002.3、REQ-004.2
- 签名：

  ```
  pub struct FontFace { /* 字体数据与解析状态 */ }
  impl FontFace {
      pub fn from_bytes(data: Vec<u8>) -> Result<Self>;
      pub fn glyph_index(&self, ch: char) -> u32;
      pub fn glyph_outline(&self, ch: char) -> Result<Outline>;
      pub fn outline_of_glyph_index(&self, index: u32) -> Result<Outline>;
      pub fn units_per_em(&self) -> f32;
  }
  ```

- requires：data 为字体文件字节（ttf / otf / ttc）
- ensures：**native 与 wasm 共用同一签名与同一实现**（REQ-002.3），不再有 _of_wasm 平行方法；非法或损坏字节返回 Err，不 panic / 不 UB（替换现状 transmute，REQ-004.2）；返回轮廓与字体字形数据一致（TERM-018）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidFont | 字体字节非法或损坏 | 返回 Err，调用方降级或提示 |
  | InvalidParam | 字形索引不存在 | 返回 Err |

- 顺序约束：无
- 性能：解析后轮廓提取线性于轮廓弧数

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-048 | REQ-004.2 | 一段损坏字体字节 | FontFace::from_bytes | 返回 Err(InvalidFont)，不 panic |
| CASE-049 | REQ-001.1 | 合法字体与字符 | glyph_outline | 端点数量与坐标与 pi_sdf 一致（容差内） |
| CASE-050 | REQ-002.3 | 同一字体与字符 | native 与 wasm 分别提取 | 轮廓一致 |
| CASE-051 | REQ-002.1 | native 目标 | 编译并调用 glyph_outline | 接口可用 |

### API-025 Metrics 字形度量

- 模块：MOD-008
- 对应需求：REQ-001.1
- 签名：

  ```
  pub struct GlyphMetrics { pub advance: f32, pub extents: Aabb, pub is_clockwise: bool }
  impl FontFace {
      pub fn glyph_metrics(&self, ch: char) -> Result<GlyphMetrics>;
      pub fn ascender(&self) -> f32;
      pub fn descender(&self) -> f32;
  }
  ```

- requires：字符存在于字体中
- ensures：advance ≥ 0；units_per_em > 0；extents 为该字形轮廓包围盒
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 字符无对应字形 | 返回 Err |

- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-052 | REQ-001.1 | 合法字体与字符 | glyph_metrics | advance 与 pi_sdf 一致，非负 |
| CASE-081 | REQ-004.3 | 字体中不存在的字符 | glyph_metrics | 返回 Err(InvalidParam)，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/api/font/font_face.rs | API-024 | 是 | |
| src/api/font/font_face_inner.rs | API-024 | 否（实现） | |
| src/api/font/metrics.rs | API-025 | 是 | |
| src/api/font/metrics_inner.rs | API-025 | 否（实现） | |
