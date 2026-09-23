/**
 * 弧列表距离场（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/sdf/sdf.rs
 */

use super::sdf::SdfSample;
use crate::model::base::consts::FARWAY;
use crate::model::base::error::{Error, Result};
use crate::model::geom::curve::Arc;
use crate::model::geom::primitive::{Point, Vector};

/**
 * 由完整弧集合计算某点的距离场采样。
 *
 * 契约：API-014
 *
 * 约束：
 *   - requires  indices 中每个值均小于 all 的长度
 *   - ensures   distance 符号正确；arc_index 指向产生最小距离的弧；distance 非 NaN
 *   - 错误      InvalidParam —— indices 越界
 *
 * 参数：arcs — 完整弧集合
 * 参数：p — 目标点
 */
pub fn sdf_from_arcs(arcs: &[Arc], p: Point) -> SdfSample {
    sample(arcs, p)
}

/**
 * 由索引子集计算某点的距离场采样。
 *
 * 契约：API-014
 *
 * 约束：
 *   - requires  indices 中每个值均小于 all 的长度
 *   - ensures   distance 符号正确；arc_index 指向产生最小距离的弧；distance 非 NaN
 *   - 错误      InvalidParam —— indices 越界
 *
 * 参数：all — 完整弧集合
 * 参数：indices — 参与计算的弧下标
 * 参数：p — 目标点
 */
pub fn sdf_from_indices(all: &[Arc], indices: &[usize], p: Point) -> Result<SdfSample> {
    for &index in indices {
        if index >= all.len() {
            return Err(Error::InvalidParam("sdf arc index out of range"));
        }
    }
    let selected: Vec<Arc> = indices.iter().map(|&index| all[index]).collect();
    let sampled = sample(&selected, p);
    // 对外暴露的下标指回 all，保证 sdf_from_indices(all, 恒等下标) 与 sdf_from_arcs(all) 一致。
    let arc_index = sampled.arc_index.map(|local| indices[local]);
    debug_assert!(arc_index.map_or(true, |index| index < all.len()));
    Ok(SdfSample {
        distance: sampled.distance,
        arc_index,
    })
}

/// 在弧序列上取最小几何距离并定符号；弧序列为空（或全无有限距离）时返回远点标记。
fn sample(arcs: &[Arc], p: Point) -> SdfSample {
    let mut min_dist = f32::INFINITY;
    let mut closest: Option<usize> = None;
    for (index, arc) in arcs.iter().enumerate() {
        let distance = arc.distance_to_point(p);
        if distance < min_dist {
            min_dist = distance;
            closest = Some(index);
        }
    }
    match closest {
        None => SdfSample {
            distance: FARWAY,
            arc_index: None,
        },
        Some(index) => {
            let distance = side_of(&arcs[index], p) * min_dist;
            debug_assert!(min_dist.is_finite(), "最近距离应为有限值");
            debug_assert!(!distance.is_nan(), "距离不得为 NaN");
            SdfSample {
                distance,
                arc_index: Some(index),
            }
        }
    }
}

/// 点相对弧的方向符号。
///
/// 约定：以弧的行进方向为基准，点位于左侧返回 -1、右侧返回 +1。
/// 对逆时针闭合轮廓（内部在左）即「内部为负、外部为正」，消除原实现的符号缺陷（REQ-005.4）。
fn side_of(arc: &Arc, p: Point) -> f32 {
    let (q, dir) = closest_frame(arc, p);
    let wx = p.x - q.x;
    let wy = p.y - q.y;
    let cross = dir.x * wy - dir.y * wx;
    if cross > 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// 最近点及该点处的行进方向。
fn closest_frame(arc: &Arc, p: Point) -> (Point, Vector) {
    if arc.d == 0.0 {
        let q = closest_on_segment(arc.p0, arc.p1, p);
        return (q, Vector::new(arc.p1.x - arc.p0.x, arc.p1.y - arc.p0.y));
    }
    if arc.wedge_contains_point(p) {
        let center = arc.center();
        let rx = p.x - center.x;
        let ry = p.y - center.y;
        let rn = (rx * rx + ry * ry).sqrt();
        let (ux, uy) = if rn == 0.0 {
            (1.0, 0.0)
        } else {
            (rx / rn, ry / rn)
        };
        let radius = arc.radius();
        let q = Point::new(center.x + ux * radius, center.y + uy * radius);
        let dir = if arc.d > 0.0 {
            Vector::new(-uy, ux)
        } else {
            Vector::new(uy, -ux)
        };
        return (q, dir);
    }
    let (t0, t1) = arc.tangents();
    if p.distance_to(arc.p0) <= p.distance_to(arc.p1) {
        (arc.p0, t0)
    } else {
        (arc.p1, t1)
    }
}

/// 点到线段的最近点（将投影参数夹到 [0,1]）。
fn closest_on_segment(a: Point, b: Point, p: Point) -> Point {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return a;
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0);
    Point::new(a.x + t * dx, a.y + t * dy)
}

