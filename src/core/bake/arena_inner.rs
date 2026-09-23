/**
 * 弧数据索引 arena（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/bake/arena.rs
 */

use super::arena::ArcArena;
use crate::model::raster::texture::UnitArc;

/**
 * 构造空 arena。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 */
pub fn arena_new() -> ArcArena {
    // ①键映射与池都为空：Default 即空 arena（无 unsafe、无裸指针）
    ArcArena::default()
}

/**
 * 以键登记单位弧。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 *
 * 参数：arena — 目标 arena
 * 参数：key — 单位弧内容的稳定哈希
 * 参数：unit — 待登记的单位弧
 */
pub fn arena_intern(arena: &mut ArcArena, key: u64, unit: UnitArc) -> usize {
    // ①查键映射命中则返回旧下标：同键去重、零克隆，且先到者内容不被覆盖
    if let Some(index) = arena.key_index.get(&key) {
        return *index;
    }
    // ②否则压入池取得新下标（下标即位置，池增长不会改变既有下标）
    let index = arena.pool.len();
    arena.pool.push(unit);
    // ③登记键到下标的映射
    arena.key_index.insert(key, index);
    // ④返回下标
    index
}

/**
 * 按下标读取单位弧。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 *
 * 参数：arena — 目标 arena
 * 参数：index — 单位弧下标
 */
pub fn arena_get(arena: &ArcArena, index: usize) -> Option<&UnitArc> {
    // ①按池下标取值 ②越界时 Vec::get 返回 None ③否则返回共享引用
    arena.pool.get(index)
}

/**
 * 按下标可变读取单位弧。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 *
 * 参数：arena — 目标 arena
 * 参数：index — 单位弧下标
 */
pub fn arena_get_mut(arena: &mut ArcArena, index: usize) -> Option<&mut UnitArc> {
    // ①按池下标取可变值 ②越界返回 None ③否则返回独占可变引用（借用检查杜绝别名）
    arena.pool.get_mut(index)
}

/**
 * 单位弧数量。
 *
 * 契约：API-018
 *
 * 约束：
 *   - requires  key 是单位弧内容的稳定哈希
 *   - ensures   无 unsafe、无悬垂、无同时可变别名；池增长后既有下标仍指向原元素；同一 key 重复 intern 返回同一下标；可整体序列化
 *   - 错误      InvalidParam —— 下标越界访问时 get 返回 None
 *
 * 参数：arena — 目标 arena
 */
pub fn arena_len(arena: &ArcArena) -> usize {
    // ①返回池长度
    arena.pool.len()
}

#[cfg(test)]
mod tests {
    use crate::core::bake::arena::ArcArena;
    use crate::model::geom::curve::ArcEndpoint;
    use crate::model::raster::texture::UnitArc;

    fn unit(px: f32, py: f32, d: f32) -> UnitArc {
        UnitArc {
            endpoints: vec![ArcEndpoint {
                px,
                py,
                d,
                tag: None,
            }],
            sdf_min: -1.0,
            sdf_max: 1.0,
            show: true,
        }
    }

    // CASE-034：同一 key 重复 intern 返回同一下标，池只增一条。
    #[test]
    fn case_034_same_key_dedups_to_single_slot() {
        let mut arena = ArcArena::new();
        let first = arena.intern(42, unit(0.0, 0.0, 0.0));
        let second = arena.intern(42, unit(9.0, 9.0, 0.5));
        assert_eq!(first, second, "同一 key 应返回同一下标");
        assert_eq!(arena.len(), 1, "同一 key 只应入池一次");

        // 先到者胜出：内容不被后到者覆盖，保证既有下标语义稳定。
        let stored = arena.get(first).expect("已登记下标应可取");
        assert_eq!(stored.endpoints[0].px, 0.0);

        // 不同 key 得到新下标。
        let third = arena.intern(43, unit(1.0, 2.0, 0.0));
        assert_ne!(third, first, "不同 key 应分配不同下标");
        assert_eq!(arena.len(), 2);
    }

    // CASE-035：下标越界访问返回 None，不 panic。
    #[test]
    fn case_035_out_of_range_get_is_none() {
        let arena = ArcArena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.get(0).is_none(), "空 arena 的 get(0) 应为 None");
        assert!(arena.get(usize::MAX).is_none(), "极大下标应为 None");
    }

    #[test]
    fn case_035_out_of_range_get_mut_is_none() {
        let mut arena = ArcArena::new();
        assert!(arena.get_mut(0).is_none(), "空 arena 的 get_mut(0) 应为 None");

        let index = arena.intern(7, unit(3.0, 4.0, 0.0));
        let slot = arena.get_mut(index).expect("落界下标应可取可变引用");
        slot.sdf_max = 5.0;
        assert_eq!(arena.get(index).map(|held| held.sdf_max), Some(5.0));
        assert!(arena.get_mut(index + 1).is_none(), "越界可变访问应为 None");
    }

    // 下标稳定：池多次增长（触发重分配）后，既有下标仍指向原元素。
    #[test]
    fn arena_indices_stay_stable_across_growth() {
        let mut arena = ArcArena::new();
        let mut indices = Vec::new();
        for key in 0..64u64 {
            indices.push(arena.intern(key, unit(key as f32, 0.0, 0.0)));
        }
        for (key, &index) in indices.iter().enumerate() {
            let held = arena.get(index).expect("既有下标在池增长后应仍有效");
            assert_eq!(held.endpoints[0].px, key as f32);
        }
        assert_eq!(arena.len(), 64);
    }
}
