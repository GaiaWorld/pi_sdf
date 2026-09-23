/**
 * 路径动词、路径、图形元（实现）
 *
 * 设计：design/pi_sdf2/00-index.md
 * 实现：src/model/path/path.rs
 */

use super::path::ShapeKind;
use super::{Path, PathVerb, Shape};
use crate::core::grid::grid::compute_cell_grid;
use crate::core::outline::fit::bezier_to_arcs;
use crate::core::raster::layout::compute_layout;
use crate::core::raster::raster::rasterize;
use crate::model::base::error::{Error, Result};
use crate::model::geom::aabb::Aabb;
use crate::model::geom::curve::{Arc, Bezier};
use crate::model::geom::primitive::Point;
use crate::model::outline::contour::{Contour, Outline};
use crate::model::raster::sdf::{RasterOptions, SdfTexture};

/// 贝塞尔拟合弧的容差：与 pi_sdf 对 SVG 路径使用的 0.01 一致。
const FIT_TOLERANCE: f32 = 0.01;

/// 路径细分尺度：仅决定轮廓包围盒的外扩边距（越大边距越宽）。
const PATH_CELL_SCALE: f32 = 1.0;

/// 椭圆离散为线段的每整圆段数。
const ELLIPSE_SEGMENTS: u32 = 64;

/// 椭圆弧离散的最大角步长（弧度）。
const ARC_STEP: f32 = std::f32::consts::FRAC_PI_8;

/// 90 度圆弧的曲率参数 tan(π/8)。
const QUARTER_ARC_D: f32 = 0.414_213_56;

/// 路径动词所需的坐标点数（每点 2 个分量）。
///
/// 与 pi_sdf compute_outline 的读数一致：椭圆弧读「半径、旋转与标志、终点」共 3 点。
fn verb_point_count(verb: PathVerb) -> usize {
    match verb {
        PathVerb::MoveTo
        | PathVerb::MoveToRelative
        | PathVerb::LineTo
        | PathVerb::LineToRelative
        | PathVerb::HorizontalLineTo
        | PathVerb::HorizontalLineToRelative
        | PathVerb::VerticalLineTo
        | PathVerb::VerticalLineToRelative => 1,
        PathVerb::QuadTo
        | PathVerb::QuadToRelative
        | PathVerb::SmoothQuadTo
        | PathVerb::SmoothQuadToRelative => 2,
        PathVerb::CubicTo
        | PathVerb::CubicToRelative
        | PathVerb::SmoothCubicTo
        | PathVerb::SmoothCubicToRelative => 3,
        PathVerb::EllipticalArcTo | PathVerb::EllipticalArcToRelative => 3,
        PathVerb::Close => 0,
    }
}

/// 从扁平坐标序列读取一个点并前移游标；越界返回 None（不 panic）。
fn take_point(points: &[f32], cursor: &mut usize) -> Option<Point> {
    let x = *points.get(*cursor)?;
    let y = *points.get(*cursor + 1)?;
    *cursor += 2;
    Some(Point::new(x, y))
}

/// 点叠加相对偏移。
fn offset_point(base: Point, delta: Point) -> Point {
    Point::new(base.x + delta.x, base.y + delta.y)
}

/// 二次贝塞尔升阶为等价三次贝塞尔。
fn quad_to_cubic(p0: Point, c: Point, p3: Point) -> Bezier {
    let two_thirds = 2.0 / 3.0;
    Bezier::new(
        p0,
        Point::new(
            p0.x + (c.x - p0.x) * two_thirds,
            p0.y + (c.y - p0.y) * two_thirds,
        ),
        Point::new(
            p3.x + (c.x - p3.x) * two_thirds,
            p3.y + (c.y - p3.y) * two_thirds,
        ),
        p3,
    )
}

/// 路径到轮廓的累加器：维护当前子路径与游标。
struct PathOutlineBuilder {
    arcs: Vec<Arc>,
    contours: Vec<Contour>,
    start: Point,
    current: Point,
    active: bool,
    closed: bool,
}

impl PathOutlineBuilder {
    fn new() -> Self {
        Self {
            arcs: Vec::new(),
            contours: Vec::new(),
            start: Point::new(0.0, 0.0),
            current: Point::new(0.0, 0.0),
            active: false,
            closed: false,
        }
    }

