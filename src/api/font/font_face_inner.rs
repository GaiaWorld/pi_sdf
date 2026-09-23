/**
 * 字体面与字形轮廓提取（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/api/font/font_face.rs
 */

use super::FontFace;
use crate::model::base::error::{Error, Result};
use crate::model::geom::curve::{Arc, Bezier};
use crate::model::geom::primitive::Point;
use crate::model::outline::contour::{Contour, Outline};

use allsorts::binary::read::ReadScope;
use allsorts::cff::CFF;
use allsorts::font::{GlyphTableFlags, MatchingPresentation};
use allsorts::font_data::{DynamicFontTableProvider, FontData};
use allsorts::outline::{OutlineBuilder, OutlineSink};
use allsorts::pathfinder_geometry::line_segment::LineSegment2F;
use allsorts::pathfinder_geometry::vector::Vector2F;
use allsorts::tables::glyf::GlyfTable;
use allsorts::tables::loca::LocaTable;
use allsorts::tables::{FontTableProvider, SfntVersion};
use allsorts::{tag, Font};

/// 点重合判定的极小容差（字体单位）。
const POINT_EPSILON: f32 = 1e-6;
/// 轮廓拟合允许的最大偏差：每 em 的 1/1024（REQ-001.1）。
const FIT_TOLERANCE_PER_EM: f32 = 1.0 / 1024.0;

/**
 * 字体解析状态（实现侧自由演化的不透明状态）。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 */
#[derive(Debug, Clone)]
pub struct FontState {
    /// 是否已成功解析
    pub parsed: bool,
    /// 每 em 的字体单位数；构造时校验并缓存，保证 > 0
    pub units_per_em: f32,
}

/// 解析字体文件；任意表缺失或字节非法时返回 None（不 panic）。
fn parse_font(data: &[u8]) -> Option<Font<DynamicFontTableProvider<'_>>> {
    let scope = ReadScope::new(data);
    let font_file = scope.read::<FontData<'_>>().ok()?;
    let provider = font_file.table_provider(0).ok()?;
    Font::new(provider).ok()
}

/// 把字体轮廓的回调指令累积为若干条弧子轮廓。
struct ArcSink {
    /// 已完成的子轮廓
    contours: Vec<Contour>,
    /// 当前子轮廓的弧序列
    arcs: Vec<Arc>,
    /// 当前笔画位置
    current: Point,
    /// 当前子轮廓起点
    start: Point,
    /// 是否处于有效子轮廓中
    active: bool,
    /// 贝塞尔拟合允许的最大偏差
    tolerance: f32,
}

impl ArcSink {
    /// 以拟合偏差构造空累积器。
    fn new(tolerance: f32) -> Self {
        debug_assert!(tolerance > 0.0 && tolerance.is_finite());
        ArcSink {
            contours: Vec::new(),
            arcs: Vec::new(),
            current: Point::new(0.0, 0.0),
            start: Point::new(0.0, 0.0),
            active: false,
            tolerance,
        }
    }

    /// 两点在极小容差下是否重合。
    fn same(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() <= POINT_EPSILON && (a.y - b.y).abs() <= POINT_EPSILON
    }

    /// 追加一条直线弧（d = 0），忽略零长度段。
    fn push_line(&mut self, to: Point) {
        if !Self::same(self.current, to) {
            self.arcs.push(Arc::new(self.current, to, 0.0));
        }
        self.current = to;
    }

    /// 把一段三次贝塞尔拟合为弧序列并追加。
    fn push_curve(&mut self, c1: Point, c2: Point, to: Point) {
        let bezier = Bezier::new(self.current, c1, c2, to);
        match crate::core::outline::fit::bezier_to_arcs(&bezier, self.tolerance) {
            Ok(mut fitted) => {
                debug_assert!(!fitted.is_empty());
                self.arcs.append(&mut fitted);
            }
            // 退化曲线：回退为一条直线弧，保证端点连续
            Err(_) => self.arcs.push(Arc::new(self.current, to, 0.0)),
        }
        self.current = to;
    }

    /// 结束当前子轮廓（空子轮廓被丢弃）。
    fn finish_contour(&mut self, closed: bool) {
        if self.active && !self.arcs.is_empty() {
            let arcs = std::mem::take(&mut self.arcs);
            self.contours.push(Contour::new(arcs, closed));
        } else {
            self.arcs.clear();
        }
        self.active = false;
    }

    /// 收尾：把未显式 close 的残余子轮廓按闭合处理。
    fn finish(&mut self) {
        if self.active {
            self.finish_contour(true);
        }
    }
}

impl OutlineSink for ArcSink {
    fn move_to(&mut self, to: Vector2F) {
        // 字体轮廓正常都会先 close；此处为未 close 的异常序列兜底
        self.finish_contour(false);
        let p = Point::new(to.x(), to.y());
        self.current = p;
        self.start = p;
        self.active = true;
    }

