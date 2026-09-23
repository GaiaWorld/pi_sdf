/**
 * 弧数据索引 arena 与单位弧
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use std::collections::HashMap;
use crate::core::geom::ArcEndpoint;
use crate::core::bake::arena_inner;

/**
 * 单位弧：烘焙时归属某个格元的一条弧记录。
 *
 * 契约：API-019
 *
 * 约束：
 *   - requires  endpoints 非空；sdf_min 不大于 sdf_max
 *   - ensures   距离区间有序
 *   - 错误      无
 */
#[derive(Debug, Clone)]
pub struct UnitArc {
    /// 该单位弧包含的弧端点集合，非空
    pub endpoints: Vec<ArcEndpoint>,
    /// 距离区间下限
    pub sdf_min: f32,
    /// 距离区间上限
    pub sdf_max: f32,
    /// 是否参与渲染
    pub show: bool,
}

/**
 * 弧数据的索引 arena：以索引而非裸指针共享单位弧。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 */
#[derive(Debug, Default, Clone)]
pub struct ArcArena {
    key_index: HashMap<u64, usize>,
    pool: Vec<UnitArc>,
}

impl ArcArena {
    /// 构造空 arena。
    ///
    /// 契约：API-018
    ///
    /// 约束：
    ///   - requires  key 是单位弧内容的稳定哈希
    ///   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
    ///   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
    pub fn new() -> Self {
        arena_inner::arena_new()
    }

    /// 以键登记一条单位弧，返回其下标。
    ///
    /// 契约：API-018
    ///
    /// 约束：
    ///   - requires  key 是单位弧内容的稳定哈希
    ///   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
    ///   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
    ///
    /// 参数：key — 单位弧内容的稳定哈希
    /// 参数：unit — 待登记的单位弧
    pub fn intern(&mut self, key: u64, unit: UnitArc) -> usize {
        arena_inner::arena_intern(self, key, unit)
    }

    /// 按下标读取单位弧。
    ///
    /// 契约：API-018
    ///
    /// 约束：
    ///   - requires  key 是单位弧内容的稳定哈希
    ///   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
    ///   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
    ///
    /// 参数：index — 单位弧下标
    pub fn get(&self, index: usize) -> Option<&UnitArc> {
        arena_inner::arena_get(self, index)
    }

    /// 按下标可变读取单位弧。
    ///
    /// 契约：API-018
    ///
    /// 约束：
    ///   - requires  key 是单位弧内容的稳定哈希
    ///   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
    ///   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
    ///
    /// 参数：index — 单位弧下标
    pub fn get_mut(&mut self, index: usize) -> Option<&mut UnitArc> {
        arena_inner::arena_get_mut(self, index)
    }

    /// 单位弧数量。
    ///
    /// 契约：API-018
    ///
    /// 约束：
    ///   - requires  key 是单位弧内容的稳定哈希
    ///   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
    ///   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
    pub fn len(&self) -> usize {
        arena_inner::arena_len(self)
    }
}
