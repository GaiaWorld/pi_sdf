//! TASK-03 几何基础：仅经冻结接口验证（CASE-006..019、073..075）。
//!
//! 断言基于数学性质（距离、模长、包含关系、端点一致），容差 1e-5。

use pi_sdf2::model::{Aabb, Arc, ArcEndpoint, Bezier, Direction, Line, Point, Segment, Vector};

const PI: f32 = std::f32::consts::PI;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-5
}

fn point_close(a: Point, b: Point) -> bool {
    close(a.x, b.x) && close(a.y, b.y)
}

fn sample_bezier() -> Bezier {
    Bezier::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 3.0),
        Point::new(3.0, 3.0),
        Point::new(3.0, 0.0),
    )
}

#[test]
fn case_006_point_distance() {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(3.0, 4.0);
    assert!(close(a.distance_to(b), 5.0), "距离应为 5.0");
    assert!(close(a.squared_distance_to(b), 25.0), "平方距离应为 25.0");
    assert!(a.distance_to(b) >= 0.0, "距离恒非负");
    assert!(point_close(a.midpoint(b), Point::new(1.5, 2.0)), "中点应位于两点连线中点");
    assert!(point_close(a.midpoint(b), b.midpoint(a)), "中点应对称");
    assert!(point_close(a.add_vector(a.to_vector()), a), "加自身向量应回到原点");
}

#[test]
fn case_008_vector_normalized() {
    let v = Vector::new(3.0, 4.0);
    let n = v.normalized();
    assert!(close(n.norm(), 1.0), "归一化后模长应为 1");
    assert!(close(n.x, 0.6) && close(n.y, 0.8), "方向应保持不变");
    assert!(close(n.cross(v), 0.0), "归一化应保持方向（叉积为 0）");
}

#[test]
fn case_009_zero_vector_normalized() {
    let z = Vector::new(0.0, 0.0).normalized();
    assert_eq!(z, Vector::new(0.0, 0.0));
    assert!(close(z.norm(), 0.0));
}

#[test]
fn case_010_line_intersect() {
    let l1 = Line::from_points(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
    let l2 = Line::from_points(Point::new(0.0, 0.0), Point::new(0.0, 1.0));
    let p = l1.intersect(&l2).expect("两条非平行直线应相交");
    assert!(point_close(p, Point::new(0.0, 0.0)));

    let l3 = Line::from_points(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
    let l4 = Line::from_points(Point::new(0.0, 2.0), Point::new(1.0, 1.0));
    let p2 = l3.intersect(&l4).expect("两条非平行直线应相交");
    assert!(point_close(p2, Point::new(1.0, 1.0)));
}

#[test]
fn case_011_parallel_line_intersect_none() {
    let l1 = Line::from_points(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
    let l2 = Line::from_points(Point::new(0.0, 1.0), Point::new(1.0, 1.0));
    assert!(l1.intersect(&l2).is_none(), "平行直线应返回 None");
}

#[test]
fn case_012_segment_squared_distance() {
    let s = Segment::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0));
    let inner = Point::new(2.0, 3.0);
    assert!(close(s.squared_distance_to_point(inner), 9.0));
    assert!(close(s.distance_to_point(inner), 3.0));
    assert!(s.distance_to_point(inner) <= Point::new(0.0, 0.0).distance_to(inner) + 1e-6);

    let beyond = Point::new(-3.0, 0.0);
    assert!(close(s.squared_distance_to_point(beyond), 9.0));
    assert!(close(s.distance_to_point(beyond), 3.0));
}

#[test]
fn case_013_segment_nearest_points() {
    let a = Segment::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0));
    let b = Segment::new(Point::new(1.0, 2.0), Point::new(1.0, 5.0));
    let (pa, pb) = Segment::nearest_points_on_line_segments(&a, &b);
    assert!(a.contains_in_span(pa), "最近点应落在第一条段内");
    assert!(b.contains_in_span(pb), "最近点应落在第二条段内");
    assert!(close(pa.distance_to(pb), 2.0));

    let c = Segment::new(Point::new(0.0, 0.0), Point::new(4.0, 4.0));
    let d = Segment::new(Point::new(0.0, 4.0), Point::new(4.0, 0.0));
    let (pc, pd) = Segment::nearest_points_on_line_segments(&c, &d);
    assert!(c.contains_in_span(pc) && d.contains_in_span(pd));
    assert!(close(pc.distance_to(pd), 0.0), "相交线段的最近距离应为 0");
}

