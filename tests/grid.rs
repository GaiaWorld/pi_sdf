//! TASK-07 网格细分：仅经冻结接口验证（CASE-029 .. 033、079）。
//!
//! 断言基于数学性质与边界条件（有效索引、tiling、二分对齐、终止性），容差 1e-6 量级。
//! 说明：跨平台一致（CASE-031）与现状一致（CASE-033）在本机以「确定性 + 二分结构」性质验证；
//! 真正的 wasm/native 对照由 wasm32 构建与 TASK-15 等价回归覆盖。

use pi_sdf2::api::compute_cell_grid;
use pi_sdf2::model::{Aabb, Arc, Cell, Contour, Error, Outline, Point};

const TOL: f32 = 1e-6;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOL
}

/// 单位正方形轮廓（四条线段），包围盒恰为 (0,0)-(1,1)。
fn square_outline() -> Outline {
    let arcs = vec![
        Arc::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0), 0.0),
        Arc::new(Point::new(1.0, 0.0), Point::new(1.0, 1.0), 0.0),
        Arc::new(Point::new(1.0, 1.0), Point::new(0.0, 1.0), 0.0),
        Arc::new(Point::new(0.0, 1.0), Point::new(0.0, 0.0), 0.0),
    ];
    Outline::new(vec![Contour::new(arcs, true)])
}

/// r 是否为 1/2^k（k>=0）。
fn is_dyadic_ratio(r: f32) -> bool {
    if !(r > 0.0) {
        return false;
    }
    let k = (1.0 / r).log2();
    (k - k.round()).abs() <= 1e-4 && k.round() >= 0.0
}

#[test]
fn case_029_cell_indices_valid_dedup_sorted() {
    // 直接构造：越界索引非法，落界索引合法（CASE-029 的边界条件）。
    let bounds = Aabb::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
    let out_of_range = Cell {
        bounds,
        arc_indices: vec![0, 2],
    };
    assert!(!out_of_range.is_valid(2), "索引 2 超出弧数量 2，应非法");
    assert!(out_of_range.is_valid(3), "索引 2 落在弧数量 3 内，应合法");

    // 计算路径：每个格元的索引必须落界、去重且升序。
    let grid = compute_cell_grid(&square_outline(), 0.1, false).expect("合法输入应返回网格");
    assert_eq!(grid.arcs.len(), 4);
    for cell in &grid.cells {
        assert!(cell.is_valid(grid.arcs.len()), "格元索引应全部落界");
        let mut expected = cell.arc_indices.clone();
        expected.sort_unstable();
        expected.dedup();
        assert_eq!(expected, cell.arc_indices, "索引应去重且升序");
    }
}

#[test]
fn case_030_grid_valid_and_covers_extents() {
    let grid = compute_cell_grid(&square_outline(), 0.1, false).expect("合法输入应返回网格");
    assert!(grid.is_valid(), "网格索引应整体自洽");
    assert!(!grid.cells.is_empty(), "至少产生一个叶格元");

    for cell in &grid.cells {
        assert!(!cell.bounds.is_empty(), "格元 bounds 应为非空盒");
        assert!(
            grid.extents.includes(&cell.bounds),
            "格元应落在网格范围之内"
        );
    }

    // 递归二分把根盒精确切成互不重叠的叶，面积和应等于根盒面积。
    let total: f32 = grid
        .cells
        .iter()
        .map(|cell| cell.bounds.width() * cell.bounds.height())
        .sum();
    let expected = grid.extents.width() * grid.extents.height();
    assert!(
        (total - expected).abs() <= 1e-4 * expected.max(1.0),
        "格元应恰好铺满 extents：合计 {} vs {}",
        total,
        expected
    );
}

#[test]
fn case_031_partition_reproducible() {
    // native 与 wasm 共用同一实现；本机以两次运行逐格一致作为等价证据。
    let first = compute_cell_grid(&square_outline(), 0.1, false).expect("首次计算应成功");
    let second = compute_cell_grid(&square_outline(), 0.1, false).expect("二次计算应成功");

    assert_eq!(first.extents, second.extents);
    assert_eq!(first.arcs, second.arcs);
    assert_eq!(first.cells.len(), second.cells.len());
    for (a, b) in first.cells.iter().zip(second.cells.iter()) {
        assert_eq!(a.bounds, b.bounds, "同输入格元划分应完全一致");
        assert_eq!(a.arc_indices, b.arc_indices, "同输入弧索引集合应完全一致");
    }
}

