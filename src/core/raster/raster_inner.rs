/**
 * SDF 纹理光栅化（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/raster/raster.rs
 */

use crate::core::sdf::sdf::sdf_from_arcs;
use crate::model::base::error::{Error, Result};
use crate::model::geom::aabb::Aabb;
use crate::model::geom::curve::Arc;
use crate::model::geom::primitive::Point;
use crate::model::grid::grid::CellGrid;
use crate::model::raster::sdf::{RasterOptions, SdfTexture, TextureInfo, TextureLayout};

/**
 * 按格元将距离场光栅化为 SDF 纹理。
 *
 * 契约：API-036
 *
 * 约束：
 *   - requires  layout 的 tex_size 与目标纹理一致；grid 的索引有效
 *   - ensures   pixels 长度 = tex_size × tex_size；逐像素值与现状等价；复杂度同阶
 *   - 错误      InvalidParam —— 网格索引越界；Geometry —— 布局与网格范围不符
 *
 * 参数：grid — 近邻弧网格
 *
 * 参数：layout — 纹理布局
 *
 * 参数：opts — 光栅化选项
 */
pub fn rasterize(grid: &CellGrid, layout: &TextureLayout, opts: &RasterOptions) -> Result<SdfTexture> {
    // ① 索引越界属调用方可纠正的输入错误：先整体校验，避免逐像素才失败（CASE-045）
    if !grid.is_valid() {
        return Err(Error::InvalidParam("网格索引越界"));
    }

    let side = layout.tex_size as usize;
    let pixel_count = side
        .checked_mul(side)
        .ok_or(Error::InvalidParam("纹理边长过大"))?;

    let distance = layout.distance;
    if !distance.is_finite() || distance <= 0.0 {
        return Err(Error::Geometry("布局距离非正，无法量化"));
    }

    // ② 由「网格范围 ↔ 图集边界」还原像素几何；两者必须相符（API-036 错误模式）
    let atlas = layout.atlas_bounds;
    let span_x = atlas.maxs.x - atlas.mins.x;
    let span_y = atlas.maxs.y - atlas.mins.y;
    let grid_w = grid.extents.width();
    let grid_h = grid.extents.height();
    if !(span_x > 0.0) || !(span_y > 0.0) || !(grid_w > 0.0) || !(grid_h > 0.0) {
        return Err(Error::Geometry("布局与网格范围不符"));
    }
    let unit_x = grid_w / span_x;
    let unit_y = grid_h / span_y;
    let scale_tol = 1e-3 * unit_x.abs().max(unit_y.abs());
    if (unit_x - unit_y).abs() > scale_tol {
        return Err(Error::Geometry("布局与网格范围不符：横纵像素尺度不一致"));
    }
    // 单个像素在网格坐标系下的边长：与 pi_sdf encode_sdf 的 unit_d 一致
    let unit_d = unit_x;
    // 网格范围各向外扩 offset 个像素得到栅格覆盖的方格（对应 pi_sdf 的 extents2）
    let plane_box = Aabb::new(
        Point::new(
            grid.extents.mins.x - atlas.mins.x * unit_d,
            grid.extents.mins.y - atlas.mins.y * unit_d,
        ),
        Point::new(
            grid.extents.maxs.x + atlas.mins.x * unit_d,
            grid.extents.maxs.y + atlas.mins.y * unit_d,
        ),
    );

    // ③ 逐格元 × 覆盖像素 × 近邻弧采样；未被格元覆盖的像素保持 0（与现状一致）
    let mut pixels = vec![0u8; pixel_count];
    for cell in &grid.cells {
        let covered = match cell.bounds.collision(&plane_box) {
            Some(box_) => box_,
            None => continue,
        };
        let begin_x = pixel_index(covered.mins.x - plane_box.mins.x, unit_d, side);
        let begin_y = pixel_index(covered.mins.y - plane_box.mins.y, unit_d, side);
        let end_x = pixel_index(covered.maxs.x - plane_box.mins.x, unit_d, side);
        let end_y = pixel_index(covered.maxs.y - plane_box.mins.y, unit_d, side);
        if begin_x >= end_x || begin_y >= end_y {
            continue;
        }
        // 每格元一次性取出近邻弧，避免逐像素分配（复杂度与现状同阶）
        let near: Vec<Arc> = cell
            .arc_indices
            .iter()
            .map(|&index| grid.arcs[index])
            .collect();
        for i in begin_x..end_x {
            let px = (i as f32 + 0.5) * unit_d + plane_box.mins.x;
            for j in begin_y..end_y {
                let py = (j as f32 + 0.5) * unit_d + plane_box.mins.y;
                let mut sdf = sdf_from_arcs(&near, Point::new(px, py)).distance;
                // 与现状一致：先按 1e-4 量化，抹去浮点抖动
                sdf = (sdf * 10000.0).round() * 0.0001;
                if let Some(true) = opts.is_reverse {
                    sdf = -sdf;
                }
                let alpha = shade(sdf, distance, opts.is_outer_glow);
                // is_svg 直接按行存放；字形链路翻转 y 轴以适配纹理坐标系
                let at = if opts.is_svg {
                    j * side + i
                } else {
                    (side - 1 - j) * side + i
                };
                pixels[at] = alpha;
            }
        }
    }

    Ok(SdfTexture {
        pixels,
        tex_size: layout.tex_size,
        tex_info: TextureInfo {
            plane_bounds: layout.plane_bounds,
            atlas_bounds: layout.atlas_bounds,
            // 渲染端按 plane/atlas 边界换算定位；偏移保持 0（与 pi_sdf 一致）
            sdf_offset_x: 0.0,
            sdf_offset_y: 0.0,
        },
    })
}

