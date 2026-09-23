/**
 * 包围盒（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/geom/aabb.rs
 */

use super::{Aabb, Direction};
use crate::model::geom::primitive::{Point, Segment};

/**
 * 由两点构造盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：mins — 最小点
 * 参数：maxs — 最大点
 */
pub fn aabb_new(mins: Point, maxs: Point) -> Aabb {
    Aabb { mins, maxs }
}

/**
 * 构造空盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 */
pub fn aabb_new_invalid() -> Aabb {
    Aabb {
        mins: Point {
            x: f32::INFINITY,
            y: f32::INFINITY,
        },
        maxs: Point {
            x: f32::INFINITY,
            y: f32::INFINITY,
        },
    }
}

/**
 * 判定空盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 */
pub fn aabb_is_empty(b: &Aabb) -> bool {
    let min_x_inf = crate::model::base::num::is_inf(b.mins.x);
    let min_y_inf = crate::model::base::num::is_inf(b.mins.y);
    min_x_inf || min_y_inf || b.mins.x > b.maxs.x || b.mins.y > b.maxs.y
}

/**
 * 扩张盒以包含点。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：p — 待包含的点
 */
pub fn aabb_extend(b: &mut Aabb, p: Point) {
    if aabb_is_empty(b) {
        b.mins = p;
        b.maxs = p;
        return;
    }
    b.mins.x = b.mins.x.min(p.x);
    b.mins.y = b.mins.y.min(p.y);
    b.maxs.x = b.maxs.x.max(p.x);
    b.maxs.y = b.maxs.y.max(p.y);
}

/**
 * 扩张盒以包含另一个盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：other — 另一个盒
 */
pub fn aabb_extend_by(b: &mut Aabb, other: &Aabb) {
    if aabb_is_empty(other) {
        return;
    }
    aabb_extend(b, other.mins);
    aabb_extend(b, other.maxs);
}

/**
 * 判定包含另一个盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：other — 待判定的盒
 */
pub fn aabb_includes(b: &Aabb, other: &Aabb) -> bool {
    b.mins.x <= other.mins.x
        && b.mins.y <= other.mins.y
        && other.maxs.x <= b.maxs.x
        && other.maxs.y <= b.maxs.y
}

/**
 * 盒宽度。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 */
pub fn aabb_width(b: &Aabb) -> f32 {
    b.maxs.x - b.mins.x
}

/**
 * 盒高度。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 */
pub fn aabb_height(b: &Aabb) -> f32 {
    b.maxs.y - b.mins.y
}

/**
 * 按比例缩放盒。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：s — 缩放比例，正数
 */
pub fn aabb_scale(b: &mut Aabb, s: f32) {
    let cx = (b.mins.x + b.maxs.x) * 0.5;
    let cy = (b.mins.y + b.maxs.y) * 0.5;
    let hw = (b.maxs.x - b.mins.x) * 0.5 * s;
    let hh = (b.maxs.y - b.mins.y) * 0.5 * s;
    b.mins = Point {
        x: cx - hw,
        y: cy - hh,
    };
    b.maxs = Point {
        x: cx + hw,
        y: cy + hh,
    };
}

/**
 * 沿长边一分为二。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 */
pub fn aabb_half(b: &Aabb) -> (Aabb, Aabb) {
    let w = b.maxs.x - b.mins.x;
    let h = b.maxs.y - b.mins.y;
    if w >= h {
        let mx = (b.mins.x + b.maxs.x) * 0.5;
        (
            Aabb {
                mins: b.mins,
                maxs: Point {
                    x: mx,
                    y: b.maxs.y,
                },
            },
            Aabb {
                mins: Point {
                    x: mx,
                    y: b.mins.y,
                },
                maxs: b.maxs,
            },
        )
    } else {
        let my = (b.mins.y + b.maxs.y) * 0.5;
        (
            Aabb {
                mins: b.mins,
                maxs: Point {
                    x: b.maxs.x,
                    y: my,
                },
            },
            Aabb {
                mins: Point {
                    x: b.mins.x,
                    y: my,
                },
                maxs: b.maxs,
            },
        )
    }
}

/**
 * 与另一个盒的交集。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：other — 另一个盒
 */
pub fn aabb_collision(b: &Aabb, other: &Aabb) -> Option<Aabb> {
    let minx = b.mins.x.max(other.mins.x);
    let miny = b.mins.y.max(other.mins.y);
    let maxx = b.maxs.x.min(other.maxs.x);
    let maxy = b.maxs.y.min(other.maxs.y);
    if minx <= maxx && miny <= maxy {
        return Some(Aabb {
            mins: Point { x: minx, y: miny },
            maxs: Point { x: maxx, y: maxy },
        });
    }
    None
}

/**
 * 取某条边。
 *
 * 契约：API-009
 *
 * 约束：
 *   - requires  非空盒满足 mins 分量不大于 maxs 分量
 *   - ensures   is_empty 对 mins 与 maxs 全为无穷的空盒返回真
 *   - 错误      无
 *
 * 参数：b — 盒
 * 参数：dir — 边的方向
 */
pub fn aabb_bound(b: &Aabb, dir: Direction) -> Segment {
    match dir {
        Direction::Top => Segment::new(b.mins, Point { x: b.maxs.x, y: b.mins.y }),
        Direction::Bottom => Segment::new(Point { x: b.mins.x, y: b.maxs.y }, b.maxs),
        Direction::Left => Segment::new(b.mins, Point { x: b.mins.x, y: b.maxs.y }),
        Direction::Right => Segment::new(Point { x: b.maxs.x, y: b.mins.y }, b.maxs),
    }
}