#[test]
fn case_014_bezier_split() {
    let b = sample_bezier();
    let (left, right) = b.split(0.5);
    let mid = b.midpoint();
    assert!(point_close(left.p0, b.p0));
    assert!(point_close(left.p3, mid));
    assert!(point_close(right.p0, mid));
    assert!(point_close(right.p3, b.p3));
    assert!(point_close(left.p3, right.p0), "两段应在分割点拼接");
    assert!(point_close(b.point(0.0), b.p0));
    assert!(point_close(b.point(1.0), b.p3));
}

#[test]
fn case_015_bezier_segment() {
    let b = sample_bezier();
    let seg = b.segment(0.25, 0.75);
    assert!(point_close(seg.p0, b.point(0.25)));
    assert!(point_close(seg.p3, b.point(0.75)));
    let half = b.segment(0.25, 0.75).midpoint();
    assert!(point_close(half, b.point(0.5)));
}

#[test]
fn case_016_large_arc_wedge() {
    let arc = Arc::new(Point::new(0.0, 0.0), Point::new(2.0, 0.0), 2.0);
    assert!(arc.angle() > PI, "d=2 应对应大弧（圆心角大于 PI）");
    let c = arc.center();
    let r = arc.radius();
    let outside = Point::new(c.x, c.y + r + 1.0);
    let inside = Point::new(c.x, c.y - r - 1.0);
    assert!(!arc.wedge_contains_point(outside), "大弧外侧点应判为不包含");
    assert!(arc.wedge_contains_point(inside), "大弧扇区内点应判为包含");
    assert!(close(arc.distance_to_point(inside), 1.0));

    let small = Arc::new(Point::new(0.0, 0.0), Point::new(2.0, 0.0), 0.2679492);
    assert!(small.angle() < PI);
    let sc = small.center();
    assert!(small.wedge_contains_point(Point::new(sc.x, sc.y - 3.0)));
    assert!(!small.wedge_contains_point(Point::new(sc.x, sc.y + 3.0)));
}

#[test]
fn case_017_arc_endpoint_roundtrip() {
    let arc = Arc::new(Point::new(0.0, 0.0), Point::new(2.0, 0.0), 0.5);
    let eps = arc.endpoints();
    assert_eq!(eps.len(), 2);
    assert_eq!(eps[0].px, arc.p0.x);
    assert_eq!(eps[0].py, arc.p0.y);
    assert_eq!(eps[1].px, arc.p1.x);
    assert_eq!(eps[1].py, arc.p1.y);
    assert_eq!(eps[0].d, arc.d);
    assert_eq!(eps[1].d, arc.d);
    assert_eq!(eps[0].tag, None, "tag 不得携带几何信息");
    assert_eq!(eps[1].tag, None, "tag 不得携带几何信息");

    let restored = Arc::from_endpoints(eps).expect("两个端点应可还原弧");
    assert_eq!(restored, arc, "弧与其端点往返应逐位一致");

    let one = vec![ArcEndpoint {
        px: 1.0,
        py: 2.0,
        d: 0.3,
        tag: None,
    }];
    assert!(Arc::from_endpoints(one).is_err(), "端点数量不为 2 应返回 Err");
    assert!(Arc::from_endpoints(Vec::new()).is_err(), "空端点序列应返回 Err");
}

#[test]
fn case_018_aabb_is_empty_both_components() {
    assert!(Aabb::new_invalid().is_empty(), "空盒应判为空");
    assert!(Aabb::new(Point::new(5.0, 0.0), Point::new(0.0, 1.0)).is_empty(), "x 分量反序应为空");
    assert!(Aabb::new(Point::new(0.0, 5.0), Point::new(1.0, 0.0)).is_empty(), "y 分量反序应为空");
    assert!(!Aabb::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)).is_empty());
}

