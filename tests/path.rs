//! TASK-11 路径与图元：仅经冻结接口验证（CASE-053 / 054 / 056 / 057 / 058 / 059 / 082）。
//!
//! CASE-055（与 pi_sdf 逐像素等价）按用户决定推迟到实现完成后统一做 PNG 对比；
//! 本文件以自洽性断言替代（尺寸正确、可复现、非法参数返回 Err）。
//! 断言用数学性质与边界条件，坐标容差 1e-6 量级。

use pi_sdf2::model::{Error, Path, PathVerb, Shape};

const TOL: f32 = 1e-6;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOL
}

/// CASE-053：非法动词判别值（含 0、20、200、255）返回 Err(InvalidPathVerb)，不 UB。
#[test]
fn case_053_invalid_verb_byte_returns_err() {
    for bad in [0u8, 20, 200, 255] {
        match PathVerb::try_from(bad) {
            Err(Error::InvalidPathVerb(value)) => assert_eq!(value, bad, "错误应携带原始判别值"),
            other => panic!("字节 {} 应返回 InvalidPathVerb，实际 {:?}", bad, other),
        }
        assert!(PathVerb::try_from_u8(bad).is_err(), "试转入口也应返回 Err");
    }
}

/// CASE-054：1..=19 全部判别值往返一致（与 JS u8 契约一致）。
#[test]
fn case_054_verb_roundtrip_matches_contract() {
    for value in 1u8..=19 {
        let verb = PathVerb::try_from(value).expect("合法判别值应还原为动词");
        assert_eq!(verb.to_u8(), value, "to_u8 应与输入一致");
        assert_eq!(u8::from(verb), value, "From<PathVerb> for u8 应与输入一致");
        assert_eq!(
            PathVerb::try_from_u8(value).expect("应成功").to_u8(),
            value,
            "try_from_u8 往返应一致"
        );
    }
}

/// CASE-056：坐标数量与动词不匹配返回 Err，不 panic。
#[test]
fn case_056_point_count_mismatch_returns_err() {
    // MoveTo + LineTo 需要 2 个点（4 个分量），只给 1 个点。
    assert!(matches!(
        Path::new(vec![1, 3], vec![0.0, 0.0]),
        Err(Error::InvalidParam(_))
    ));
    // 坐标过多同样非法。
    assert!(matches!(
        Path::new(vec![1], vec![0.0, 0.0, 1.0, 1.0]),
        Err(Error::InvalidParam(_))
    ));
    // 二次贝塞尔需要 2 个点；只给 1 个点非法。
    assert!(matches!(
        Path::new(vec![1, 5], vec![0.0, 0.0, 1.0, 1.0]),
        Err(Error::InvalidParam(_))
    ));
    // 合法：MoveTo + LineTo 恰两点。
    assert!(Path::new(vec![1, 3], vec![0.0, 0.0, 3.0, 4.0]).is_ok());
    // 非法动词优先于坐标校验返回 InvalidPathVerb。
    assert!(matches!(
        Path::new(vec![200], vec![]),
        Err(Error::InvalidPathVerb(200))
    ));
}

/// CASE-057：路径包围盒等于坐标点的最小/最大（含负坐标与直线退化维）。
#[test]
fn case_057_extents_cover_coordinates() {
    let path = Path::new(vec![1, 3], vec![0.0, 0.0, 3.0, 4.0]).expect("应构造成功");
    let bb = path.extents();
    assert!(close(bb.mins.x, 0.0) && close(bb.mins.y, 0.0));
    assert!(close(bb.maxs.x, 3.0) && close(bb.maxs.y, 4.0));
    assert!(close(bb.width(), 3.0) && close(bb.height(), 4.0));

    let shifted = Path::new(vec![1, 3], vec![-2.0, -5.0, 1.0, -1.0]).expect("应构造成功");
    let sbb = shifted.extents();
    assert!(close(sbb.mins.x, -2.0) && close(sbb.mins.y, -5.0));
    assert!(close(sbb.maxs.x, 1.0) && close(sbb.maxs.y, -1.0));
}