#[test]
fn case_032_subdivision_terminates() {
    // 高度重叠的弧包围盒：逼迫细分走到尺寸/深度上限，仍须有限返回。
    let mut arcs = Vec::new();
    for i in 0..16 {
        let bend = i as f32 * 0.05;
        arcs.push(Arc::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            bend,
        ));
    }
    let dense = Outline::new(vec![Contour::new(arcs, true)]);
    let grid = compute_cell_grid(&dense, 1.0, false).expect("密集输入应有限返回");
    assert!(!grid.cells.is_empty());
    assert!(grid.cells.len() <= 4096, "格元数应有界（每轴至多 32 段）");

    // 退化输入：所有弧退化为同一点，extents 无体积，仍须有限返回。
    let p = Point::new(2.0, 3.0);
    let degenerate = Outline::new(vec![Contour::new(vec![Arc::new(p, p, 0.0)], false)]);
    let point_grid = compute_cell_grid(&degenerate, 0.5, false).expect("退化输入应有限返回");
    assert_eq!(point_grid.cells.len(), 1, "退化盒应只产生一个格元");
    assert!(close(point_grid.min_width, 0.0));
    assert!(close(point_grid.min_height, 0.0));

    // 空轮廓：非错误路径，返回自洽空网格。
    let empty = compute_cell_grid(&Outline::new(Vec::new()), 0.5, false).expect("空轮廓应返回 Ok");
    assert!(empty.is_valid());
    assert_eq!(empty.arcs.len(), 0);
    assert_eq!(empty.cells.len(), 1);
}

#[test]
fn case_033_partition_is_dyadic_and_consistent() {
    let grid = compute_cell_grid(&square_outline(), 0.25, false).expect("合法输入应返回网格");
    let extent_w = grid.extents.width();
    let extent_h = grid.extents.height();

    let mut min_cell_w = f32::INFINITY;
    let mut min_cell_h = f32::INFINITY;

    for cell in &grid.cells {
        let ratio_w = cell.bounds.width() / extent_w;
        let ratio_h = cell.bounds.height() / extent_h;
        assert!(is_dyadic_ratio(ratio_w), "宽度应为 extents 的 1/2^k");
        assert!(is_dyadic_ratio(ratio_h), "高度应为 extents 的 1/2^k");

        // 对齐：左上角偏移应是自身边长的整数倍（二分网格对齐）。
        let steps_x = (cell.bounds.mins.x - grid.extents.mins.x) / cell.bounds.width();
        let steps_y = (cell.bounds.mins.y - grid.extents.mins.y) / cell.bounds.height();
        assert!(
            (steps_x - steps_x.round()).abs() <= 1e-4,
            "格元 x 起点应落在二分网格上"
        );
        assert!(
            (steps_y - steps_y.round()).abs() <= 1e-4,
            "格元 y 起点应落在二分网格上"
        );

        min_cell_w = min_cell_w.min(cell.bounds.width());
        min_cell_h = min_cell_h.min(cell.bounds.height());
    }

    assert!(
        close(grid.min_width, min_cell_w),
        "min_width 应等于最窄格元宽度"
    );
    assert!(
        close(grid.min_height, min_cell_h),
        "min_height 应等于最矮格元高度"
    );
    assert!(grid.min_width > 0.0 && grid.min_height > 0.0);
}

#[test]
fn case_079_non_positive_scale_returns_invalid_param() {
    let outline = square_outline();
    for scale in [0.0f32, -1.0, -0.25, f32::NAN, f32::INFINITY] {
        match compute_cell_grid(&outline, scale, false) {
            Err(Error::InvalidParam(_)) => {}
            other => panic!("scale={} 应返回 InvalidParam，实际 {:?}", scale, other.map(|g| g.cells.len())),
        }
    }
    assert!(
        compute_cell_grid(&outline, 1.0, false).is_ok(),
        "正有限 scale 应返回 Ok"
    );
}