#[test]
fn case_019_aabb_includes() {
    let b = Aabb::new(Point::new(0.0, 0.0), Point::new(4.0, 4.0));
    let point_box = Aabb::new(Point::new(1.0, 1.0), Point::new(1.0, 1.0));
    assert!(b.includes(&point_box), "盒应包含其内部一点");
    let outside = Aabb::new(Point::new(5.0, 5.0), Point::new(6.0, 6.0));
    assert!(!b.includes(&outside));
}

#[test]
fn case_073_degenerate_line_from_points() {
    let p = Point::new(1.0, 2.0);
    let l = Line::from_points(p, p);
    assert_eq!(l.n, Vector::new(0.0, 0.0), "重合两点应退化为零法向量");
    assert_eq!(l.c, 0.0);
    let n = l.normalized();
    assert_eq!(n.n, Vector::new(0.0, 0.0), "退化直线归一化不应 panic");
}

#[test]
fn case_074_degenerate_bezier_segment() {
    let b = sample_bezier();
    let d1 = b.segment(1.0, 0.5);
    let d2 = b.segment(0.5, 0.0);
    for d in [d1, d2] {
        assert_eq!(d.p0, d.p1);
        assert_eq!(d.p0, d.p2);
        assert_eq!(d.p0, d.p3);
        assert!(point_close(d.point(0.5), d.p0));
    }
}

#[test]
fn case_075_degenerate_arc() {
    let p = Point::new(1.0, 1.0);
    let arc = Arc::new(p, p, 0.5);
    assert_eq!(arc.p0, arc.p1);
    assert!(arc.radius().is_finite() || arc.radius().is_infinite());
    let _ = arc.center();
    let _ = arc.angle();
    let _ = arc.len();
    let _ = arc.tangents();
    let _ = arc.extents();
    let _ = arc.approximate_bezier();
    let _ = arc.wedge_contains_point(Point::new(0.0, 0.0));
    let _ = arc.distance_to_point(Point::new(0.0, 0.0));
    let _ = arc.endpoints();
}

#[test]
fn vector_ops_are_consistent() {
    let v = Vector::new(3.0, 4.0);
    assert!(close(v.dot(Vector::new(1.0, 0.0)), 3.0));
    assert!(close(v.cross(Vector::new(1.0, 0.0)), -4.0));
    assert!(close(v.norm_squared(), 25.0));
    assert!(close(v.norm(), 5.0));
    let o = v.ortho();
    assert!(close(o.x, -4.0) && close(o.y, 3.0));
    assert!(close(v.dot(o), 0.0), "正交向量点积为 0");
    assert!(close(v.angle(), 4.0f32.atan2(3.0)));
    let r = v.rebase(1.0, 0.0);
    assert!(close(r.x, 3.0) && close(r.y, 4.0));
}

