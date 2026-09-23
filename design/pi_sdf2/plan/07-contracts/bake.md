# 契约：烘焙

**模块：** MOD-006

## 接口

### API-018 ArcArena 弧数据索引 arena

- 模块：MOD-006
- 对应需求：NFR-002、REQ-004.3、REQ-001.2
- 签名：

  ```
  pub struct ArcArena { /* 平坦弧池 + 键到下标的去重表 */ }
  impl ArcArena {
      pub fn new() -> Self;
      pub fn intern(&mut self, key: u64, unit: UnitArc) -> usize;  // 键已存在则返回既有下标
      pub fn get(&self, index: usize) -> Option<&UnitArc>;
      pub fn get_mut(&mut self, index: usize) -> Option<&mut UnitArc>;
      pub fn len(&self) -> usize;
  }
  ```

- requires：key 是单位弧内容的稳定哈希
- ensures：**以索引而非裸指针共享**（ADR-002），无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | InvalidParam | 下标越界访问 | get 返回 None，不 panic |

- 顺序约束：无
- 性能：intern 与 get 均为均摊 O(1)（哈希表 + 池追加），空间与现状裸指针方案同阶

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-034 | REQ-001.2 | 同一 key 的两条单位弧 | 连续 intern | 返回同一下标，池只增一条 |
| CASE-035 | REQ-004.3 | 空 arena | get(0) | 返回 None，不 panic |

### API-019 UnitArc 单位弧

- 模块：MOD-006
- 对应需求：REQ-001.2
- 签名：

  ```
  pub struct UnitArc {
      pub endpoints: Vec<ArcEndpoint>,
      pub sdf_min: f32,
      pub sdf_max: f32,
      pub show: bool,
  }
  ```

- requires：endpoints 非空；sdf_min ≤ sdf_max
- ensures：距离区间有序（TERM-017）
- 错误模式：无
- 顺序约束：无
- 性能：无

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-036 | REQ-001.2 | 一条单位弧 | 检查区间 | sdf_min ≤ sdf_max |

### API-020 DataTexture 数据纹理

- 模块：MOD-006
- 对应需求：REQ-001.3
- 签名：

  ```
  pub struct DataTexture { pub pixels: Vec<u8>, pub width: u32, pub height: u32 }
  pub fn encode_data_texture(arena: &ArcArena) -> Result<DataTexture>
  ```

- requires：arena 中的端点坐标落在量化范围内
- ensures：每条记录可解码为弧端点（TERM-015）；不足 3 端点的记录补终止符；量化误差不超过半个量化步长（TERM-013）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Encode | 量化溢出 | 返回 Err，不 panic（修正现状溢出 panic） |

- 顺序约束：无
- 性能：线性于记录数

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-037 | REQ-001.3 | 一份 arena | encode → 解码 | 端点与输入一致（量化容差内） |
| CASE-038 | REQ-001.3 | 坐标超出量化范围的记录 | encode_data_texture | 返回 Err，不 panic |

### API-021 IndexTexture 索引纹理

- 模块：MOD-006
- 对应需求：REQ-001.3、REQ-005.1
- 签名：

  ```
  pub struct IndexTexture { pub pixels: Vec<u8>, pub width: u32, pub height: u32 }
  pub fn encode_index_texture(grid: &CellGrid, data: &DataTexture) -> Result<IndexTexture>
  ```

- requires：grid 的每个单位弧已写入 data
- ensures：索引指向有效数据纹理位置（TERM-016）；**填充循环保证终止**（修正现状死循环，REQ-005.1）
- 错误模式：

  | 错误 | 触发条件 | 调用方应做什么 |
  |---|---|---|
  | Encode | 偏移超出 14 位位宽 | 返回 Err，不 panic（修正现状溢出 panic） |

- 顺序约束：必须在 encode_data_texture 之后调用
- 性能：线性于格元数

#### 用例

| ID | 关联 | 前置 | 动作 | 期望 |
|---|---|---|---|---|
| CASE-039 | REQ-005.1 | 一份网格与数据纹理 | encode_index_texture | 有限时间内返回，无死循环 |
| CASE-040 | REQ-001.3 | 一份索引纹理 | 逐项解引用 | 每个索引指向有效数据位置 |
| CASE-080 | REQ-001.3 | 偏移超出 14 位位宽 | encode_index_texture | 返回 Err(Encode)，不 panic |

## 未实现标记

| 位置 | 标签 | 原因 |
|---|---|---|
| 全部实现体 | TODO-DECL | 由 D7b 落骨架，实现在后续阶段 |

## 落地到 src/

| 文件 | 对应 API | 冻结 | 已确认 |
|---|---|---|---|
| src/core/bake/arena.rs | API-018、API-019 | 是 | |
| src/core/bake/arena_inner.rs | API-018、API-019 | 否（实现） | |
| src/core/bake/texture.rs | API-020、API-021 | 是 | |
| src/core/bake/texture_inner.rs | API-020、API-021 | 否（实现） | |
