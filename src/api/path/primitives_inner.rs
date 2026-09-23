/**
 * 路径动词与图元（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/api/path/primitives.rs
 */

use super::{PathVerb, Shape};
use crate::core::base::error::{Error, Result};
use crate::core::outline::contour::Outline;
use crate::core::geom::aabb::Aabb;

/**
 * 由判别值还原路径动词。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 *
 * 参数：value — 路径动词判别值
 */
pub fn path_verb_try_from(value: u8) -> Result<PathVerb> {
    // TODO-DECL —— 实现逻辑：①按 1..=19 逐一匹配 ②非法返回 InvalidPathVerb ③否则返回动词
    todo!("TODO-DECL")
}

/**
 * 取路径动词判别值。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 *
 * 参数：verb — 路径动词
 */
pub fn path_verb_to_u8(verb: PathVerb) -> u8 {
    // TODO-DECL —— 实现逻辑：①按枚举判别值返回 u8
    todo!("TODO-DECL")
}

/**
 * 图形元转轮廓。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 *
 * 参数：shape — 图形元
 */
pub fn shape_to_outline(shape: &Shape) -> Result<Outline> {
    // TODO-DECL —— 实现逻辑：①校验尺寸合法 ②按图元类型离散为弧 ③组轮廓返回
    todo!("TODO-DECL")
}

/**
 * 图形元包围盒。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 *
 * 参数：shape — 图形元
 */
pub fn shape_extents(shape: &Shape) -> Aabb {
    // TODO-DECL —— 实现逻辑：①按图元类型求边界 ②组包围盒返回
    todo!("TODO-DECL")
}

/**
 * 判定是否为面积图元。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 *
 * 参数：shape — 图形元
 */
pub fn shape_is_area(shape: &Shape) -> bool {
    // TODO-DECL —— 实现逻辑：①线段类返回假 ②其余返回真
    todo!("TODO-DECL")
}