    fn line_to(&mut self, to: Vector2F) {
        let p = Point::new(to.x(), to.y());
        if self.active {
            self.push_line(p);
        } else {
            self.current = p;
        }
    }

    fn quadratic_curve_to(&mut self, ctrl: Vector2F, to: Vector2F) {
        let p0 = self.current;
        let c = Point::new(ctrl.x(), ctrl.y());
        let p3 = Point::new(to.x(), to.y());
        // 二次贝塞尔升阶为三次贝塞尔，几何完全一致
        let c1 = Point::new(
            p0.x + (c.x - p0.x) * (2.0 / 3.0),
            p0.y + (c.y - p0.y) * (2.0 / 3.0),
        );
        let c2 = Point::new(
            p3.x + (c.x - p3.x) * (2.0 / 3.0),
            p3.y + (c.y - p3.y) * (2.0 / 3.0),
        );
        if self.active {
            self.push_curve(c1, c2, p3);
        } else {
            self.current = p3;
        }
    }

    fn cubic_curve_to(&mut self, ctrl: LineSegment2F, to: Vector2F) {
        let c1 = Point::new(ctrl.from_x(), ctrl.from_y());
        let c2 = Point::new(ctrl.to_x(), ctrl.to_y());
        let p3 = Point::new(to.x(), to.y());
        if self.active {
            self.push_curve(c1, c2, p3);
        } else {
            self.current = p3;
        }
    }

    fn close(&mut self) {
        if self.active {
            let start = self.start;
            if !Self::same(self.current, start) {
                self.push_line(start);
            }
            self.finish_contour(true);
        }
    }
}

/// 从字体字节提取指定字形索引的轮廓子集。
fn build_contours(data: &[u8], glyph_index: u16, tolerance: f32) -> Result<Vec<Contour>> {
    let scope = ReadScope::new(data);
    let font_file = scope
        .read::<FontData<'_>>()
        .map_err(|_| Error::InvalidFont("字体字节非法或损坏"))?;
    let provider = font_file
        .table_provider(0)
        .map_err(|_| Error::InvalidFont("字体无可用的子字体"))?;
    let font = Font::new(provider).map_err(|_| Error::InvalidFont("字体表缺失或损坏"))?;
    let sfnt_version = font.font_table_provider.sfnt_version();
    let mut sink = ArcSink::new(tolerance);

    if font.glyph_table_flags.contains(GlyphTableFlags::GLYF) {
        let head = font
            .head_table()
            .map_err(|_| Error::InvalidFont("head 表解析失败"))?
            .ok_or(Error::InvalidFont("缺少 head 表"))?;
        let num_glyphs = font.maxp_table.num_glyphs;
        let loca_data = font
            .font_table_provider
            .read_table_data(tag::LOCA)
            .map_err(|_| Error::InvalidFont("loca 表解析失败"))?
            .to_vec();
        let glyf_data = font
            .font_table_provider
            .read_table_data(tag::GLYF)
            .map_err(|_| Error::InvalidFont("glyf 表解析失败"))?
            .to_vec();
        let loca = ReadScope::new(&loca_data)
            .read_dep::<LocaTable<'_>>((usize::from(num_glyphs), head.index_to_loc_format))
            .map_err(|_| Error::InvalidFont("loca 表非法"))?;
        let mut glyf = ReadScope::new(&glyf_data)
            .read_dep::<GlyfTable<'_>>(&loca)
            .map_err(|_| Error::InvalidFont("glyf 表非法"))?;
        glyf.visit(glyph_index, &mut sink)
            .map_err(|_| Error::InvalidFont("字形轮廓提取失败"))?;
    } else if font.glyph_table_flags.contains(GlyphTableFlags::CFF) && sfnt_version == tag::OTTO {
        let cff_data = font
            .font_table_provider
            .read_table_data(tag::CFF)
            .map_err(|_| Error::InvalidFont("CFF 表解析失败"))?
            .to_vec();
        let mut cff = ReadScope::new(&cff_data)
            .read::<CFF<'_>>()
            .map_err(|_| Error::InvalidFont("CFF 表非法"))?;
        cff.visit(glyph_index, &mut sink)
            .map_err(|_| Error::InvalidFont("CFF 字形轮廓提取失败"))?;
    } else {
        return Err(Error::InvalidFont("字体不含受支持的轮廓表"));
    }

    sink.finish();
    Ok(sink.contours)
}

/**
 * 由字体字节构造字体面。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 *
 * 参数：data — 字体文件字节
 */
