/**
 * 贝塞尔拟合弧与描边几何（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/outline/fit.rs
 */

use super::fit::StrokeMesh;
use crate::model::base::error::{Error, Result};
use crate::model::geom::curve::{Arc, Bezier};
use crate::model::geom::primitive::{Point, Vector};

const MAX_FIT_DEPTH: u32 = 16;
const DEGENERATE_CHORD: f32 = 1e-12;
const DEGENERATE_EXTENT: f32 = 1e-9;
const SAMPLE_COUNT: u32 = 64;
const FIT_SAFETY: f32 = 0.9;
const ZERO_D: f32 = 1e-6;
const STRAIGHT_D: f32 = 1e-4;

/**
 * 把三次贝塞尔拟合为弧序列。
 *
 * 契约：API-012
 *
 * 约束：
 *   - requires  max_deviation 为正有限值
 *   - ensures   生成弧的首尾端点与贝塞尔一致；每弧中点相对贝塞尔的偏差不超过 max_deviation
 *   - 错误      InvalidParam —— max_deviation 非正
 *
 * 参数：b — 待拟合的贝塞尔曲线
 * 参数：max_deviation — 允许的最大中点偏差，正有限值
 */
pub fn bezier_to_arcs(b: &Bezier, max_deviation: f32) -> Result<Vec<Arc>> {
    if !(max_deviation > 0.0) || !max_deviation.is_finite() {
        return Err(Error::InvalidParam(
            "max_deviation must be positive and finite",
        ));
    }
    let mut arcs: Vec<Arc> = Vec::new();
    fit_bezier(b, max_deviation, 0, &mut arcs);
    debug_assert!(!arcs.is_empty());
    if let (Some(first), Some(last)) = (arcs.first(), arcs.last()) {
        debug_assert!(first.p0 == b.p0);
        debug_assert!(last.p1 == b.p3);
    }
    Ok(arcs)
}

/**
 * 由轮廓弧序列生成描边网格。
 *
 * 契约：API-013
 *
 * 约束：
 *   - requires  thickness 为正有限值；arcs 非空
 *   - ensures   每条弧生成 4 个顶点、2 个三角形（索引数 = 弧数 × 6）；法线按节点邻接计算
 *   - 错误      InvalidParam —— thickness 非正
 *
 * 参数：arcs — 轮廓弧序列
 * 参数：thickness — 描边厚度，正有限值
 */
pub fn stroke_mesh(arcs: &[Arc], thickness: f32) -> Result<StrokeMesh> {
    if !(thickness > 0.0) || !thickness.is_finite() {
        return Err(Error::InvalidParam(
            "thickness must be positive and finite",
        ));
    }
    let count = arcs.len();
    let mut mesh = StrokeMesh {
        positions: Vec::with_capacity(count * 8),
        uvs: Vec::with_capacity(count * 8),
        indices: Vec::with_capacity(count * 6),
    };
    for i in 0..count {
        let arc = &arcs[i];
        let prev = if i == 0 { None } else { Some(&arcs[i - 1]) };
        let next = if i + 1 < count {
            Some(&arcs[i + 1])
        } else {
            None
        };
        let start_normal = joint_normal(prev, Some(arc), arc.p0);
        let end_normal = joint_normal(Some(arc), next, arc.p1);

        let base = (mesh.positions.len() / 2) as u16;
        push_vertex(&mut mesh, arc.p0, start_normal, thickness, 0.0, 0.0);
        push_vertex(&mut mesh, arc.p0, negate(start_normal), thickness, 0.0, 1.0);
        push_vertex(&mut mesh, arc.p1, end_normal, thickness, 1.0, 0.0);
        push_vertex(&mut mesh, arc.p1, negate(end_normal), thickness, 1.0, 1.0);

        mesh.indices.push(base);
        mesh.indices.push(base + 1);
        mesh.indices.push(base + 2);
        mesh.indices.push(base + 1);
        mesh.indices.push(base + 2);
        mesh.indices.push(base + 3);
    }
    debug_assert_eq!(mesh.positions.len(), count * 8);
    debug_assert_eq!(mesh.uvs.len(), count * 8);
    debug_assert_eq!(mesh.indices.len(), count * 6);
    Ok(mesh)
}