#[test]
fn line_sub_and_signed_vector_sign() {
    let l = Line::from_points(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
    let norm = l.normalized();
    assert!(close(norm.n.norm(), 1.0));

    let pos = l.sub(&Point::new(0.0, 5.0));
    assert!(close(pos.vec2.norm(), 5.0));
    assert!(!pos.negative);
    let neg = l.sub(&Point::new(0.0, -2.0));
    assert!(close(neg.vec2.norm(), 2.0));
    assert!(neg.negative);

    let same = Point::new(0.0, 5.0).shortest_distance_to_line(&l);
    assert!(close(same.vec2.norm(), pos.vec2.norm()));
    assert_eq!(same.negative, pos.negative);

    let sv = pi_sdf2::model::SignedVector::new(3.0, 4.0, false).neg();
    assert_eq!(sv.vec2, Vector::new(3.0, 4.0));
    assert!(sv.negative);
}

#[test]
fn arc_geometry_is_consistent() {
    let arc = Arc::new(Point::new(0.0, 0.0), Point::new(2.0, 0.0), 1.0);
    assert!(close(arc.angle(), PI));
    assert!(close(arc.radius(), 1.0));
    let c = arc.center();
    assert!(point_close(c, Point::new(1.0, 0.0)));
    assert!(close(arc.len(), PI));

    let below = Point::new(1.0, -2.0);
    assert!(close(arc.distance_to_point(below), 1.0));
    assert!(close(arc.squared_distance_to_point(below), 1.0));

    let ext = arc.extents();
    assert!(point_close(ext.mins, Point::new(0.0, -1.0)));
    assert!(point_close(ext.maxs, Point::new(2.0, 0.0)));

    let (t0, t1) = arc.tangents();
    let r0 = Vector::new(arc.p0.x - c.x, arc.p0.y - c.y);
    let r1 = Vector::new(arc.p1.x - c.x, arc.p1.y - c.y);
    assert!(close(t0.dot(r0), 0.0), "起点切线应垂直于半径");
    assert!(close(t1.dot(r1), 0.0), "终点切线应垂直于半径");

    let bz = arc.approximate_bezier();
    assert!(point_close(bz.point(0.0), arc.p0));
    assert!(point_close(bz.point(1.0), arc.p1));
    assert!(point_close(bz.midpoint(), Point::new(1.0, -1.0)));
}

#[test]
fn aabb_ops_are_consistent() {
    let mut b = Aabb::new(Point::new(0.0, 0.0), Point::new(2.0, 1.0));
    assert!(close(b.width(), 2.0) && close(b.height(), 1.0));
    b.extend(Point::new(-1.0, 3.0));
    assert!(point_close(b.mins, Point::new(-1.0, 0.0)));
    assert!(point_close(b.maxs, Point::new(2.0, 3.0)));

    let mut empty = Aabb::new_invalid();
    empty.extend(Point::new(5.0, 5.0));
    assert!(point_close(empty.mins, Point::new(5.0, 5.0)));
    empty.extend_by(&Aabb::new(Point::new(0.0, 0.0), Point::new(6.0, 6.0)));
    assert!(point_close(empty.mins, Point::new(0.0, 0.0)));
    assert!(point_close(empty.maxs, Point::new(6.0, 6.0)));

    let hit = Aabb::new(Point::new(4.0, 4.0), Point::new(7.0, 7.0)).collision(&empty);
    assert!(hit.is_some());
    let miss = Aabb::new(Point::new(7.0, 7.0), Point::new(8.0, 8.0)).collision(&empty);
    assert!(miss.is_none());

    let wide = Aabb::new(Point::new(0.0, 0.0), Point::new(4.0, 1.0));
    let (h1, h2) = wide.half();
    assert!(close(h1.width(), 2.0) && close(h2.width(), 2.0));
    assert!(close(h1.height(), 1.0) && close(h2.height(), 1.0));

    let top = wide.bound(Direction::Top);
    assert!(point_close(top.a, Point::new(0.0, 0.0)));
    assert!(point_close(top.b, Point::new(4.0, 0.0)));
    let right = wide.bound(Direction::Right);
    assert!(point_close(right.a, Point::new(4.0, 0.0)));
    assert!(point_close(right.b, Point::new(4.0, 1.0)));

    let mut scaled = Aabb::new(Point::new(0.0, 0.0), Point::new(2.0, 2.0));
    scaled.scale(0.5);
    assert!(point_close(scaled.mins, Point::new(0.5, 0.5)));
    assert!(point_close(scaled.maxs, Point::new(1.5, 1.5)));
}

#[test]
fn segment_contains_in_span_is_inclusive() {
    let s = Segment::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0));
    assert!(s.contains_in_span(Point::new(0.0, 0.0)));
    assert!(s.contains_in_span(Point::new(2.0, 5.0)));
    assert!(s.contains_in_span(Point::new(4.0, 0.0)));
    assert!(!s.contains_in_span(Point::new(5.0, 0.0)));
}
