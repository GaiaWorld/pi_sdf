/**
 * 纹理布局计算（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/core/raster/layout.rs
 */

use crate::model::base::error::{Error, Result};
use crate::model::geom::aabb::Aabb;
use crate::model::geom::primitive::Point;
use crate::model::raster::sdf::TextureLayout;

/**
 * 由轮廓范围与参数计算纹理布局。
 *
 * 契约：API-035
 *
 * 约束：
 *   - requires  tex_size > 0；pxrange > 0；units_per_em > 0
 *   - ensures   结果稳定可复现；distance = 每像素距离 × pxrange；tex_size > 0、pxrange > 0、distance > 0（TERM-011、TERM-012）
 *   - 错误      InvalidParam —— 参数非正
 *
 * 参数：extents — 轮廓包围盒
 *
 * 参数：tex_size — 纹理边长，正整数
 *
 * 参数：pxrange — 距离像素范围，正整数
 *
 * 参数：units_per_em — 字体单位，正数
 *
 * 参数：offset — 边距像素数
 *
 * 参数：is_svg — 是否 SVG 路径（否则字形）
 */
pub fn compute_layout(
    extents: Aabb,
    tex_size: u32,
    pxrange: u32,
    units_per_em: f32,
    offset: u32,
    is_svg: bool,
) -> Result<TextureLayout> {
    // ① 校验三个正参数：非正（含 NaN）一律返回 Err，不 panic（REQ-004.3）
    if tex_size == 0 {
        return Err(Error::InvalidParam("tex_size 必须为正"));
    }
    if pxrange == 0 {
        return Err(Error::InvalidParam("pxrange 必须为正"));
    }
    if !units_per_em.is_finite() || units_per_em <= 0.0 {
        return Err(Error::InvalidParam("units_per_em 必须为正有限值"));
    }

    // ② 平面边界 = 轮廓范围按 1/units_per_em 逐分量缩放（与 pi_sdf 的 Aabb::scaled 一致，绕原点而非中心）
    let extents_w = extents.width();
    let extents_h = extents.height();
    let inv_units = 1.0 / units_per_em;
    let plane_bounds = Aabb::new(
        Point::new(extents.mins.x * inv_units, extents.mins.y * inv_units),
        Point::new(extents.maxs.x * inv_units, extents.maxs.y * inv_units),
    );

    // ③ 每像素距离与总距离：distance = 每像素距离 × pxrange（TERM-011、TERM-012）
    let px_distance = extents_w.max(extents_h) / tex_size as f32;
    let distance = px_distance * pxrange as f32;
    let expand = px_distance * offset as f32;

    // ④ 按 offset 个像素向四周扩边，为距离场边缘留出余量
    let mut expanded = extents;
    expanded.mins.x -= expand;
    expanded.mins.y -= expand;
    expanded.maxs.x += expand;
    expanded.maxs.y += expand;

    let final_tex = tex_size + offset * 2;
    let mut atlas_bounds = Aabb::new(
        Point::new(offset as f32, offset as f32),
        Point::new(final_tex as f32 - offset as f32, final_tex as f32 - offset as f32),
    );

    // ⑤ 非方形轮廓：沿短边补足，并按「字形 / 路径」两种约定修正图集边界
    let temp = extents_w - extents_h;
    if temp > 0.0 {
        expanded.maxs.y += temp;
        if expanded.height() > 0.0 {
            let adjust = (temp / expanded.height() * final_tex as f32).trunc();
            if is_svg {
                atlas_bounds.maxs.y -= adjust;
            } else {
                // 字形：y 轴在图集内向下翻转，改修下边界
                atlas_bounds.mins.y += (temp / expanded.height() * final_tex as f32).ceil();
            }
        }
    } else if expanded.width() > 0.0 {
        expanded.maxs.x -= temp;
        atlas_bounds.maxs.x -= (temp.abs() / expanded.width() * final_tex as f32).trunc();
    }

    debug_assert!(atlas_bounds.mins.x <= atlas_bounds.maxs.x);
    debug_assert!(atlas_bounds.mins.y <= atlas_bounds.maxs.y);

    Ok(TextureLayout {
        plane_bounds,
        atlas_bounds,
        distance,
        tex_size: final_tex,
    })
}

#[cfg(test)]
mod tests {
    use crate::core::raster::layout::compute_layout;
    use crate::model::base::error::Error;
    use crate::model::geom::aabb::Aabb;
    use crate::model::geom::primitive::Point;

    /// 方形轮廓范围（1000×1000），便于验证补方与缩放。
    fn square_extents() -> Aabb {
        Aabb::new(Point::new(0.0, 0.0), Point::new(1000.0, 1000.0))
    }

    // CASE-041：同一输入两次结果完全一致（消除现状的哈希无序不稳定）。
    #[test]
    fn case_041_compute_layout_is_reproducible() {
        let first = compute_layout(square_extents(), 64, 4, 1000.0, 0, false).expect("应成功");
        let second = compute_layout(square_extents(), 64, 4, 1000.0, 0, false).expect("应成功");
        assert_eq!(first, second, "同一输入两次布局必须完全一致");

        let svg = compute_layout(square_extents(), 64, 4, 1000.0, 0, true).expect("应成功");
        let svg_again = compute_layout(square_extents(), 64, 4, 1000.0, 0, true).expect("应成功");
        assert_eq!(svg, svg_again, "路径链路的布局也应可复现");
    }

    // CASE-042：参数非正返回 Err，不 panic（REQ-004.3）。
    #[test]
    fn case_042_non_positive_params_return_invalid_param() {
        assert!(matches!(
            compute_layout(square_extents(), 0, 4, 1000.0, 0, false),
            Err(Error::InvalidParam(_))
        ));
        assert!(matches!(
            compute_layout(square_extents(), 64, 0, 1000.0, 0, false),
            Err(Error::InvalidParam(_))
        ));
        for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
            assert!(
                matches!(
                    compute_layout(square_extents(), 64, 4, bad, 0, false),
                    Err(Error::InvalidParam(_))
                ),
                "units_per_em={} 应返回 Err",
                bad
            );
        }
    }

    // 自洽：distance = 每像素距离 × pxrange；偏移使纹理边长增加 2×offset。
    #[test]
    fn distance_matches_pixel_distance_times_pxrange() {
        let extents = square_extents();
        let layout = compute_layout(extents, 64, 4, 1.0, 0, false).expect("应成功");
        let px_distance = extents.width().max(extents.height()) / 64.0;
        assert!((layout.distance - px_distance * 4.0).abs() <= 1e-6);
        assert!((layout.plane_bounds.width() - extents.width()).abs() <= 1e-3);
        assert!(layout.tex_size > 0 && layout.distance > 0.0);

        let padded = compute_layout(extents, 64, 4, 1.0, 2, false).expect("应成功");
        assert_eq!(padded.tex_size, 68, "offset 应使纹理边长增加 2×offset");
        assert!((padded.atlas_bounds.mins.x - 2.0).abs() <= 1e-6);
        assert!((padded.atlas_bounds.maxs.x - 66.0).abs() <= 1e-6);
    }
}