fn fit_bezier(b: &Bezier, tol: f32, depth: u32, out: &mut Vec<Arc>) {
    let chord = point_distance(b.p0, b.p3);
    if chord <= DEGENERATE_CHORD {
        if depth >= MAX_FIT_DEPTH || control_extent(b) <= DEGENERATE_EXTENT {
            out.push(Arc {
                p0: b.p0,
                p1: b.p3,
                d: 0.0,
            });
            return;
        }
        let (left, right) = b.split(0.5);
        fit_bezier(&left, tol, depth + 1, out);
        fit_bezier(&right, tol, depth + 1, out);
        return;
    }
    let arc = arc_through_three(b.p0, b.p3, b.midpoint());
    if depth >= MAX_FIT_DEPTH || max_deviation(b, &arc) <= tol * FIT_SAFETY {
        out.push(arc);
        return;
    }
    let (left, right) = b.split(0.5);
    fit_bezier(&left, tol, depth + 1, out);
    fit_bezier(&right, tol, depth + 1, out);
}

fn arc_through_three(p0: Point, p1: Point, pm: Point) -> Arc {
    if p0 == pm || p1 == pm {
        return Arc {
            p0,
            p1,
            d: 0.0,
        };
    }
    let ux = p0.x - pm.x;
    let uy = p0.y - pm.y;
    let vx = p1.x - pm.x;
    let vy = p1.y - pm.y;
    if (ux == 0.0 && uy == 0.0) || (vx == 0.0 && vy == 0.0) {
        return Arc {
            p0,
            p1,
            d: 0.0,
        };
    }
    let a0 = uy.atan2(ux);
    let a1 = vy.atan2(vx);
    let d = (((a1 - a0) * 0.5) - std::f32::consts::FRAC_PI_2).tan();
    let d = if d.is_finite() && d.abs() >= ZERO_D {
        d
    } else {
        0.0
    };
    Arc { p0, p1, d }
}

fn max_deviation(b: &Bezier, a: &Arc) -> f32 {
    let mut max_dev = 0.0_f32;
    let mut i = 0;
    while i <= SAMPLE_COUNT {
        let t = i as f32 / SAMPLE_COUNT as f32;
        let dev = a.distance_to_point(b.point(t));
        if !dev.is_finite() {
            return f32::INFINITY;
        }
        if dev > max_dev {
            max_dev = dev;
        }
        i += 1;
    }
    max_dev
}

fn control_extent(b: &Bezier) -> f32 {
    let mut extent = point_distance(b.p0, b.p1);
    extent = extent.max(point_distance(b.p1, b.p2));
    extent = extent.max(point_distance(b.p2, b.p3));
    extent = extent.max(point_distance(b.p0, b.p3));
    extent
}

fn point_distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn joint_normal(prev: Option<&Arc>, next: Option<&Arc>, p: Point) -> Vector {
    let mut nx = 0.0_f32;
    let mut ny = 0.0_f32;
    if let Some(a) = prev {
        let (x, y) = endpoint_normal(a, p);
        nx += x;
        ny += y;
    }
    if let Some(a) = next {
        let (x, y) = endpoint_normal(a, p);
        nx += x;
        ny += y;
    }
    let len = (nx * nx + ny * ny).sqrt();
    if len > 0.0 {
        Vector {
            x: nx / len,
            y: ny / len,
        }
    } else {
        Vector { x: 0.0, y: 0.0 }
    }
}

fn endpoint_normal(a: &Arc, p: Point) -> (f32, f32) {
    if a.d.abs() < STRAIGHT_D {
        let dx = a.p1.x - a.p0.x;
        let dy = a.p1.y - a.p0.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            return (0.0, 0.0);
        }
        (-dy / len, dx / len)
    } else {
        let c = a.center();
        let dx = p.x - c.x;
        let dy = p.y - c.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            return (0.0, 0.0);
        }
        (dx / len, dy / len)
    }
}

fn negate(v: Vector) -> Vector {
    Vector { x: -v.x, y: -v.y }
}

fn push_vertex(mesh: &mut StrokeMesh, p: Point, n: Vector, half: f32, u: f32, v: f32) {
    mesh.positions.push(p.x + n.x * half);
    mesh.positions.push(p.y + n.y * half);
    mesh.uvs.push(u);
    mesh.uvs.push(v);
}

#[cfg(test)]
mod outline_fit_tests {
    use crate::core::outline::fit::{bezier_to_arcs, stroke_mesh};
    use crate::model::base::error::Error;
    use crate::model::geom::curve::{Arc, Bezier};
    use crate::model::geom::primitive::Point;