/// CASE-058：六种图元各自 to_outline —— 弧数、闭合性、绕向与包围盒自洽。
#[test]
fn case_058_six_shapes_outline_and_extents() {
    // 圆：四个 90 度圆弧，逆时针，包围盒 cx±r、cy±r。
    let circle = Shape::circle(1.0, 2.0, 2.0);
    let circle_outline = circle.to_outline().expect("圆轮廓应成功");
    assert_eq!(circle_outline.contours.len(), 1, "圆应为单个子轮廓");
    assert_eq!(circle_outline.contours[0].arcs.len(), 4, "圆应为四段弧");
    assert!(circle_outline.contours[0].is_closed);
    assert!(!circle_outline.contours[0].is_clockwise(), "圆应为逆时针");
    let cbb = circle_outline.extents();
    assert!(close(cbb.mins.x, -1.0) && close(cbb.maxs.x, 3.0));
    assert!(close(cbb.mins.y, 0.0) && close(cbb.maxs.y, 4.0));

    // 矩形：四条线段，逆时针，包围盒 x..x+w、y..y+h。
    let rect = Shape::rect(-1.0, -1.0, 3.0, 2.0);
    let rect_outline = rect.to_outline().expect("矩形轮廓应成功");
    assert_eq!(rect_outline.contours[0].arcs.len(), 4);
    assert!(rect_outline.contours[0].is_closed);
    assert!(!rect_outline.contours[0].is_clockwise(), "矩形应为逆时针");
    let rbb = rect_outline.extents();
    assert!(close(rbb.mins.x, -1.0) && close(rbb.maxs.x, 2.0));
    assert!(close(rbb.mins.y, -1.0) && close(rbb.maxs.y, 1.0));

    // 线段：单条弧、开放、非面积。
    let line = Shape::line(0.0, 0.0, 2.0, 0.0, None);
    let line_outline = line.to_outline().expect("线段轮廓应成功");
    assert_eq!(line_outline.contours[0].arcs.len(), 1);
    assert!(!line_outline.contours[0].is_closed);
    assert!(!line.is_area());

    // 椭圆：非空轮廓，包围盒 cx±rx、cy±ry（离散误差 0.01 内）。
    let ellipse = Shape::ellipse(0.0, 0.0, 3.0, 1.0);
    let ellipse_outline = ellipse.to_outline().expect("椭圆轮廓应成功");
    assert!(!ellipse_outline.contours.is_empty());
    assert!(ellipse_outline.contours[0].is_closed);
    let ebb = ellipse_outline.extents();
    assert!((ebb.mins.x + 3.0).abs() <= 0.01 && (ebb.maxs.x - 3.0).abs() <= 0.01);
    assert!((ebb.mins.y + 1.0).abs() <= 0.01 && (ebb.maxs.y - 1.0).abs() <= 0.01);

    // 多边形：三点三角形，闭合逆时针。
    let polygon = Shape::polygon(vec![0.0, 0.0, 4.0, 0.0, 0.0, 3.0]);
    let polygon_outline = polygon.to_outline().expect("多边形轮廓应成功");
    assert!(polygon_outline.contours[0].is_closed);
    assert_eq!(polygon_outline.contours[0].arcs.len(), 3);
    assert!(!polygon_outline.contours[0].is_clockwise(), "多边形应为逆时针");
    let pbb = polygon_outline.extents();
    assert!(close(pbb.mins.x, 0.0) && close(pbb.maxs.x, 4.0));
    assert!(close(pbb.mins.y, 0.0) && close(pbb.maxs.y, 3.0));

    // 折线：开放、非面积。
    let polyline = Shape::polyline(vec![0.0, 0.0, 1.0, 2.0, 3.0, 0.0], false);
    let polyline_outline = polyline.to_outline().expect("折线轮廓应成功");
    assert_eq!(polyline_outline.contours[0].arcs.len(), 2);
    assert!(!polyline_outline.contours[0].is_closed);
    assert!(!polyline.is_area());

    // 面积判定：线段类为假，其余为真。
    assert!(circle.is_area() && rect.is_area() && ellipse.is_area() && polygon.is_area());
}

/// CASE-059：半径为负的圆/椭圆、宽高为负的矩形返回 Err(InvalidParam)，不 panic。
#[test]
fn case_059_negative_size_returns_err() {
    assert!(matches!(
        Shape::circle(0.0, 0.0, -1.0).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    assert!(matches!(
        Shape::ellipse(0.0, 0.0, -2.0, 1.0).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    assert!(matches!(
        Shape::ellipse(0.0, 0.0, 1.0, -2.0).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    assert!(matches!(
        Shape::rect(0.0, 0.0, -1.0, 2.0).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    // 零半径非负，允许（退化轮廓，非错误路径）。
    assert!(Shape::circle(0.0, 0.0, 0.0).to_outline().is_ok());
}

/// CASE-082：点数不足 3 的多边形返回 Err(InvalidParam)，不 panic。
#[test]
fn case_082_polygon_too_few_points_returns_err() {
    assert!(matches!(
        Shape::polygon(vec![0.0, 0.0, 1.0, 1.0]).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    assert!(matches!(
        Shape::polygon(vec![0.0, 0.0]).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    assert!(matches!(
        Shape::polygon(Vec::new()).to_outline(),
        Err(Error::InvalidParam(_))
    ));
    // 点数足够则应成功。
    assert!(Shape::polygon(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0]).to_outline().is_ok());
    // 折线点数不足同样非法。
    assert!(matches!(
        Shape::polyline(vec![0.0, 0.0], false).to_outline(),
        Err(Error::InvalidParam(_))
    ));
}

/// CASE-055 的自洽替代：SDF 纹理由「轮廓 → 网格 → 布局 → 光栅化」链路产出，
/// 尺寸正确、可复现，非法参数返回 Err。
#[test]
fn sdf_texture_is_consistent_and_reproducible() {
    // 闭合矩形路径（绝对动词），逆时针。
    let path = Path::new(
        vec![1, 3, 3, 3, 19],
        vec![0.0, 0.0, 8.0, 0.0, 8.0, 6.0, 0.0, 6.0],
    )
    .expect("合法路径应构造成功");

    let texture = path.sdf_texture(32, 4).expect("合法参数应生成纹理");
    assert_eq!(texture.tex_size, 32, "offset 为 0 时纹理边长应等于 tex_size");
    assert_eq!(texture.pixels.len(), 32 * 32, "像素数应为 tex_size²");
    assert!(texture.is_consistent());

    let again = path.sdf_texture(32, 4).expect("重复生成应成功");
    assert_eq!(texture.pixels, again.pixels, "同一输入两次结果应一致");
    assert_eq!(texture.tex_info, again.tex_info);

    // 参数非正返回 Err，不 panic。
    assert!(matches!(path.sdf_texture(0, 4), Err(Error::InvalidParam(_))));
    assert!(matches!(path.sdf_texture(32, 0), Err(Error::InvalidParam(_))));
}