    /// 结束当前子路径；只有累积到弧时才落为一个子轮廓。
    fn flush(&mut self) {
        if self.active && !self.arcs.is_empty() {
            let arcs = std::mem::take(&mut self.arcs);
            self.contours.push(Contour::new(arcs, self.closed));
        } else {
            self.arcs.clear();
        }
        self.active = false;
        self.closed = false;
    }

    fn move_to(&mut self, p: Point) {
        self.flush();
        self.start = p;
        self.current = p;
        self.active = true;
    }

    fn push_arc(&mut self, to: Point, d: f32) {
        if !self.active {
            return;
        }
        if to.x != self.current.x || to.y != self.current.y {
            self.arcs.push(Arc::new(self.current, to, d));
        }
        self.current = to;
    }

    fn curve_to(&mut self, curve: Bezier) -> Result<()> {
        let fitted = bezier_to_arcs(&curve, FIT_TOLERANCE)?;
        if self.active {
            self.arcs.extend(fitted);
        }
        self.current = curve.p3;
        Ok(())
    }

    fn close(&mut self) {
        if self.active {
            self.push_arc(self.start, 0.0);
            self.closed = true;
            self.flush();
        }
    }

    fn finish(mut self) -> Outline {
        self.flush();
        Outline::new(self.contours)
    }
}

/// 按 SVG 端点参数化离散一段椭圆弧为线段弧。
///
/// 与 pi_sdf 用 lyon 转二次贝塞尔再拟合不同：此处直接按角度采样为线段弧（CASE-055 等价
/// 回归推迟，见任务说明）。
fn append_elliptical_arc(
    builder: &mut PathOutlineBuilder,
    to: Point,
    rx: f32,
    ry: f32,
    rotation: f32,
    large_arc: bool,
    sweep: bool,
) {
    let from = builder.current;
    if rx == 0.0 || ry == 0.0 || (from.x == to.x && from.y == to.y) {
        builder.push_arc(to, 0.0);
        return;
    }
    let mut rx = rx.abs();
    let mut ry = ry.abs();
    let sin_phi = rotation.sin();
    let cos_phi = rotation.cos();
    let dx2 = (from.x - to.x) * 0.5;
    let dy2 = (from.y - to.y) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }
    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let num = rx2 * ry2 - rx2 * y1p * y1p - ry2 * x1p * x1p;
    let den = rx2 * y1p * y1p + ry2 * x1p * x1p;
    let coef = if den == 0.0 {
        0.0
    } else {
        let sign = if large_arc != sweep { 1.0 } else { -1.0 };
        sign * (num.max(0.0) / den).sqrt()
    };
    let cxp = coef * (rx * y1p / ry);
    let cyp = coef * (-ry * x1p / rx);
    let cx = cos_phi * cxp - sin_phi * cyp + (from.x + to.x) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (from.y + to.y) * 0.5;
    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;
    let theta1 = uy.atan2(ux);
    let mut delta = (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let tau = std::f32::consts::TAU;
    if !sweep && delta > 0.0 {
        delta -= tau;
    } else if sweep && delta < 0.0 {
        delta += tau;
    }
    let steps = ((delta.abs() / ARC_STEP).ceil() as u32).max(1);
    for i in 1..=steps {
        let t = theta1 + delta * (i as f32 / steps as f32);
        let st = t.sin();
        let ct = t.cos();
        let next = if i == steps {
            to
        } else {
            Point::new(
                cx + cos_phi * rx * ct - sin_phi * ry * st,
                cy + sin_phi * rx * ct + cos_phi * ry * st,
            )
        };
        builder.push_arc(next, 0.0);
    }
}