    fn point(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn dist(a: Point, b: Point) -> f32 {
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
    }

    fn bezier(p0: Point, p1: Point, p2: Point, p3: Point) -> Bezier {
        Bezier::new(p0, p1, p2, p3)
    }

    fn max_curve_deviation(b: &Bezier, arcs: &[Arc]) -> f32 {
        let samples = 1024;
        let mut worst = 0.0_f32;
        for i in 0..=samples {
            let t = i as f32 / samples as f32;
            let p = b.point(t);
            let mut nearest = f32::INFINITY;
            for a in arcs {
                let d = a.distance_to_point(p);
                if d < nearest {
                    nearest = d;
                }
            }
            if nearest > worst {
                worst = nearest;
            }
        }
        worst
    }

    // CASE-024 贝塞尔拟合弧：首尾端点一致，最大偏差 ≤ 容差
    #[test]
    fn case_024_bezier_to_arcs_endpoints_and_deviation() {
        let b = bezier(point(0.0, 0.0), point(0.0, 1.0), point(1.0, 1.0), point(1.0, 0.0));
        let tol = 1e-3_f32;
        let arcs = bezier_to_arcs(&b, tol).expect("fit must succeed");
        assert!(!arcs.is_empty());
        assert!(dist(arcs.first().unwrap().p0, b.p0) <= 1e-6);
        assert!(dist(arcs.last().unwrap().p1, b.p3) <= 1e-6);
        for w in arcs.windows(2) {
            assert!(dist(w[0].p1, w[1].p0) <= 1e-5);
        }
        assert!(max_curve_deviation(&b, &arcs) <= tol);
    }

    // CASE-025 退化为直线的贝塞尔：返回单条线段语义弧（d = 0）
    #[test]
    fn case_025_straight_bezier_yields_single_line_arc() {
        let b = bezier(point(0.0, 0.0), point(1.0, 0.0), point(2.0, 0.0), point(3.0, 0.0));
        let arcs = bezier_to_arcs(&b, 1e-3).expect("fit must succeed");
        assert_eq!(arcs.len(), 1);
        assert_eq!(arcs[0].d, 0.0);
        assert!(dist(arcs[0].p0, b.p0) <= 1e-6);
        assert!(dist(arcs[0].p1, b.p3) <= 1e-6);
    }

    // CASE-026 一条含 3 弧的轮廓：位置 24 个分量（12 顶点）、索引 18 个
    #[test]
    fn case_026_stroke_mesh_counts_for_three_arcs() {
        let arcs = vec![
            Arc::new(point(0.0, 0.0), point(1.0, 0.0), 0.0),
            Arc::new(point(1.0, 0.0), point(1.0, 1.0), 0.0),
            Arc::new(point(1.0, 1.0), point(0.0, 0.0), 0.0),
        ];
        let thickness = 0.1_f32;
        let mesh = stroke_mesh(&arcs, thickness).expect("stroke must succeed");
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.uvs.len(), 24);
        assert_eq!(mesh.indices.len(), 18);
        assert_eq!(mesh.indices.len(), arcs.len() * 6);
        let vertex_count = mesh.positions.len() / 2;
        for &idx in &mesh.indices {
            assert!((idx as usize) < vertex_count);
        }
        let v0 = point(mesh.positions[0], mesh.positions[1]);
        let v1 = point(mesh.positions[2], mesh.positions[3]);
        let mid = point((v0.x + v1.x) * 0.5, (v0.y + v1.y) * 0.5);
        assert!((dist(v0, v1) - 2.0 * thickness).abs() <= 1e-5);
        assert!(dist(mid, arcs[0].p0) <= 1e-5);
    }

    // CASE-076 max_deviation 为 0 或负：返回 Err(InvalidParam)，不 panic
    #[test]
    fn case_076_bezier_to_arcs_rejects_non_positive_tolerance() {
        let b = bezier(point(0.0, 0.0), point(0.0, 1.0), point(1.0, 1.0), point(1.0, 0.0));
        assert!(matches!(bezier_to_arcs(&b, 0.0), Err(Error::InvalidParam(_))));
        assert!(matches!(bezier_to_arcs(&b, -1e-3), Err(Error::InvalidParam(_))));
        assert!(matches!(bezier_to_arcs(&b, f32::NAN), Err(Error::InvalidParam(_))));
    }

    // CASE-077 thickness 为 0 或负：返回 Err(InvalidParam)，不 panic
    #[test]
    fn case_077_stroke_mesh_rejects_non_positive_thickness() {
        let arcs = vec![Arc::new(point(0.0, 0.0), point(1.0, 0.0), 0.0)];
        assert!(matches!(stroke_mesh(&arcs, 0.0), Err(Error::InvalidParam(_))));
        assert!(matches!(stroke_mesh(&arcs, -0.5), Err(Error::InvalidParam(_))));
        assert!(matches!(stroke_mesh(&arcs, f32::NAN), Err(Error::InvalidParam(_))));
    }
}
