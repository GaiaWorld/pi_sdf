//! TASK-04 轮廓与绕向：仅经冻结接口 Contour / Outline 调用（CASE-020..023）。
//!
//! 断言基于数学性质（绕向、端点往返、内外判定）与边界条件，容差 1e-6。

use pi_sdf2::model::outline::contour::{contour_winding, even_odd, outline_winding};
use pi_sdf2::model::outline::{Contour, Outline};
use pi_sdf2::model::{Arc, ArcEndpoint, Point};

const TOL: f32 = 1e-6;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOL
}

fn point_close(a: Point, b: Point) -> bool {
    close(a.x, b.x) && close(a.y, b.y)
}

fn line(a: Point, b: Point) -> Arc {
    Arc::new(a, b, 0.0)
}

/// 逆时针正方形 (0,0)->(2,0)->(2,2)->(0,2)。
fn square_ccw() -> Contour {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(2.0, 0.0);
    let c = Point::new(2.0, 2.0);
    let d = Point::new(0.0, 2.0);
    Contour::new(vec![line(a, b), line(b, c), line(c, d), line(d, a)], true)
}

/// 顺时针正方形 (0,0)->(0,2)->(2,2)->(2,0)。
fn square_cw() -> Contour {
    let a = Point::new(0.0, 0.0);
    let b = Point::new(2.0, 0.0);
    let c = Point::new(2.0, 2.0);
    let d = Point::new(0.0, 2.0);
    Contour::new(vec![line(a, d), line(d, c), line(c, b), line(b, a)], true)
}

#[test]
fn case_020_reverse_takes_effect_in_place() {
    let mut c = square_cw();
    assert!(c.is_clockwise(), "前置：应为顺时针");
    let before = c.arcs.clone();

    c.reverse();

    assert!(!c.is_clockwise(), "原地反转后应变为逆时针");
    assert!(c.is_closed, "reverse 不改变闭合性");
    // 弧序列确实被改写（说明作用在传入对象本身，而非副本）。
    assert_eq!(c.arcs.len(), before.len());
    assert!(point_close(c.arcs[0].p0, before[before.len() - 1].p1));
    assert!(point_close(c.arcs[0].p1, before[before.len() - 1].p0));
    assert!(close(c.arcs[0].d, -before[before.len() - 1].d));

    c.reverse();
    assert!(c.is_clockwise(), "反转两次应回到原绕向");
}

#[test]
fn case_021_endpoints_roundtrip() {
    let v0 = Point::new(2.0, 0.0);
    let v1 = Point::new(2.0, 2.0);
    let v2 = Point::new(0.0, 2.0);
    let eps = vec![
        ArcEndpoint { px: v0.x, py: v0.y, d: 0.0, tag: None },
        ArcEndpoint { px: v1.x, py: v1.y, d: 0.0, tag: None },
        ArcEndpoint { px: v2.x, py: v2.y, d: 0.5, tag: None },
        ArcEndpoint { px: v0.x, py: v0.y, d: 0.0, tag: None },
    ];
    let c = Contour::from_endpoints(eps.clone());
    assert!(c.is_closed, "首尾相接应判为闭合");
    assert_eq!(c.arcs.len(), 3);
    assert!(close(c.arcs[1].d, 0.5), "第二条弧应携带端点给定的 d");

    let out = c.endpoints();
    assert_eq!(out.len(), eps.len());
    assert_eq!(out, eps, "endpoints -> from_endpoints -> endpoints 应一致");
}

#[test]
fn case_021_open_endpoints_not_closed() {
    let open = Contour::from_endpoints(vec![
        ArcEndpoint { px: 0.0, py: 0.0, d: 0.0, tag: None },
        ArcEndpoint { px: 1.0, py: 0.0, d: 0.0, tag: None },
        ArcEndpoint { px: 2.0, py: 0.0, d: 0.0, tag: None },
    ]);
    assert!(!open.is_closed, "首尾不接应返回未闭合轮廓");
    assert_eq!(open.arcs.len(), 2);
}

