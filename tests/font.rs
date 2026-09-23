//! TASK-10 字体：仅经冻结接口 FontFace / GlyphMetrics 验证。
//!
//! 仓库不含 .ttf fixture，测试读取系统字体；若无可用字体则打印 skip 并跳过。

use pi_sdf2::api::font::FontFace;
use pi_sdf2::model::base::error::Error;
use pi_sdf2::model::geom::primitive::Point;
use pi_sdf2::model::outline::contour::Outline;

/// 坐标比较容差（字体单位，量级 1e-6）。
const COORD_TOLERANCE: f32 = 1e-6;

/// 依次尝试若干系统字体路径，返回首个可读字体字节。
fn system_font_bytes() -> Option<Vec<u8>> {
    let candidates = [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
    ];
    for path in candidates.iter() {
        let p = std::path::Path::new(path);
        if p.exists() {
            if let Ok(bytes) = std::fs::read(p) {
                return Some(bytes);
            }
        }
    }
    None
}

/// 无系统字体时跳过当前用例。
macro_rules! font_or_skip {
    () => {
        match system_font_bytes() {
            Some(bytes) => bytes,
            None => {
                eprintln!("skip: no system font");
                return;
            }
        }
    };
}

/// 断言两点在容差内一致。
fn assert_close(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() <= COORD_TOLERANCE,
        "x 不一致: {} vs {}",
        a.x,
        b.x
    );
    assert!(
        (a.y - b.y).abs() <= COORD_TOLERANCE,
        "y 不一致: {} vs {}",
        a.y,
        b.y
    );
}

/// 轮廓的结构签名：逐子轮廓记录闭合标志与逐弧的位精确坐标。
fn outline_signature(outline: &Outline) -> Vec<(bool, Vec<[u32; 5]>)> {
    outline
        .contours
        .iter()
        .map(|c| {
            let arcs = c
                .arcs
                .iter()
                .map(|a| {
                    [
                        a.p0.x.to_bits(),
                        a.p0.y.to_bits(),
                        a.p1.x.to_bits(),
                        a.p1.y.to_bits(),
                        a.d.to_bits(),
                    ]
                })
                .collect();
            (c.is_closed, arcs)
        })
        .collect()
}

// CASE-048：损坏字体字节返回 Err(InvalidFont)，不 panic（REQ-004.2）。
#[test]
fn case_048_corrupt_bytes_return_invalid_font() {
    assert!(matches!(
        FontFace::from_bytes(vec![0x00, 0x01, 0x02, 0x03, 0x04]),
        Err(Error::InvalidFont(_))
    ));
    assert!(matches!(
        FontFace::from_bytes(Vec::new()),
        Err(Error::InvalidFont(_))
    ));
    if let Some(bytes) = system_font_bytes() {
        // 合法字体被截断（仅保留 sfnt 头）也必须返回 Err，且不 panic
        let head_only = bytes[..12.min(bytes.len())].to_vec();
        assert!(matches!(
            FontFace::from_bytes(head_only),
            Err(Error::InvalidFont(_))
        ));
    }
}

// CASE-051：native 目标下接口可用，且可编译调用 glyph_outline（REQ-002.1）。
#[test]
fn case_051_native_outline_interface_available() {
    let bytes = font_or_skip!();
    let face = FontFace::from_bytes(bytes).expect("合法系统字体应解析成功");
    assert!(face.units_per_em() > 0.0, "units_per_em 必须为正");
    let index = face.glyph_index('A');
    assert!(index > 0, "字体应包含字母 A 的字形");
    let outline = face.glyph_outline('A').expect("A 应可提取轮廓");
    assert!(!outline.contours.is_empty(), "A 的轮廓不应为空");
}