/// 逐动词把路径解析为轮廓。
fn build_outline(path: &Path) -> Result<Outline> {
    let mut builder = PathOutlineBuilder::new();
    let mut cursor = 0usize;
    for &verb in path.verbs.iter() {
        match verb {
            PathVerb::MoveTo => {
                let p = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.move_to(p);
            }
            PathVerb::MoveToRelative => {
                let d = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.move_to(offset_point(builder.current, d));
            }
            PathVerb::LineTo => {
                let p = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.push_arc(p, 0.0);
            }
            PathVerb::LineToRelative => {
                let d = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.push_arc(offset_point(builder.current, d), 0.0);
            }
            PathVerb::HorizontalLineTo => {
                let p = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = Point::new(p.x, builder.current.y);
                builder.push_arc(to, 0.0);
            }
            PathVerb::HorizontalLineToRelative => {
                let d = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = Point::new(builder.current.x + d.x, builder.current.y);
                builder.push_arc(to, 0.0);
            }
            PathVerb::VerticalLineTo => {
                let p = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = Point::new(builder.current.x, p.y);
                builder.push_arc(to, 0.0);
            }
            PathVerb::VerticalLineToRelative => {
                let d = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = Point::new(builder.current.x, builder.current.y + d.y);
                builder.push_arc(to, 0.0);
            }
            PathVerb::QuadTo | PathVerb::SmoothQuadTo => {
                let c = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.curve_to(quad_to_cubic(builder.current, c, to))?;
            }
            PathVerb::QuadToRelative | PathVerb::SmoothQuadToRelative => {
                let c = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let base = builder.current;
                builder.curve_to(quad_to_cubic(
                    base,
                    offset_point(base, c),
                    offset_point(base, to),
                ))?;
            }
            PathVerb::CubicTo | PathVerb::SmoothCubicTo => {
                let c1 = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let c2 = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                builder.curve_to(Bezier::new(builder.current, c1, c2, to))?;
            }
            PathVerb::CubicToRelative | PathVerb::SmoothCubicToRelative => {
                let c1 = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let c2 = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let base = builder.current;
                builder.curve_to(Bezier::new(
                    base,
                    offset_point(base, c1),
                    offset_point(base, c2),
                    offset_point(base, to),
                ))?;
            }
            PathVerb::EllipticalArcTo | PathVerb::EllipticalArcToRelative => {
                let radii = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let params = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let raw_to = take_point(&path.points, &mut cursor).ok_or(Error::Geometry("路径坐标缺失"))?;
                let to = if verb == PathVerb::EllipticalArcToRelative {
                    offset_point(builder.current, raw_to)
                } else {
                    raw_to
                };
                let flags = params.y as i32;
                let large_arc = flags & 2 != 0;
                let sweep = flags & 1 != 0;
                append_elliptical_arc(
                    &mut builder,
                    to,
                    radii.x,
                    radii.y,
                    params.x,
                    large_arc,
                    sweep,
                );
            }
            PathVerb::Close => builder.close(),
        }
    }
    Ok(builder.finish())
}

/// 路径是否为面积图元：以 Close 收尾，或首尾坐标重合。
fn path_is_area(path: &Path) -> bool {
    if path.verbs.last() == Some(&PathVerb::Close) {
        return true;
    }
    let count = path.points.len() / 2;
    if count < 2 {
        return false;
    }
    let first = Point::new(path.points[0], path.points[1]);
    let last = Point::new(path.points[(count - 1) * 2], path.points[(count - 1) * 2 + 1]);
    first == last
}

/// 丢弃零长度弧后组轮廓；无有效弧时返回空轮廓。
fn outline_from_arcs(arcs: Vec<Arc>, is_closed: bool) -> Outline {
    let arcs: Vec<Arc> = arcs.into_iter().filter(|arc| arc.p0 != arc.p1).collect();
    if arcs.is_empty() {
        Outline::new(Vec::new())
    } else {
        Outline::new(vec![Contour::new(arcs, is_closed)])
    }
}

/// 圆轮廓：四个 90 度圆弧，逆时针。
fn circle_arcs(cx: f32, cy: f32, r: f32) -> Vec<Arc> {
    if r <= 0.0 {
        return Vec::new();
    }
    let left = Point::new(cx - r, cy);
    let bottom = Point::new(cx, cy - r);
    let right = Point::new(cx + r, cy);
    let top = Point::new(cx, cy + r);
    vec![
        Arc::new(left, bottom, QUARTER_ARC_D),
        Arc::new(bottom, right, QUARTER_ARC_D),
        Arc::new(right, top, QUARTER_ARC_D),
        Arc::new(top, left, QUARTER_ARC_D),
    ]
}

/// 矩形轮廓：四条线段，逆时针。
fn rect_arcs(x: f32, y: f32, w: f32, h: f32) -> Vec<Arc> {
    let min_x = x.min(x + w);
    let min_y = y.min(y + h);
    let max_x = x.max(x + w);
    let max_y = y.max(y + h);
    let p0 = Point::new(min_x, min_y);
    let p1 = Point::new(max_x, min_y);
    let p2 = Point::new(max_x, max_y);
    let p3 = Point::new(min_x, max_y);
    vec![
        Arc::new(p0, p1, 0.0),
        Arc::new(p1, p2, 0.0),
        Arc::new(p2, p3, 0.0),
        Arc::new(p3, p0, 0.0),
    ]
}