pub fn font_face_from_bytes(data: Vec<u8>) -> Result<FontFace> {
    // ①解析并校验字体：非法或损坏字节一律返回 InvalidFont，绝不 panic / UB
    // 校验在独立作用域内完成，确保对 data 的借用先结束，随后才可移动 data
    let units_per_em = {
        let font = parse_font(&data).ok_or(Error::InvalidFont("字体字节非法或损坏"))?;
        // ②必须可提取轮廓
        if !font.has_glyph_outlines() {
            return Err(Error::InvalidFont("字体不含受支持的轮廓表"));
        }
        // ③读 head 表并校验 unitsPerEm > 0
        let head = font
            .head_table()
            .map_err(|_| Error::InvalidFont("head 表解析失败"))?
            .ok_or(Error::InvalidFont("缺少 head 表"))?;
        if head.units_per_em == 0 {
            return Err(Error::InvalidFont("unitsPerEm 必须为正"));
        }
        head.units_per_em
    };
    // ④保存字节与已校验状态
    Ok(FontFace {
        data,
        state: FontState {
            parsed: true,
            units_per_em: f32::from(units_per_em),
        },
    })
}

/**
 * 查字符的字形索引。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 *
 * 参数：face — 字体面
 * 参数：ch — 目标字符
 */
pub fn font_face_glyph_index(face: &FontFace, ch: char) -> u32 {
    // ①按字符查 cmap ②缺字返回 0 ③返回字形索引
    match parse_font(&face.data) {
        Some(mut font) => u32::from(
            font.lookup_glyph_index(ch, MatchingPresentation::NotRequired, None)
                .0,
        ),
        // 构造时已校验；此处兜底返回 0（.notdef），不 panic
        None => 0,
    }
}

/**
 * 取字符的字形轮廓。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 *
 * 参数：face — 字体面
 * 参数：ch — 目标字符
 */
pub fn font_face_glyph_outline(face: &FontFace, ch: char) -> Result<Outline> {
    // ①求字形索引 ②缺字返回 InvalidParam ③按索引提取轮廓
    let index = font_face_glyph_index(face, ch);
    if index == 0 {
        return Err(Error::InvalidParam("字符无对应字形"));
    }
    font_face_outline_of_glyph_index(face, index)
}

/**
 * 按字形索引取轮廓。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 *
 * 参数：face — 字体面
 * 参数：index — 字形索引
 */
pub fn font_face_outline_of_glyph_index(face: &FontFace, index: u32) -> Result<Outline> {
    debug_assert!(face.state.parsed);
    // ①校验索引存在
    let font = parse_font(&face.data).ok_or(Error::InvalidFont("字体字节非法或损坏"))?;
    if index >= u32::from(font.maxp_table.num_glyphs) {
        return Err(Error::InvalidParam("字形索引不存在"));
    }
    drop(font);
    // ②以每 em 的 1/1024 为拟合偏差提取 ③组轮廓返回
    let tolerance = (face.state.units_per_em * FIT_TOLERANCE_PER_EM).max(POINT_EPSILON);
    let contours = build_contours(&face.data, index as u16, tolerance)?;
    Ok(Outline::new(contours))
}

/**
 * 每 em 的字体单位数。
 *
 * 契约：API-024
 *
 * 约束：
 *   - requires  data 为字体文件字节（ttf / otf / ttc）
 *   - ensures   native 与 wasm 共用同一签名与同一实现；非法或损坏字节返回 Err，不 panic、不 UB
 *   - 错误      InvalidFont —— 字体字节非法或损坏；InvalidParam —— 字形索引不存在
 *
 * 参数：face — 字体面
 */
pub fn font_face_units_per_em(face: &FontFace) -> f32 {
    // ①读 head 表 ②返回 unitsPerEm ③>0（构造时已校验并缓存）
    debug_assert!(face.state.parsed);
    debug_assert!(face.state.units_per_em > 0.0);
    face.state.units_per_em
}

/// 字形水平步进（字体单位，非负）；解析失败时返回 0。
pub(crate) fn font_horizontal_advance(face: &FontFace, index: u32) -> f32 {
    match parse_font(&face.data) {
        Some(mut font) => font
            .horizontal_advance(index as u16)
            .map(|advance| f32::from(advance))
            .unwrap_or(0.0)
            .max(0.0),
        None => 0.0,
    }
}

/// 字体上升部高度（字体单位）；解析失败时返回 0。
pub(crate) fn font_ascender_value(face: &FontFace) -> f32 {
    match parse_font(&face.data) {
        Some(font) => f32::from(font.hhea_table.ascender),
        None => 0.0,
    }
}

/// 字体下降部高度（字体单位，通常为负）；解析失败时返回 0。
pub(crate) fn font_descender_value(face: &FontFace) -> f32 {
    match parse_font(&face.data) {
        Some(font) => f32::from(font.hhea_table.descender),
        None => 0.0,
    }
}