// CASE-049：字形轮廓端点成链、闭合、有限，且包围盒非退化（REQ-001.1）。
#[test]
fn case_049_outline_endpoints_are_chained_and_closed() {
    let bytes = font_or_skip!();
    let face = FontFace::from_bytes(bytes).expect("合法系统字体应解析成功");
    let outline = face.glyph_outline('A').expect("A 应可提取轮廓");
    assert!(!outline.contours.is_empty(), "A 的轮廓不应为空");

    for contour in outline.contours.iter() {
        assert!(contour.is_closed, "字形子轮廓应闭合");
        assert!(!contour.arcs.is_empty(), "子轮廓不应只有空弧序列");
        // 相邻弧首尾相接
        for pair in contour.arcs.windows(2) {
            assert_close(pair[0].p1, pair[1].p0);
        }
        // 末弧终点必须回到首弧起点
        let first = &contour.arcs[0];
        let last = &contour.arcs[contour.arcs.len() - 1];
        assert_close(last.p1, first.p0);
        // 所有端点与曲率参数均为有限值
        for arc in contour.arcs.iter() {
            assert!(arc.p0.x.is_finite() && arc.p0.y.is_finite());
            assert!(arc.p1.x.is_finite() && arc.p1.y.is_finite());
            assert!(arc.d.is_finite());
        }
    }

    let extents = outline.extents();
    assert!(extents.width() > 0.0 && extents.height() > 0.0, "包围盒非退化");
    assert!(
        extents.width() < 10.0 * face.units_per_em(),
        "包围盒不应超出合理范围"
    );
}

// CASE-050：native 与 wasm 共用同一 from_bytes 签名与实现；同一输入多次提取一致，
// 且字符入口与索引入口走同一实现（不存在 _of_wasm 平行方法）（REQ-002.3）。
#[test]
fn case_050_shared_implementation_is_single_path() {
    let bytes = font_or_skip!();
    let face = FontFace::from_bytes(bytes).expect("合法系统字体应解析成功");

    let by_char_a = face.glyph_outline('A').expect("A 应可提取轮廓");
    let by_char_b = face.glyph_outline('A').expect("A 应可提取轮廓");
    assert_eq!(
        outline_signature(&by_char_a),
        outline_signature(&by_char_b),
        "同一实现的重复提取必须完全一致"
    );

    let index = face.glyph_index('A');
    let by_index = face
        .outline_of_glyph_index(index)
        .expect("按索引应可提取轮廓");
    assert_eq!(
        outline_signature(&by_char_a),
        outline_signature(&by_index),
        "字符入口与索引入口必须走同一实现"
    );
}

// CASE-052：字形度量 advance 非负且与轮廓一致（REQ-001.1）。
#[test]
fn case_052_metrics_advance_non_negative_and_consistent() {
    let bytes = font_or_skip!();
    let face = FontFace::from_bytes(bytes).expect("合法系统字体应解析成功");

    let metrics = face.glyph_metrics('A').expect("A 应有字形度量");
    assert!(metrics.advance >= 0.0, "advance 必须非负");
    assert!(metrics.advance > 0.0, "字母 A 的 advance 应为正");

    let outline = face.glyph_outline('A').expect("A 应可提取轮廓");
    let extents = outline.extents();
    assert!((metrics.extents.mins.x - extents.mins.x).abs() <= COORD_TOLERANCE);
    assert!((metrics.extents.mins.y - extents.mins.y).abs() <= COORD_TOLERANCE);
    assert!((metrics.extents.maxs.x - extents.maxs.x).abs() <= COORD_TOLERANCE);
    assert!((metrics.extents.maxs.y - extents.maxs.y).abs() <= COORD_TOLERANCE);
    assert_eq!(metrics.is_clockwise, outline.is_clockwise());

    let again = face.glyph_metrics('A').expect("A 应有字形度量");
    assert_eq!(metrics, again, "同一输入度量必须一致");

    // ascender 应在 descender 之上，且高度为正
    let ascender = face.ascender();
    let descender = face.descender();
    assert!(ascender > 0.0, "ascender 应为正");
    assert!(ascender > descender, "ascender 应高于 descender");
}

// CASE-081：字体中不存在的字符返回 Err(InvalidParam)，不 panic（REQ-004.3）。
#[test]
fn case_081_missing_char_returns_invalid_param() {
    let bytes = font_or_skip!();
    let face = FontFace::from_bytes(bytes).expect("合法系统字体应解析成功");

    let missing = '\u{10FFFF}';
    assert_eq!(face.glyph_index(missing), 0, "非字符不应命中 cmap");
    match face.glyph_metrics(missing) {
        Err(Error::InvalidParam(_)) => {}
        other => panic!("应返回 InvalidParam，实得 {:?}", other),
    }
    assert!(matches!(
        face.glyph_outline(missing),
        Err(Error::InvalidParam(_))
    ));
}