#[test]
fn case_022_outline_winding_rewrites_in_place() {
    let mut o = Outline::new(vec![square_cw()]);
    assert!(o.is_clockwise(), "前置：轮廓需要反转");

    outline_winding(&mut o, true);

    assert!(!o.is_clockwise(), "改写须对传入 outline 可见");
    assert!(!o.contours[0].is_clockwise(), "改写须对传入对象的子轮廓可见（非副本）");

    outline_winding(&mut o, true);
    assert!(!o.is_clockwise(), "已满足目标绕向时重复调用不再翻转");
}

#[test]
fn outline_winding_alternates_nested_contours() {
    let outer = Contour::new(
        vec![
            line(Point::new(0.0, 0.0), Point::new(4.0, 0.0)),
            line(Point::new(4.0, 0.0), Point::new(4.0, 4.0)),
            line(Point::new(4.0, 4.0), Point::new(0.0, 4.0)),
            line(Point::new(0.0, 4.0), Point::new(0.0, 0.0)),
        ],
        true,
    );
    let hole = Contour::new(
        vec![
            line(Point::new(1.0, 1.0), Point::new(3.0, 1.0)),
            line(Point::new(3.0, 1.0), Point::new(3.0, 3.0)),
            line(Point::new(3.0, 3.0), Point::new(1.0, 3.0)),
            line(Point::new(1.0, 3.0), Point::new(1.0, 1.0)),
        ],
        true,
    );
    assert!(!outer.is_clockwise() && !hole.is_clockwise(), "前置：两环均逆时针");

    let mut o = Outline::new(vec![outer, hole]);
    outline_winding(&mut o, false);

    assert!(o.contours[0].is_clockwise(), "外轮廓应被翻转为顺时针");
    assert!(!o.contours[1].is_clockwise(), "洞应保持与外轮廓相反（奇偶修正）");
}

#[test]
fn contour_winding_targets_requested_orientation() {
    let mut c = square_ccw();
    contour_winding(&mut c, false);
    assert!(c.is_clockwise(), "inverse=false 的目标是顺时针");
    contour_winding(&mut c, false);
    assert!(c.is_clockwise(), "重复调用不再翻转");

    contour_winding(&mut c, true);
    assert!(!c.is_clockwise(), "inverse=true 的目标是逆时针");
}

#[test]
fn case_023_even_odd_inside_point() {
    let c = square_ccw();
    assert!(even_odd(&c, Point::new(1.0, 1.0)), "闭合轮廓内一点应返回真");
    assert!(!even_odd(&c, Point::new(3.0, 1.0)), "右侧外点应返回假");
    assert!(!even_odd(&c, Point::new(-1.0, 1.0)), "左侧外点应返回假");
    assert!(!even_odd(&c, Point::new(1.0, 3.0)), "上方外点应返回假");

    // 由两段半圆构成的单位圆。
    let circle = Contour::new(
        vec![
            Arc::new(Point::new(1.0, 0.0), Point::new(-1.0, 0.0), 1.0),
            Arc::new(Point::new(-1.0, 0.0), Point::new(1.0, 0.0), 1.0),
        ],
        true,
    );
    assert!(even_odd(&circle, Point::new(0.0, 0.5)), "圆内点应为真");
    assert!(!even_odd(&circle, Point::new(0.0, 1.5)), "圆外点应为假");
}

#[test]
fn contour_and_outline_extents() {
    let c = square_ccw();
    let bb = c.extents();
    assert!(point_close(bb.mins, Point::new(0.0, 0.0)));
    assert!(point_close(bb.maxs, Point::new(2.0, 2.0)));

    let o = Outline::new(vec![c]);
    let ob = o.extents();
    assert!(point_close(ob.mins, Point::new(0.0, 0.0)));
    assert!(point_close(ob.maxs, Point::new(2.0, 2.0)));

    assert!(Contour::new(vec![], false).extents().is_empty());
    assert!(Outline::new(vec![]).extents().is_empty());
}