/// 把网格坐标偏移换算为像素下标：四舍五入后夹取到 [0, side]。
fn pixel_index(coord: f32, unit_d: f32, side: usize) -> usize {
    let snapped = ((coord / unit_d) * 10000.0).round() * 0.0001;
    let rounded = snapped.round();
    if !(rounded > 0.0) {
        0
    } else if rounded >= side as f32 {
        side
    } else {
        rounded as usize
    }
}

/// 由有符号距离量化灰度：与 pi_sdf compute_sdf2 的两种映射一致。
fn shade(sdf: f32, distance: f32, is_outer_glow: bool) -> u8 {
    if is_outer_glow {
        let alpha = (1.0 - sdf / distance).powf(1.99) * 255.0;
        alpha.round() as u8
    } else {
        let alpha = (1.0 - sdf / distance) * 127.0;
        alpha.round() as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::core::grid::grid::compute_cell_grid;
    use crate::core::raster::layout::compute_layout;
    use crate::core::raster::raster::rasterize;
    use crate::model::base::error::Error;
    use crate::model::geom::aabb::Aabb;
    use crate::model::geom::curve::Arc;
    use crate::model::geom::primitive::Point;
    use crate::model::grid::grid::{Cell, CellGrid};
    use crate::model::outline::contour::{Contour, Outline};
    use crate::model::raster::sdf::{RasterOptions, TextureLayout};

    const TEX: u32 = 64;
    const PXRANGE: u32 = 4;

    /// 单位正方形轮廓（(0,0)-(1,1)，逆时针四条线段）。
    fn square_outline() -> Outline {
        let arcs = vec![
            Arc::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0), 0.0),
            Arc::new(Point::new(1.0, 0.0), Point::new(1.0, 1.0), 0.0),
            Arc::new(Point::new(1.0, 1.0), Point::new(0.0, 1.0), 0.0),
            Arc::new(Point::new(0.0, 1.0), Point::new(0.0, 0.0), 0.0),
        ];
        Outline::new(vec![Contour::new(arcs, true)])
    }

    fn grid_and_layout() -> (CellGrid, TextureLayout) {
        let grid = compute_cell_grid(&square_outline(), 0.25, false).expect("网格应成功");
        let layout =
            compute_layout(grid.extents, TEX, PXRANGE, 1.0, 0, false).expect("布局应成功");
        (grid, layout)
    }

    fn plain_opts() -> RasterOptions {
        RasterOptions {
            is_outer_glow: false,
            is_svg: false,
            is_reverse: None,
        }
    }

    // CASE-044：pixels 长度等于 tex_size²。
    #[test]
    fn case_044_texture_pixel_count_matches_tex_size() {
        let (grid, layout) = grid_and_layout();
        let tex = rasterize(&grid, &layout, &plain_opts()).expect("光栅化应成功");
        assert_eq!(tex.tex_size, layout.tex_size);
        assert_eq!(tex.pixels.len(), (tex.tex_size * tex.tex_size) as usize);
        assert!(tex.is_consistent());
    }

    // CASE-047：同一输入两次结果完全一致（确定性，REQ-001.2）。
    #[test]
    fn case_047_rasterize_is_reproducible() {
        let (grid, layout) = grid_and_layout();
        let opts = RasterOptions {
            is_outer_glow: true,
            is_svg: false,
            is_reverse: Some(false),
        };
        let first = rasterize(&grid, &layout, &opts).expect("首次应成功");
        let second = rasterize(&grid, &layout, &opts).expect("二次应成功");
        assert_eq!(first.pixels, second.pixels);
        assert_eq!(first.tex_size, second.tex_size);
        assert_eq!(first.tex_info, second.tex_info);
    }

    // CASE-045：越界网格索引返回 Err，不 panic。
    #[test]
    fn case_045_out_of_range_index_returns_err() {
        let grid = CellGrid {
            extents: Aabb::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
            arcs: Vec::new(),
            cells: vec![Cell {
                bounds: Aabb::new(Point::new(0.0, 0.0), Point::new(0.5, 0.5)),
                arc_indices: vec![0],
            }],
            min_width: 1.0,
            min_height: 1.0,
            is_area: false,
        };
        let layout = compute_layout(grid.extents, 32, PXRANGE, 1.0, 0, false).expect("布局应成功");
        assert!(matches!(
            rasterize(&grid, &layout, &plain_opts()),
            Err(Error::InvalidParam(_))
        ));
    }

    // 自洽：内部像素比外部像素更亮，灰度落在 u8 值域。
    #[test]
    fn inside_pixels_brighter_than_outside() {
        let (grid, layout) = grid_and_layout();
        let tex = rasterize(&grid, &layout, &plain_opts()).expect("光栅化应成功");
        let side = tex.tex_size as usize;
        let center = tex.pixels[(side / 2) * side + side / 2];
        let corner = tex.pixels[0];
        assert!(center > 127, "内部像素应亮于中点阈值，实际 {}", center);
        assert!(corner < 127, "角落在形状外，应暗于中点阈值，实际 {}", corner);
    }

    // 自洽：is_reverse 翻转符号后灰度互补（covered 像素 ≈ 254 - a；未覆盖同为 0）。
    #[test]
    fn is_reverse_complements_alpha() {
        let (grid, layout) = grid_and_layout();
        let normal = rasterize(&grid, &layout, &plain_opts()).expect("应成功");
        let reversed = rasterize(
            &grid,
            &layout,
            &RasterOptions {
                is_outer_glow: false,
                is_svg: false,
                is_reverse: Some(true),
            },
        )
        .expect("应成功");
        for (a, b) in normal.pixels.iter().zip(reversed.pixels.iter()) {
            let sum = *a as i32 + *b as i32;
            assert!(
                sum == 0 || (sum - 254).abs() <= 1,
                "翻转前后应互补或同为未覆盖 0，实际 {} + {} = {}",
                a,
                b,
                sum
            );
        }
    }

    // 自洽：is_svg 仅上下翻转像素行。
    #[test]
    fn is_svg_flips_rows() {
        let (grid, layout) = grid_and_layout();
        let geom = rasterize(&grid, &layout, &plain_opts()).expect("应成功");
        let svg = rasterize(
            &grid,
            &layout,
            &RasterOptions {
                is_outer_glow: false,
                is_svg: true,
                is_reverse: None,
            },
        )
        .expect("应成功");
        let side = geom.tex_size as usize;
        for j in 0..side {
            for i in 0..side {
                assert_eq!(
                    svg.pixels[j * side + i],
                    geom.pixels[(side - 1 - j) * side + i]
                );
            }
        }
    }
}
