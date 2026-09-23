/**
 * 四叉细分（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/grid/grid.rs
 */

use crate::model::base::error::{Error, Result};
use crate::model::geom::aabb::Aabb;
use crate::model::geom::curve::Arc;
use crate::model::geom::primitive::Point;
use crate::model::outline::contour::Outline;
use crate::model::grid::grid::{Cell, CellGrid};

/// 每轴最大格元数：与 pi_sdf 现状一致（细分到每轴 32 段即停止）。
const MAX_CELLS_PER_AXIS: f32 = 32.0;

/// 细分递归深度硬上限：无论浮点如何抖动都保证终止（REQ-005.1）。
const MAX_SUBDIVISION_DEPTH: u32 = 64;

/**
 * 求与给定包围盒相交的弧索引（core 内部辅助，细分使用）。
 *
 * 契约：API-017
 *
 * 约束：
 *   - requires  scale 为正有限值
 *   - ensures   细分必然终止；结果与现状格元划分一致
 *   - 错误      InvalidParam —— scale 非正
 *
 * 参数：aabb — 查询范围
 * 参数：arcs — 候选弧集合
 */
pub(crate) fn near_arc_indices(aabb: &Aabb, arcs: &[Arc]) -> Vec<usize> {
    // ①遍历弧 ②用弧包围盒与 aabb 求交 ③收集命中索引
    let mut indices = Vec::new();
    for (index, arc) in arcs.iter().enumerate() {
        if aabb.collision(&arc.extents()).is_some() {
            indices.push(index);
        }
    }
    indices
}

/**
 * 由轮廓计算近邻弧网格。
 *
 * 契约：API-017
 *
 * 约束：
 *   - requires  scale 为正有限值
 *   - ensures   细分必然终止；结果与现状格元划分一致
 *   - 错误      InvalidParam —— scale 非正
 *
 * 参数：outline — 输入轮廓
 * 参数：scale — 细分尺度，正有限值
 * 参数：is_area — 是否按面积图元处理
 */
pub fn compute_cell_grid(outline: &Outline, scale: f32, is_area: bool) -> Result<CellGrid> {
    // ①校验 scale 为正有限值（NaN 与非正、无穷一并拒绝）
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::InvalidParam("scale 必须为正有限值"));
    }

    // ②求轮廓包围盒并补方外扩（对应 pi_sdf compute_cell_range 语义）
    let raw = outline.extents();
    let base = if raw.is_empty() {
        // 空轮廓退化到原点盒，保证后续 tiling 与输出自洽（非错误路径）。
        Aabb::new(Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 0.0 })
    } else {
        raw
    };
    let extents = pad_and_square(&base, scale);

    // ③汇总全局弧集合
    let mut arcs = Vec::new();
    for contour in outline.contours.iter() {
        arcs.extend_from_slice(&contour.arcs);
    }

    // ④递归二分细分格元，并求每格元近邻弧
    let mut cells = Vec::new();
    let mut min_width = f32::INFINITY;
    let mut min_height = f32::INFINITY;
    subdivide(
        &extents,
        &extents,
        &arcs,
        0,
        &mut cells,
        &mut min_width,
        &mut min_height,
    );

    // ⑤汇总返回
    debug_assert!(cells.iter().all(|cell| !cell.bounds.is_empty()));
    debug_assert!(cells.iter().all(|cell| cell.is_valid(arcs.len())));
    Ok(CellGrid {
        extents,
        arcs,
        cells,
        min_width,
        min_height,
        is_area,
    })
}

/// 补方并按 scale 外扩：与 pi_sdf compute_cell_range 语义一致。
fn pad_and_square(bbox: &Aabb, scale: f32) -> Aabb {
    let mut b = *bbox;
    let diff = b.width() - b.height();
    if diff > 0.0 {
        b.maxs.y += diff;
    } else {
        b.maxs.x -= diff;
    }
    let pad = scale * 0.5 * b.width();
    b.mins.x -= pad;
    b.mins.y -= pad;
    b.maxs.x += pad;
    b.maxs.y += pad;
    b
}

/// 递归二分细分：近邻弧不超过 2 或达到每轴 32 段上限时落为叶格元。
fn subdivide(
    cell: &Aabb,
    root: &Aabb,
    arcs: &[Arc],
    depth: u32,
    cells: &mut Vec<Cell>,
    min_width: &mut f32,
    min_height: &mut f32,
) {
    let width = cell.width();
    let height = cell.height();
    if width < *min_width {
        *min_width = width;
    }
    if height < *min_height {
        *min_height = height;
    }

    let near = near_arc_indices(cell, arcs);

    // 尺寸上限使用相对容差，避免浮点抖动导致迟迟不触发。
    let tol = 1.0 + 1e-6;
    let at_size_limit = width <= root.width() / MAX_CELLS_PER_AXIS * tol + f32::EPSILON
        && height <= root.height() / MAX_CELLS_PER_AXIS * tol + f32::EPSILON;
    let at_depth_limit = depth >= MAX_SUBDIVISION_DEPTH;

    if near.len() <= 2 || at_size_limit || at_depth_limit {
        // near_arc_indices 按弧序收集，天然去重且升序。
        debug_assert!(near.windows(2).all(|pair| pair[0] < pair[1]));
        cells.push(Cell {
            bounds: *cell,
            arc_indices: near,
        });
        return;
    }

    let (first, second) = cell.half();
    subdivide(&first, root, arcs, depth + 1, cells, min_width, min_height);
    subdivide(&second, root, arcs, depth + 1, cells, min_width, min_height);
}
