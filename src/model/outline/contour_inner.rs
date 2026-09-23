/**
 * 轮廓与子轮廓、绕向判定（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/outline/contour.rs
 */

use super::{Contour, Outline};
use crate::model::base::num;
use crate::model::geom::ArcEndpoint;
use crate::model::geom::primitive::Point;
use crate::model::geom::curve::Arc;
use crate::model::geom::aabb::Aabb;

/**
 * 由弧序列与闭合标志构造子轮廓。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：arcs — 弧序列
 * 参数：is_closed — 是否闭合
 */
pub fn contour_new(arcs: Vec<Arc>, is_closed: bool) -> Contour {
    Contour { arcs, is_closed }
}

/**
 * 由弧端点序列构造子轮廓。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：eps — 弧端点序列
 */
pub fn contour_from_endpoints(eps: Vec<ArcEndpoint>) -> Contour {
    let num_endpoints = eps.len();
    let mut arcs = Vec::with_capacity(num_endpoints.saturating_sub(1));
    // 端点序列为「起、终」成对：第 i 条弧由 eps[i] 到 eps[i+1]，
    // 曲率参数取该弧终点所携带的 d（与 glyphy 端点约定一致）。
    for i in 0..num_endpoints.saturating_sub(1) {
        let p0 = Point { x: eps[i].px, y: eps[i].py };
        let p1 = Point { x: eps[i + 1].px, y: eps[i + 1].py };
        let d = eps[i + 1].d;
        arcs.push(Arc { p0, p1, d });
    }
    // 首尾端点坐标相接即视为闭合；不接时返回未闭合轮廓（非错误路径）。
    let is_closed = num_endpoints >= 2
        && num::float_equals(eps[0].px, eps[num_endpoints - 1].px, None)
        && num::float_equals(eps[0].py, eps[num_endpoints - 1].py, None);
    contour_new(arcs, is_closed)
}

/**
 * 导出子轮廓的弧端点序列。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：c — 子轮廓
 */
pub fn contour_endpoints(c: &Contour) -> Vec<ArcEndpoint> {
    let num_arcs = c.arcs.len();
    let mut eps = Vec::with_capacity(num_arcs + 1);
    if num_arcs == 0 {
        return eps;
    }
    // 闭合时首端点的 d 取收尾弧；开放时取首弧，保证与 from_endpoints 往返一致。
    let head_d = if c.is_closed { c.arcs[num_arcs - 1].d } else { c.arcs[0].d };
    eps.push(ArcEndpoint {
        px: c.arcs[0].p0.x,
        py: c.arcs[0].p0.y,
        d: head_d,
        tag: None,
    });
    for arc in c.arcs.iter() {
        eps.push(ArcEndpoint {
            px: arc.p1.x,
            py: arc.p1.y,
            d: arc.d,
            tag: None,
        });
    }
    eps
}

/**
 * 判定子轮廓绕向。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：c — 子轮廓
 */
pub fn contour_is_clockwise(c: &Contour) -> bool {
    contour_signed_area(c) < 0.0
}

/**
 * 反转子轮廓绕向。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：c — 子轮廓，原地改写
 */
pub fn contour_reverse(c: &mut Contour) {
    // 原地改写：不复制弧序列，保证调用方可见（REQ-005.3）。
    c.arcs.reverse();
    for arc in c.arcs.iter_mut() {
        std::mem::swap(&mut arc.p0, &mut arc.p1);
        arc.d = -arc.d;
    }
}

/**
 * 子轮廓包围盒。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：c — 子轮廓
 */
pub fn contour_extents(c: &Contour) -> Aabb {
    let mut bb = Aabb::new_invalid();
    for arc in c.arcs.iter() {
        bb.extend_by(&arc.extents());
    }
    bb
}

/**
 * 构造轮廓。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：contours — 子轮廓集合
 */
pub fn outline_new(contours: Vec<Contour>) -> Outline {
    Outline { contours }
}

/**
 * 轮廓包围盒。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：o — 轮廓
 */
pub fn outline_extents(o: &Outline) -> Aabb {
    let mut bb = Aabb::new_invalid();
    for c in o.contours.iter() {
        bb.extend_by(&contour_extents(c));
    }
    bb
}

/**
 * 反转轮廓全部子轮廓绕向。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：o — 轮廓，原地改写
 */
pub fn outline_reverse(o: &mut Outline) {
    for c in o.contours.iter_mut() {
        contour_reverse(c);
    }
}

/**
 * 判定轮廓整体绕向。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 *
 * 参数：o — 轮廓
 */
pub fn outline_is_clockwise(o: &Outline) -> bool {
    let mut area = 0.0f32;
    for c in o.contours.iter() {
        area += contour_signed_area(c);
    }
    area < 0.0
}

/**
 * 按需求反转子轮廓绕向。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  contour 至少含一条弧
 *   - ensures   需要反转时真正改写传入对象
 *   - 错误      无
 *
 * 参数：contour — 待判定的子轮廓，原地改写
 * 参数：inverse — 是否需要反转
 */