/// 椭圆轮廓：按角度离散为线段弧，逆时针。
fn ellipse_arcs(cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<Arc> {
    let steps = ELLIPSE_SEGMENTS;
    let tau = std::f32::consts::TAU;
    let mut arcs = Vec::with_capacity(steps as usize);
    for i in 0..steps {
        let a0 = tau * (i as f32) / (steps as f32);
        let a1 = tau * ((i + 1) as f32) / (steps as f32);
        let p0 = Point::new(cx + rx * a0.cos(), cy + ry * a0.sin());
        let p1 = Point::new(cx + rx * a1.cos(), cy + ry * a1.sin());
        arcs.push(Arc::new(p0, p1, 0.0));
    }
    arcs
}

/// 由扁平坐标构造折线弧；closed 为真时补上收尾弧。
fn polyline_arcs(points: &[f32], closed: bool) -> Vec<Arc> {
    let count = points.len() / 2;
    if count == 0 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity(count);
    for pair in points.chunks_exact(2) {
        vertices.push(Point::new(pair[0], pair[1]));
    }
    let mut arcs = Vec::with_capacity(count);
    for i in 0..count.saturating_sub(1) {
        if vertices[i] != vertices[i + 1] {
            arcs.push(Arc::new(vertices[i], vertices[i + 1], 0.0));
        }
    }
    if closed && vertices[count - 1] != vertices[0] {
        arcs.push(Arc::new(vertices[count - 1], vertices[0], 0.0));
    }
    arcs
}

/// 闭合折线是否为顺时针。
fn polygon_is_clockwise(points: &[f32]) -> bool {
    let count = points.len() / 2;
    if count < 3 {
        return false;
    }
    let mut area = 0.0f32;
    for i in 0..count {
        let j = (i + 1) % count;
        area += points[i * 2] * points[j * 2 + 1] - points[j * 2] * points[i * 2 + 1];
    }
    area < 0.0
}

/// 把闭合折线规范为逆时针（内部在行进方向左侧，距离为负）。
fn normalized_ccw(points: &[f32]) -> Vec<f32> {
    if !polygon_is_clockwise(points) {
        return points.to_vec();
    }
    let count = points.len() / 2;
    let mut reversed = Vec::with_capacity(points.len());
    for i in (0..count).rev() {
        reversed.push(points[i * 2]);
        reversed.push(points[i * 2 + 1]);
    }
    reversed
}

/// 图形元转轮廓：按类型校验尺寸并离散。
fn shape_outline(shape: &Shape) -> Result<Outline> {
    match &shape.kind {
        ShapeKind::Circle { cx, cy, r } => {
            if *r < 0.0 {
                return Err(Error::InvalidParam("圆的半径不能为负"));
            }
            Ok(outline_from_arcs(circle_arcs(*cx, *cy, *r), true))
        }
        ShapeKind::Rect { x, y, w, h } => {
            if *w < 0.0 || *h < 0.0 {
                return Err(Error::InvalidParam("矩形的宽高不能为负"));
            }
            Ok(outline_from_arcs(rect_arcs(*x, *y, *w, *h), true))
        }
        ShapeKind::Line { ax, ay, bx, by, .. } => {
            let from = Point::new(*ax, *ay);
            let to = Point::new(*bx, *by);
            let arcs = if from == to {
                Vec::new()
            } else {
                vec![Arc::new(from, to, 0.0)]
            };
            Ok(outline_from_arcs(arcs, false))
        }
        ShapeKind::Ellipse { cx, cy, rx, ry } => {
            if *rx < 0.0 || *ry < 0.0 {
                return Err(Error::InvalidParam("椭圆的半径不能为负"));
            }
            Ok(outline_from_arcs(ellipse_arcs(*cx, *cy, *rx, *ry), true))
        }
        ShapeKind::Polygon { points } => {
            if points.len() % 2 != 0 || points.len() / 2 < 3 {
                return Err(Error::InvalidParam("多边形至少需要 3 个点"));
            }
            let ccw = normalized_ccw(points);
            Ok(outline_from_arcs(polyline_arcs(&ccw, true), true))
        }
        ShapeKind::Polyline { points, is_close } => {
            if points.len() % 2 != 0 || points.len() / 2 < 2 {
                return Err(Error::InvalidParam("折线至少需要 2 个点"));
            }
            let normalized = if *is_close {
                normalized_ccw(points)
            } else {
                points.clone()
            };
            Ok(outline_from_arcs(
                polyline_arcs(&normalized, *is_close),
                *is_close,
            ))
        }
    }
}

/// 图形元包围盒：按类型取参数驱动的最小/最大。
fn shape_bounds(shape: &Shape) -> Aabb {
    let mut bounds = Aabb::new_invalid();
    match &shape.kind {
        ShapeKind::Circle { cx, cy, r } => {
            bounds.extend(Point::new(cx - r, cy - r));
            bounds.extend(Point::new(cx + r, cy + r));
        }
        ShapeKind::Rect { x, y, w, h } => {
            bounds.extend(Point::new(*x, *y));
            bounds.extend(Point::new(x + w, y + h));
        }
        ShapeKind::Line { ax, ay, bx, by, .. } => {
            bounds.extend(Point::new(*ax, *ay));
            bounds.extend(Point::new(*bx, *by));
        }
        ShapeKind::Ellipse { cx, cy, rx, ry } => {
            bounds.extend(Point::new(cx - rx, cy - ry));
            bounds.extend(Point::new(cx + rx, cy + ry));
        }
        ShapeKind::Polygon { points } | ShapeKind::Polyline { points, .. } => {
            for pair in points.chunks_exact(2) {
                bounds.extend(Point::new(pair[0], pair[1]));
            }
        }
    }
    bounds
}

/**
 * 由判别值还原路径动词。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 *
 * 参数：value — 路径动词判别值
 */
pub fn path_verb_try_from(value: u8) -> Result<PathVerb> {
    match value {
        1 => Ok(PathVerb::MoveTo),
        2 => Ok(PathVerb::MoveToRelative),
        3 => Ok(PathVerb::LineTo),
        4 => Ok(PathVerb::LineToRelative),
        5 => Ok(PathVerb::QuadTo),
        6 => Ok(PathVerb::QuadToRelative),
        7 => Ok(PathVerb::SmoothQuadTo),
        8 => Ok(PathVerb::SmoothQuadToRelative),
        9 => Ok(PathVerb::CubicTo),
        10 => Ok(PathVerb::CubicToRelative),
        11 => Ok(PathVerb::SmoothCubicTo),
        12 => Ok(PathVerb::SmoothCubicToRelative),
        13 => Ok(PathVerb::HorizontalLineTo),
        14 => Ok(PathVerb::HorizontalLineToRelative),
        15 => Ok(PathVerb::VerticalLineTo),
        16 => Ok(PathVerb::VerticalLineToRelative),
        17 => Ok(PathVerb::EllipticalArcTo),
        18 => Ok(PathVerb::EllipticalArcToRelative),
        19 => Ok(PathVerb::Close),
        other => Err(Error::InvalidPathVerb(other)),
    }
}

/**
 * 取路径动词判别值。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 *
 * 参数：verb — 路径动词
 */
pub fn path_verb_to_u8(verb: PathVerb) -> u8 {
    verb as u8
}

/**
 * 由动词字节与扁平坐标构造路径。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 *
 * 参数：verbs — 路径动词判别值序列
 *
 * 参数：points — 扁平坐标序列，每点 2 个分量
 */
pub fn path_new(verbs: Vec<u8>, points: Vec<f32>) -> Result<Path> {
    let mut parsed = Vec::with_capacity(verbs.len());
    for value in verbs {
        parsed.push(path_verb_try_from(value)?);
    }
    path_from_verbs(parsed, points)
}

/**
 * 由路径动词与扁平坐标构造路径。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 *
 * 参数：verbs — 路径动词序列
 *
 * 参数：points — 扁平坐标序列，每点 2 个分量
 */
pub fn path_from_verbs(verbs: Vec<PathVerb>, points: Vec<f32>) -> Result<Path> {
    let required: usize = verbs.iter().map(|&verb| verb_point_count(verb) * 2).sum();
    if points.len() != required {
        return Err(Error::InvalidParam("坐标数量与路径动词不匹配"));
    }
    debug_assert_eq!(points.len() % 2, 0);
    Ok(Path { verbs, points })
}

/**
 * 路径转轮廓。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   结果与现状一致
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配
 *
 * 参数：path — 路径
 */
pub fn path_to_outline(path: &Path) -> Result<Outline> {
    build_outline(path)
}

/**
 * 路径生成 SDF 纹理。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  tex_size、pxrange 为正
 *   - ensures   结果与现状等价
 *   - 错误      InvalidParam —— 参数非正
 *
 * 参数：path — 路径
 *
 * 参数：tex_size — 纹理边长，正整数
 *
 * 参数：pxrange — 距离像素范围，正整数
 */
pub fn path_sdf_texture(path: &Path, tex_size: u32, pxrange: u32) -> Result<SdfTexture> {
    if tex_size == 0 {
        return Err(Error::InvalidParam("tex_size 必须为正"));
    }
    if pxrange == 0 {
        return Err(Error::InvalidParam("pxrange 必须为正"));
    }
    let outline = path_to_outline(path)?;
    let raw = outline.extents();
    if raw.is_empty() || (raw.width() <= 0.0 && raw.height() <= 0.0) {
        return Err(Error::InvalidParam("路径退化，无法生成 SDF 纹理"));
    }
    let is_area = path_is_area(path);
    let grid = compute_cell_grid(&outline, PATH_CELL_SCALE, is_area)?;
    let layout = compute_layout(grid.extents, tex_size, pxrange, 1.0, 0, true)?;
    let opts = RasterOptions {
        is_outer_glow: false,
        is_svg: true,
        is_reverse: None,
    };
    rasterize(&grid, &layout, &opts)
}

/**
 * 路径包围盒。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   结果与现状一致
 *   - 错误      无
 *
 * 参数：path — 路径
 */
pub fn path_extents(path: &Path) -> Aabb {
    let mut bounds = Aabb::new_invalid();
    for pair in path.points.chunks_exact(2) {
        bounds.extend(Point::new(pair[0], pair[1]));
    }
    bounds
}

/**
 * 反转路径方向。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   结果原地作用于自身
 *   - 错误      无
 *
 * 参数：path — 路径，原地改写
 */
pub fn path_reverse(path: &mut Path) {
    path.verbs.reverse();
    let mut pairs: Vec<[f32; 2]> = path
        .points
        .chunks_exact(2)
        .map(|pair| [pair[0], pair[1]])
        .collect();
    pairs.reverse();
    path.points = pairs.into_iter().flatten().collect();
}

/**
 * 图形元转轮廓。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 *
 * 参数：shape — 图形元
 */
pub fn shape_to_outline(shape: &Shape) -> Result<Outline> {
    shape_outline(shape)
}

/**
 * 图形元包围盒。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   结果与现状一致
 *   - 错误      无
 *
 * 参数：shape — 图形元
 */
pub fn shape_extents(shape: &Shape) -> Aabb {
    shape_bounds(shape)
}

/**
 * 判定是否为面积图元。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   线段类返回假，其余返回真
 *   - 错误      无
 *
 * 参数：shape — 图形元
 */
pub fn shape_is_area(shape: &Shape) -> bool {
    !matches!(&shape.kind, ShapeKind::Line { .. } | ShapeKind::Polyline { .. })
}

/**
 * 构造圆。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径为非负值
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_circle(cx: f32, cy: f32, r: f32) -> Shape {
    Shape {
        kind: ShapeKind::Circle { cx, cy, r },
    }
}

/**
 * 构造矩形。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  宽高为非负值
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_rect(x: f32, y: f32, w: f32, h: f32) -> Shape {
    Shape {
        kind: ShapeKind::Rect { x, y, w, h },
    }
}

/**
 * 构造线段。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  step 为 None 或正数
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_line(ax: f32, ay: f32, bx: f32, by: f32, step: Option<f32>) -> Shape {
    Shape {
        kind: ShapeKind::Line { ax, ay, bx, by, step },
    }
}

/**
 * 构造椭圆。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径为非负值
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Shape {
    Shape {
        kind: ShapeKind::Ellipse { cx, cy, rx, ry },
    }
}

/**
 * 构造多边形。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  至少 3 点
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_polygon(points: Vec<f32>) -> Shape {
    Shape {
        kind: ShapeKind::Polygon { points },
    }
}

/**
 * 构造折线。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  至少 2 点
 *   - ensures   返回对应图元
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
pub fn shape_polyline(points: Vec<f32>, is_close: bool) -> Shape {
    Shape {
        kind: ShapeKind::Polyline { points, is_close },
    }
}