#[cfg(test)]
mod tests {
    use crate::core::sdf::sdf::{sdf_from_arcs, sdf_from_indices};
    use crate::model::base::error::Error;
    use crate::model::geom::curve::Arc;
    use crate::model::geom::primitive::Point;

    /// tan(PI/8)：90 度小弧的曲率参数。
    const D90: f32 = 0.414_213_56;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() <= 1e-6
    }

    /// 逆时针单位圆（内部在行进方向左侧，内部为负）。
    fn unit_circle_ccw() -> Vec<Arc> {
        vec![
            Arc::new(Point::new(1.0, 0.0), Point::new(0.0, 1.0), D90),
            Arc::new(Point::new(0.0, 1.0), Point::new(-1.0, 0.0), D90),
            Arc::new(Point::new(-1.0, 0.0), Point::new(0.0, -1.0), D90),
            Arc::new(Point::new(0.0, -1.0), Point::new(1.0, 0.0), D90),
        ]
    }

    /// 逆时针单位正方形（内部为负）：四条 d=0 的线段弧。
    fn unit_square_ccw() -> Vec<Arc> {
        vec![
            Arc::new(Point::new(1.0, -1.0), Point::new(1.0, 1.0), 0.0),
            Arc::new(Point::new(1.0, 1.0), Point::new(-1.0, 1.0), 0.0),
            Arc::new(Point::new(-1.0, 1.0), Point::new(-1.0, -1.0), 0.0),
            Arc::new(Point::new(-1.0, -1.0), Point::new(1.0, -1.0), 0.0),
        ]
    }

    /// 仅用冻结 Arc 接口独立求出的几何最短距离。
    fn geometric_distance(arcs: &[Arc], p: Point) -> f32 {
        arcs.iter()
            .map(|arc| arc.distance_to_point(p))
            .fold(f32::INFINITY, f32::min)
    }

    #[test]
    fn case_027_inside_point_negative() {
        let arcs = unit_circle_ccw();
        let center = Point::new(0.0, 0.0);
        let inside = [
            Point::new(0.0, 0.0),
            Point::new(0.5, 0.0),
            Point::new(0.0, 0.6),
            Point::new(-0.3, -0.3),
        ];
        for p in inside {
            let sample = sdf_from_arcs(&arcs, p);
            assert!(sample.arc_index.is_some(), "非空集合应给出最近弧");
            assert!(sample.distance < 0.0, "内点符号应表示在内部");
            let expected = p.distance_to(center) - 1.0;
            assert!(close(sample.distance, expected), "内点幅值应等于几何距离");
            assert!(close(sample.distance.abs(), geometric_distance(&arcs, p)));
        }
    }

    #[test]
    fn case_028_outside_sign_opposite_and_magnitude() {
        let arcs = unit_circle_ccw();
        let center = Point::new(0.0, 0.0);
        let inside = sdf_from_arcs(&arcs, Point::new(0.1, 0.1));
        assert!(inside.distance < 0.0);

        let outside = [
            Point::new(2.0, 0.0),
            Point::new(0.0, 2.0),
            Point::new(0.0, -3.0),
            Point::new(1.5, 1.5),
        ];
        for p in outside {
            let sample = sdf_from_arcs(&arcs, p);
            assert!(sample.distance > 0.0, "外点符号应与内点相反");
            assert!(
                close(sample.distance.abs(), geometric_distance(&arcs, p)),
                "外点幅值应等于几何最短距离"
            );
            assert!(close(sample.distance.abs(), p.distance_to(center) - 1.0));
        }
    }

    #[test]
    fn case_078_index_out_of_range_returns_err() {
        let arcs = unit_circle_ccw();
        let p = Point::new(0.1, 0.1);
        assert!(matches!(
            sdf_from_indices(&arcs, &[0, 4], p),
            Err(Error::InvalidParam(_))
        ));
        assert!(matches!(
            sdf_from_indices(&arcs, &[usize::MAX], p),
            Err(Error::InvalidParam(_))
        ));

        let ok = sdf_from_indices(&arcs, &[0, 1, 2, 3], p);
        assert!(ok.is_ok(), "合法下标应成功");
        if let Ok(sample) = ok {
            assert!(sample.arc_index.is_some());
            assert!(close(sample.distance, sdf_from_arcs(&arcs, p).distance));
        }
    }

    #[test]
    fn arc_index_points_to_nearest_arc() {
        let arcs = unit_square_ccw();
        let outside = sdf_from_arcs(&arcs, Point::new(2.0, 0.5));
        assert_eq!(outside.arc_index, Some(0), "最近弧应为右边界");
        assert!(close(outside.distance, 1.0));

        let inside = sdf_from_arcs(&arcs, Point::new(0.0, 0.5));
        assert_eq!(inside.arc_index, Some(1), "最近弧应为上边界");
        assert!(close(inside.distance, -0.5));
    }

    #[test]
    fn empty_arc_set_has_no_index_and_no_nan() {
        let sample = sdf_from_arcs(&[], Point::new(3.0, 4.0));
        assert!(sample.arc_index.is_none());
        assert!(!sample.distance.is_nan());
    }
}