pub fn contour_winding(contour: &mut Contour, inverse: bool) {
    if contour.arcs.is_empty() {
        return;
    }
    // 与 glyphy 一致：r = inverse XOR 当前顺时针 XOR 奇偶修正项。
    // 单子轮廓无其它轮廓上下文，奇偶修正项取「不在任何其它轮廓内」（even）。
    let even_odd_correction = true;
    let need_reverse = num::xor(num::xor(inverse, contour_is_clockwise(contour)), even_odd_correction);
    if need_reverse {
        contour_reverse(contour);
    }
}

/**
 * 按需求反转轮廓绕向。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  outline 至少含一条子轮廓
 *   - ensures   需要反转时真正改写传入对象
 *   - 错误      无
 *
 * 参数：outline — 待判定的轮廓，原地改写
 * 参数：inverse — 是否需要反转
 */
pub fn outline_winding(outline: &mut Outline, inverse: bool) {
    let num_contours = outline.contours.len();
    for i in 0..num_contours {
        if outline.contours[i].arcs.is_empty() {
            continue;
        }
        // 以子轮廓首个端点为采样点，统计它落在多少个「其它」子轮廓内（奇偶）。
        let sample = outline.contours[i].arcs[0].p0;
        let mut inside_odd = false;
        for j in 0..num_contours {
            if j == i || outline.contours[j].arcs.is_empty() {
                continue;
            }
            if even_odd(&outline.contours[j], sample) {
                inside_odd = !inside_odd;
            }
        }
        let even_odd_correction = !inside_odd;
        let need_reverse = num::xor(
            num::xor(inverse, contour_is_clockwise(&outline.contours[i])),
            even_odd_correction,
        );
        if need_reverse {
            contour_reverse(&mut outline.contours[i]);
        }
    }
}

/**
 * 奇偶规则内外判定。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  contour 至少含一条弧
 *   - ensures   对闭合子轮廓给出内外判定
 *   - 错误      无
 *
 * 参数：contour — 子轮廓
 * 参数：p — 目标点
 */
pub fn even_odd(contour: &Contour, p: Point) -> bool {
    let mut count = 0.0f32;
    for arc in contour.arcs.iter() {
        let s0 = categorize(arc.p0.y, p.y);
        let s1 = categorize(arc.p1.y, p.y);
        if arc.d == 0.0 {
            // 线段分支
            if s0 == 0 || s1 == 0 {
                let t = arc.tangents();
                if s0 == 0 && arc.p0.x < p.x + num::EPSILON {
                    count += 0.5 * categorize(t.0.y, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + num::EPSILON {
                    count += 0.5 * categorize(t.1.y, 0.0) as f32;
                }
                continue;
            }
            if s0 == s1 {
                continue;
            }
            let x = arc.p0.x + (arc.p1.x - arc.p0.x) * ((p.y - arc.p0.y) / (arc.p1.y - arc.p0.y));
            if x >= p.x - num::EPSILON {
                continue;
            }
            count += 1.0;
        } else {
            // 圆弧分支
            if s0 == 0 || s1 == 0 {
                let (mut t0, mut t1) = arc.tangents();
                if num::is_zero(t0.y, None) {
                    t0.y = s1 as f32;
                }
                if num::is_zero(t1.y, None) {
                    t1.y = -(s0 as f32);
                }
                if s0 == 0 && arc.p0.x < p.x + num::EPSILON {
                    count += 0.5 * categorize(t0.y, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + num::EPSILON {
                    count += 0.5 * categorize(t1.y, 0.0) as f32;
                }
            }
            let c = arc.center();
            let r = arc.radius();
            if c.x - r >= p.x {
                continue;
            }
            let y = p.y - c.y;
            let x2 = r * r - y * y;
            if x2 <= num::EPSILON {
                continue;
            }
            let dx = x2.sqrt();
            let candidates = [
                Point { x: c.x - dx, y: p.y },
                Point { x: c.x + dx, y: p.y },
            ];
            for q in candidates.iter() {
                if !point_equals(*q, arc.p0)
                    && !point_equals(*q, arc.p1)
                    && q.x < p.x - num::EPSILON
                    && arc.wedge_contains_point(*q)
                {
                    count += 1.0;
                }
            }
        }
    }
    (count.floor() as i32) & 1 == 1
}

/// 单条弧贡献的有符号面积分量（与 pi_sdf glyphy 一致）。
fn arc_signed_area(arc: &Arc) -> f32 {
    let cross = arc.p0.x * arc.p1.y - arc.p0.y * arc.p1.x;
    let dx = arc.p1.x - arc.p0.x;
    let dy = arc.p1.y - arc.p0.y;
    cross - 0.5 * arc.d * (dx * dx + dy * dy)
}

/// 子轮廓的有符号面积；负值表示顺时针。
fn contour_signed_area(c: &Contour) -> f32 {
    let mut area = 0.0f32;
    for arc in c.arcs.iter() {
        area += arc_signed_area(arc);
    }
    area
}

fn categorize(v: f32, r: f32) -> i32 {
    if v < r - num::EPSILON {
        -1
    } else if v > r + num::EPSILON {
        1
    } else {
        0
    }
}

fn point_equals(a: Point, b: Point) -> bool {
    num::float_equals(a.x, b.x, None) && num::float_equals(a.y, b.y, None)
}
