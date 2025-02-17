// use ab_glyph_rasterizer::Point;
use allsorts::pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F};
// use erased_serde::serialize_trait_object;
// use image::EncodableLayout;
use kurbo::Shape;
use lyon_geom::{point, vector, Angle, ArcFlags};
// use lyon_geom::{point, vector, Angle, ArcFlags,};
use parry2d::{
    // na::{Matrix, Matrix3},
    shape::Segment as MSegment,
};
use serde::{Deserialize, Serialize};
// use usvg::tiny_skia_path::PathSegment;

use crate::glyphy::blob::{recursion_near_arcs_of_cell, SdfInfo};
use crate::glyphy::geometry::arc::{Arc, ID};
use crate::glyphy::geometry::segment::{PPoint, PSegment};
use crate::glyphy::util::GLYPHY_INFINITY;
use crate::utils::{compute_cell_range, CellInfo, LayoutInfo, OutlineSinkExt, SdfInfo2, TexInfo2};
use crate::Vector2;
use crate::{
    glyphy::geometry::aabb::Aabb,
    utils::{compute_layout, Attribute},
};
use crate::{
    glyphy::{
        geometry::{arc::ArcEndpoint, point::PointExt},
        util::float_equals,
    },
    // svg::encode_uint_arc_impl,
    utils::GlyphVisitor,
    Point,
};
use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    fmt::Debug,
    hash::Hasher,
    mem::transmute,
};
pub const FARWAY: f32 = 20.0;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PathVerb {
    // 绝对点
    MoveTo = 1,
    // 相对点
    MoveToRelative = 2,
    LineTo = 3,
    LineToRelative = 4,
    QuadTo = 5,
    QuadToRelative = 6,
    SmoothQuadTo = 7,
    SmoothQuadToRelative = 8,
    CubicTo = 9,
    CubicToRelative = 10,
    SmoothCubicTo = 11,
    SmoothCubicToRelative = 12,
    HorizontalLineTo = 13,
    HorizontalLineToRelative = 14,
    VerticalLineTo = 15,
    VerticalLineToRelative = 16,
    EllipticalArcTo = 17,
    EllipticalArcToRelative = 18,
    Close = 19,
}

impl Into<f32> for PathVerb {
    fn into(self) -> f32 {
        match self {
            PathVerb::MoveTo => 1.0,
            PathVerb::MoveToRelative => 2.0,
            PathVerb::LineTo => 3.0,
            PathVerb::LineToRelative => 4.0,
            PathVerb::QuadTo => 5.0,
            PathVerb::QuadToRelative => 6.0,
            PathVerb::SmoothQuadTo => 7.0,
            PathVerb::SmoothQuadToRelative => 8.0,
            PathVerb::CubicTo => 9.0,
            PathVerb::CubicToRelative => 10.0,
            PathVerb::SmoothCubicTo => 11.0,
            PathVerb::SmoothCubicToRelative => 12.0,
            PathVerb::HorizontalLineTo => 13.0,
            PathVerb::HorizontalLineToRelative => 14.0,
            PathVerb::VerticalLineTo => 15.0,
            PathVerb::VerticalLineToRelative => 16.0,
            PathVerb::EllipticalArcTo => 17.0,
            PathVerb::EllipticalArcToRelative => 18.0,
            PathVerb::Close => 19.0,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Circle {
    radius: f32,
    cx: f32,
    cy: f32,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Circle {
    pub fn new(cx: f32, cy: f32, radius: f32) -> Result<Circle, String> {
        if radius <= 0.0 {
            return Err("radius < 0 of circle!!!".to_string());
        }
        let mut attribute = Attribute::default();
        attribute.start = Point::new(cx - radius, cy);
        attribute.is_close = true;

        Ok(Self {
            radius,
            cx,
            cy,
            attribute,
        })
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        vec![
            ArcEndpoint::new(self.cx + self.radius, self.cy, f32::INFINITY),
            ArcEndpoint::new(
                self.cx,
                self.cy - self.radius,
                -std::f32::consts::FRAC_PI_8.tan(),
            ),
            ArcEndpoint::new(
                self.cx - self.radius,
                self.cy,
                -std::f32::consts::FRAC_PI_8.tan(),
            ),
            ArcEndpoint::new(
                self.cx,
                self.cy + self.radius,
                -std::f32::consts::FRAC_PI_8.tan(),
            ),
            ArcEndpoint::new(
                self.cx + self.radius,
                self.cy,
                -std::f32::consts::FRAC_PI_8.tan(),
            ),
        ]
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&[self.cx, self.cy, self.radius, 1.0]));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        Aabb::new(
            Point::new(self.cx - self.radius, self.cy - self.radius),
            Point::new(self.cx + self.radius, self.cy + self.radius),
        )
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };
        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        true
    }
}

impl Circle {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut attribute = Attribute::default();
        attribute.start = Point::new(x, y);
        attribute.is_close = true;

        Self {
            x,
            y,
            width,
            height,
            attribute,
        }
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        if self.width * self.height >= 0.0 {
            vec![
                ArcEndpoint::new(self.x, self.y, f32::INFINITY),
                ArcEndpoint::new(self.x, self.y + self.height, 0.0),
                ArcEndpoint::new(self.x + self.width, self.y + self.height, 0.0),
                ArcEndpoint::new(self.x + self.width, self.y, 0.0),
                ArcEndpoint::new(self.x, self.y, 0.0),
            ]
        } else {
            vec![
                ArcEndpoint::new(self.x, self.y, f32::INFINITY),
                ArcEndpoint::new(self.x + self.width, self.y, 0.0),
                ArcEndpoint::new(self.x + self.width, self.y + self.height, 0.0),
                ArcEndpoint::new(self.x, self.y + self.height, 0.0),
                ArcEndpoint::new(self.x, self.y, 0.0),
            ]
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&[
            self.x,
            self.y,
            self.width,
            self.height,
            2.0,
        ]));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let min_x = self.x.min(self.x + self.width);
        let min_y = self.y.min(self.y + self.height);
        let max_x = self.x.max(self.x + self.width);
        let max_y = self.y.max(self.y + self.height);

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };
        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        true
    }
}

impl Rect {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Segment {
    segment: MSegment,
    pub(crate) attribute: Attribute,
    step: Option<Vec<f32>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Segment {
    pub fn new(a_x: f32, a_y: f32, b_x: f32, b_y: f32, step: Option<Vec<f32>>) -> Self {
        let mut attribute = Attribute::default();
        let a = Point::new(a_x, a_y);
        let b = Point::new(b_x, b_y);
        attribute.start = a;
        attribute.is_close = false;
        Self {
            segment: MSegment::new(a, b),
            attribute,
            step,
        }
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        if let Some(step) = &self.step {
            let r = self.get_stroke_dasharray_arc_endpoints([step[0], step[1]]);
            println!("get_arc_endpoints: {:?}", r);
            r
        } else {
            vec![
                ArcEndpoint::new(self.segment.a.x, self.segment.a.y, f32::INFINITY),
                ArcEndpoint::new(self.segment.b.x, self.segment.b.y, 0.0),
            ]
        }
    }

    fn get_stroke_dasharray_arc_endpoints(&self, step: [f32; 2]) -> Vec<ArcEndpoint> {
        let length = self.segment.length();
        let part = step[0] + step[1];
        let num = length / part;
        let mmod = num - num.trunc();
        let dir = (self.segment.b - self.segment.a).normalize();
        println!(
            "get_stroke_dasharray_arc_endpoints {:?}",
            (length, part, num, mmod)
        );
        let real = dir * step[0];
        let a_virtual = dir * step[1];

        let mut arcs = vec![];

        let mut start = ArcEndpoint::new(self.segment.a.x, self.segment.a.y, f32::INFINITY);
        for _ in 0..num as usize {
            let p0 = start.clone();
            let x = p0.p[0] + real[0];
            let y = p0.p[1] + real[1];
            let p1 = ArcEndpoint::new(x, y, 0.0);

            arcs.push(p0.clone());
            arcs.push(p1.clone());

            let x = p1.p[0] + a_virtual[0];
            let y = p1.p[1] + a_virtual[1];
            start = ArcEndpoint::new(x, y, f32::INFINITY);
        }

        if mmod > 0.01 {
            let r = mmod / (step[0] / part);
            let p = if r > 1.0 {
                let x = start.p[0] + real[0];
                let y = start.p[1] + real[1];
                ArcEndpoint::new(x, y, 0.0)
            } else {
                let x = start.p[0] + real[0] * r;
                let y = start.p[1] + real[1] * r;
                ArcEndpoint::new(x, y, 0.0)
            };
            arcs.push(start);
            arcs.push(p);
        }

        arcs
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();

        hasher.write(bytemuck::cast_slice(&[
            self.segment.a.x,
            self.segment.a.y,
            self.segment.b.x,
            self.segment.b.y,
            3.0,
        ]));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let min_x = self.segment.a.x.min(self.segment.b.x);
        let min_y = self.segment.a.y.min(self.segment.b.y);
        let max_x = self.segment.a.x.max(self.segment.b.x);
        let max_y = self.segment.a.y.max(self.segment.b.y);

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };
        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        false
    }
}

impl Segment {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Ellipse {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Ellipse {
    pub fn new(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        let mut attribute = Attribute::default();
        attribute.start = Point::new(cx - rx, cy);
        attribute.is_close = true;
        Self {
            cx,
            cy,
            rx,
            ry,
            attribute,
        }
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let center = kurbo::Point::new(self.cx as f64, self.cy as f64);
        let e = kurbo::Ellipse::new(center, (self.rx as f64, self.ry as f64), 0.0);
        // let e = kurbo::SvgArc::(center, (self.rx as f64, self.ry as f64), 0.0);

        let path = e.into_path(0.1);

        let mut verbs = Vec::with_capacity(path.elements().len());
        let mut points = Vec::with_capacity(path.elements().len() * 2);
        for p in path {
            match p {
                kurbo::PathEl::MoveTo(to) => {
                    verbs.push(PathVerb::MoveTo);
                    points.push(Point::new(to.x as f32, to.y as f32));
                }
                kurbo::PathEl::LineTo(to) => {
                    verbs.push(PathVerb::LineTo);
                    points.push(Point::new(to.x as f32, to.y as f32));
                }
                kurbo::PathEl::QuadTo(c, to) => {
                    verbs.push(PathVerb::QuadTo);
                    points.push(Point::new(c.x as f32, c.y as f32));
                    points.push(Point::new(to.x as f32, to.y as f32));
                }
                kurbo::PathEl::CurveTo(c1, c2, to) => {
                    verbs.push(PathVerb::CubicTo);
                    points.push(Point::new(c1.x as f32, c1.y as f32));
                    points.push(Point::new(c2.x as f32, c2.y as f32));
                    points.push(Point::new(to.x as f32, to.y as f32));
                }
                kurbo::PathEl::ClosePath => {
                    verbs.push(PathVerb::Close);
                }
            }
        }
        let mut sink = GlyphVisitor::new(1.0);
        // 圆弧拟合贝塞尔曲线的精度，值越小越精确
        sink.accumulate.tolerance = 0.1;
        // log::debug!("=====e.area():{}", e.area());
        if e.area() > 0.0 {
            let temp = verbs[0];
            let len = verbs.len();
            verbs[0] = verbs[len - 1];
            verbs[len - 1] = temp;
            compute_outline(points.iter().rev(), verbs.iter().rev(), &mut sink, false)
        } else {
            compute_outline(points.iter(), verbs.iter(), &mut sink, false)
        }

        sink.accumulate.result
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&[
            self.rx / self.ry, 4.0,
        ]));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        Aabb::new(
            Point::new(self.cx - self.rx, self.cy - self.ry),
            Point::new(self.cx + self.rx, self.cy + self.ry),
        )
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };

        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        true
    }
}

impl Ellipse {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Polygon {
    points: Vec<Point>,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Polygon {
    pub fn new(points: Vec<f32>) -> Self {
        let mut points = points
            .chunks(2)
            .map(|v| Point::new(v[0], v[1]))
            .collect::<Vec<Point>>();
        if !compute_direction(&points) {
            points.reverse();
        };
        let mut attribute = Attribute::default();
        attribute.is_close = true;
        attribute.start = points[0];

        Self { points, attribute }
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let len = self.points.len();
        let mut points = self.points.iter();

        let mut result = Vec::with_capacity(len + 1);
        let start = points.next().unwrap();

        result.push(ArcEndpoint::new(start.x, start.y, f32::INFINITY));

        for p in points {
            result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        }

        let end = result.last().unwrap();
        if !float_equals(end.p[0], start.x, None) || !float_equals(end.p[1], start.y, None) {
            result.push(ArcEndpoint::new(start.x, start.y, 0.0));
        }

        result
    }

    pub fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 2);
        for p in &self.points {
            key.push(p.x);
            key.push(p.y);
        }
        key.push(5.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&key));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        self.points.iter().for_each(|v| {
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        });

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };

        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        true
    }
}

impl Polygon {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Polyline {
    points: Vec<Point>,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Polyline {
    pub fn new(points: Vec<f32>) -> Self {
        let mut points = points
            .chunks(2)
            .map(|v| Point::new(v[0], v[1]))
            .collect::<Vec<Point>>();

        if !compute_direction(&points) {
            points.reverse();
        };

        let mut attribute = Attribute::default();
        attribute.start = points[0];

        let mut r = Self { points, attribute };
        r.attribute.is_close = r.is_close();

        r
    }

    pub fn is_close(&self) -> bool {
        let r = self.points.first().map(|v1| {
            self.points
                .last()
                .map(|v2| (*v1 - *v2).norm_squared() < 0.01)
        });

        if let Some(Some(r)) = r {
            return r;
        }

        false
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let is_close = self.attribute.is_close;

        let mut points = self.points.iter();
        let mut result = Vec::with_capacity(points.len() + 1);

        let start = points.next().unwrap();
        result.push(ArcEndpoint::new(start.x, start.y, f32::INFINITY));

        for p in points {
            result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        }

        // if !is_close {
        //     let mut points = self.points.iter().rev();
        //     let _ = points.next();
        //     for p in points {
        //         result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        //     }
        // }

        result
    }

    pub fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 2);
        for p in &self.points {
            key.push(p.x);
            key.push(p.y);
        }
        key.push(6.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&key));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        self.points.iter().for_each(|v| {
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        });

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
        .max(binding_box.maxs.y - binding_box.mins.y)
        .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };

        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints: self.get_arc_endpoints(),
            is_area: self.is_area(),
            is_reverse: None,
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        false
    }
}

impl Polyline {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Path {
    verbs: Vec<PathVerb>,
    points: Vec<Point>,
    pub(crate) attribute: Attribute,
    is_reverse: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Path {
    pub fn new(verbs: Vec<u8>, points: Vec<f32>) -> Self {
        // let _ = console_log::init_with_level(log::Level::Warn);
        let verbs: Vec<PathVerb> = unsafe { transmute(verbs) };

        let points = points
            .chunks(2)
            .map(|v| Point::new(v[0], v[1]))
            .collect::<Vec<Point>>();
        let is_reverse = false;
        // if points.len() > 2 && !compute_direction(&points) {
        //     points.reverse();
        //     verbs.reverse();
        //     is_reverse = true;

        //     let temp = verbs[0];
        //     let len = verbs.len();
        //     verbs[0] = verbs[len - 1];
        //     verbs[len - 1] = temp;
        // };

        // log::debug!("{:?}", (&points, &verbs));

        let mut attribute = Attribute::default();
        attribute.start = points[0];

        let mut r = Self {
            verbs,
            points,
            attribute,
            is_reverse,
        };
        r.attribute.is_close = r.is_close();

        if r.attribute.is_close {
            r.is_reverse = compute_direction(&r.points);
        }
        r
    }

    pub fn new1(verbs: Vec<PathVerb>, points: Vec<f32>) -> Self {
        // let mut verbs: Vec<PathVerb> = unsafe { transmute(verbs) };

        let points = points
            .chunks(2)
            .map(|v| Point::new(v[0], v[1]))
            .collect::<Vec<Point>>();
        let is_reverse = false;
        // if verbs.
        // if points.len() > 2 && !compute_direction(&points) {
        //     points.reverse();
        //     // verbs.reverse();
        //     is_reverse = true;

        //     let temp = verbs[0];
        //     let len = verbs.len();
        //     // verbs[0] = verbs[len - 1];
        //     // verbs[len - 1] = temp;
        // };

        // log::debug!("{:?}", (&points, &verbs));

        let mut attribute = Attribute::default();
        attribute.start = points[0];

        let mut r = Self {
            verbs,
            points,
            attribute,
            is_reverse,
        };
        r.attribute.is_close = r.is_close();

        if r.attribute.is_close {
            r.is_reverse = compute_direction(&r.points);
        }

        log::debug!("attribute.is_close: {:?}", r.attribute.is_close);
        r
    }

    fn is_close(&self) -> bool {
        if self.verbs.last() == Some(&PathVerb::Close) {
            return true;
        }

        let r = self.points.first().map(|v1| {
            self.points
                .last()
                .map(|v2| (*v1 - *v2).norm_squared() < 0.01)
        });

        if let Some(Some(r)) = r {
            return r;
        }

        false
    }

    fn get_arc_endpoints(&self) -> (Vec<ArcEndpoint>, Aabb, usize) {
        let mut sink = GlyphVisitor::new(1.0);
        // 圆弧拟合贝塞尔曲线的精度，值越小越精确
        sink.accumulate.tolerance = 0.01;

        // let is_close = self.is_close();
        compute_outline(
            self.points.iter(),
            self.verbs.iter(),
            &mut sink,
            self.is_reverse,
        );
        // if !is_close {
        //     compute_outline(
        //         self.points[0..self.points.len() - 1].iter().rev(),
        //         self.verbs[1..self.verbs.len()].iter().rev(),
        //         &mut sink,
        //         self.is_reverse,
        //     );
        // }
        let GlyphVisitor {
            accumulate,
            bbox,
            arcs,
            ..
        } = sink;
        (accumulate.result, bbox, arcs)
    }

    pub fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 4);
        for (p, v) in self.points.iter().zip(self.verbs.iter()) {
            key.push(p.x);
            key.push(p.y);
            key.push((*v).into());
        }
        key.push(7.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&key));
        hasher.finish()
    }

    fn _binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in &self.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let (arc_endpoints, binding_box, arcs) = self.get_arc_endpoints();
        let size = (binding_box.maxs.x - binding_box.mins.x)
                .max(binding_box.maxs.y - binding_box.mins.y)
                .ceil();
        let tex_size = if size < 64.0 { 32.0 } else { size * 0.5 };

        SvgInfo {
            binding_box: vec![
                binding_box.mins.x,
                binding_box.mins.y,
                binding_box.maxs.x,
                binding_box.maxs.y,
            ],
            arc_endpoints,
            is_area: self.is_area(),
            is_reverse: if arcs == 1 {
                Some(!self.is_reverse)
            } else {
                None
            },
            hash: self.get_hash(),
            tex_size
        }
    }

    pub fn get_svg_info_of_wasm(&self) -> WasmSvgInfo {
        let info = self.get_svg_info();
        WasmSvgInfo {
            buf: bitcode::serialize(&info).unwrap(),
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn is_area(&self) -> bool {
        self.is_close()
    }
}

impl Path {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmSvgInfo {
    pub buf: Vec<u8>,
    pub binding_box: Vec<f32>,
    pub is_area: bool,
    pub hash: String,
    pub tex_size: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SvgInfo {
    pub binding_box: Vec<f32>,
    arc_endpoints: Vec<ArcEndpoint>,
    pub is_area: bool,
    is_reverse: Option<bool>,
    pub hash: u64,
    pub tex_size: f32,
}

impl SvgInfo {
    pub fn new(
        binding_box: &[f32],
        arc_endpoints: Vec<f32>,
        is_area: bool,
        is_reverse: Option<bool>,
    ) -> SvgInfo {
        assert_eq!(arc_endpoints.len() % 3, 0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&arc_endpoints));
        let hash = hasher.finish();
        let mut arc_endpoints2 = Vec::with_capacity(arc_endpoints.len() / 3);
        arc_endpoints
            .chunks(3)
            .for_each(|v| arc_endpoints2.push(ArcEndpoint::new(v[0], v[1], v[2])));
        SvgInfo {
            binding_box: binding_box.to_vec(),
            arc_endpoints: arc_endpoints2,
            is_area,
            is_reverse,
            hash,
            tex_size: 0.0
        }
    }

    pub fn compute_layout(&self, tex_size: usize, pxrange: u32, cur_off: u32) -> LayoutInfo {
        compute_layout(&self.binding_box, tex_size, pxrange, 1, cur_off, true)
    }

    pub fn compute_near_arcs(&self, scale: f32) -> CellInfo {
        let mut info = compute_near_arcs(
            Aabb::new(
                Point::new(self.binding_box[0], self.binding_box[1]),
                Point::new(self.binding_box[2], self.binding_box[3]),
            ),
            &self.arc_endpoints,
            scale,
        );
        info.is_area = self.is_area;
        info
    }

    pub fn compute_sdf_tex(
        &self,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
        scale: f32,
    ) -> SdfInfo2 {
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = self.compute_layout(tex_size, pxrange, cur_off);

        let CellInfo { arcs, info, .. } = compute_near_arcs(
            Aabb::new(
                Point::new(self.binding_box[0], self.binding_box[1]),
                Point::new(self.binding_box[2], self.binding_box[3]),
            ),
            &self.arc_endpoints,
            scale,
        );
        // println!("============1111111111111111 : {:?}",(&arcs, &info));
        let pixmap = crate::utils::encode_sdf(
            &arcs,
            info,
            &Aabb::new(
                Point::new(extents[0], extents[1]),
                Point::new(extents[2], extents[3]),
            ),
            tex_size as usize,
            distance,
            if self.is_area { None } else { Some(1.0) },
            is_outer_glow,
            true,
            self.is_reverse,
        );

        SdfInfo2 {
            sdf_tex: pixmap,
            tex_size: tex_size,
            tex_info: TexInfo2 {
                sdf_offset_x: 0,
                sdf_offset_y: 0,
                advance: self.binding_box[2] - self.binding_box[0],
                plane_min_x: plane_bounds[0],
                plane_min_y: plane_bounds[1],
                plane_max_x: plane_bounds[2],
                plane_max_y: plane_bounds[3],
                atlas_min_x: atlas_bounds[0],
                atlas_min_y: atlas_bounds[1],
                atlas_max_x: atlas_bounds[2],
                atlas_max_y: atlas_bounds[3],
                char: ' ',
            },
        }
    }

    pub fn compute_sdf_cell(&self, scale: f32) -> SdfInfo {
        let cell = self.compute_near_arcs(scale);
        let blob = cell.encode_blob_arc();
        blob.encode_tex()
    }

    pub fn compute_positions_and_uv(&self, ps: &[f32], uv: &[f32], half_extend: f32, out_ps: &mut Vec<f32>, out_uv: &mut Vec<f32>, out_indices: &mut Vec<u16>){
        let mut p0 = Point::new(0., 0.);
        let ps_w = ps[2] - ps[0];
        let ps_h = ps[3] - ps[1];
        let uv_w = uv[2] - uv[0];
        let uv_h = uv[3] - uv[1];

        if self.is_area {
            return ;
        }
        println!("========= self:{:?}", self.arc_endpoints);
        for i in 0..self.arc_endpoints.len() {
            let endpoint = &self.arc_endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = Point::new(endpoint.p[0], endpoint.p[1]);
                continue;
            }
            let half_extend_2 = half_extend * 2.0;
            let p1 = Point::new(endpoint.p[0], endpoint.p[1]);
            let obb = if float_equals(endpoint.d, 0.0, None){
                calculate_obb(p0, p1, half_extend_2)
            } else {
                let arc = Arc::new(p0, p1, endpoint.d);
                [
                    Point::new(arc.aabb.mins.x - half_extend_2, arc.aabb.mins.y - half_extend_2),
                    Point::new(arc.aabb.mins.x - half_extend_2, arc.aabb.maxs.y + half_extend_2),
                    Point::new(arc.aabb.maxs.x + half_extend_2, arc.aabb.maxs.y + half_extend_2),
                    Point::new(arc.aabb.maxs.x + half_extend_2, arc.aabb.mins.y - half_extend_2),
                ]
            };
            
            
            let start = (out_ps.len() / 2) as u16; 

            let p0_x = obb[0].x;
            let uv0_x = (p0_x - ps[0]) / ps_w * uv_w + uv[0];
            out_ps.push(p0_x);
            out_uv.push(uv0_x);

            let p0_y = obb[0].y;
            let uv0_y = (p0_y - ps[1]) / ps_h * uv_h + uv[1];
            out_ps.push(p0_y);
            out_uv.push(uv0_y);

            let p1_x = obb[1].x;
            let uv1_x = (p1_x - ps[0]) / ps_w * uv_w + uv[0];
            out_ps.push(p1_x);
            out_uv.push(uv1_x);

            let p1_y =  obb[1].y;
            let uv1_y = (p1_y - ps[1]) / ps_h * uv_h + uv[1];
            out_ps.push(p1_y);
            out_uv.push(uv1_y);

            let p2_x = obb[2].x;
            let uv2_x = (p2_x - ps[0]) / ps_w * uv_w + uv[0];
            out_ps.push(p2_x);
            out_uv.push(uv2_x);

            let p2_y= obb[2].y;
            let uv2_y = (p2_y - ps[1]) / ps_h * uv_h + uv[1];
            out_ps.push(p2_y);
            out_uv.push(uv2_y);

            let p3_x = obb[3].x;
            let uv3_x = (p3_x - ps[0]) / ps_w * uv_w + uv[0];
            out_ps.push(p3_x);
            out_uv.push(uv3_x);

            let p3_y =  obb[3].y;
            let uv3_y = (p3_y - ps[1]) / ps_h * uv_h + uv[1];
            out_ps.push(p3_y);
            out_uv.push(uv3_y);
            if (p0_x - 0.0).abs() < 0.001{
                println!("======== p0:{:?}, p1:{:?}, obb: {:?}", p0, p1, obb);
            }
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            out_indices.push(start);
            out_indices.push(start + 2);
            out_indices.push(start + 1);
            out_indices.push(start);
            out_indices.push(start + 3);
            out_indices.push(start + 2);
        }
    }

}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl SvgInfo {
    pub fn new_of_wasm(
        binding_box: &[f32],
        arc_endpoints: Vec<f32>,
        is_area: bool,
        is_reverse: Option<bool>,
    ) -> WasmSvgInfo {
        let info = Self::new(binding_box, arc_endpoints, is_area, is_reverse);
        let buf = bitcode::serialize(&info).unwrap();
        WasmSvgInfo {
            buf,
            binding_box: info.binding_box,
            is_area: info.is_area,
            hash: info.hash.to_string(),
            tex_size: info.tex_size,
        }
    }

    pub fn compute_layout_of_wasm(
        binding_box: &[f32],
        tex_size: usize,
        pxrange: u32,
        cur_off: u32,
    ) -> Vec<f32> {
        let LayoutInfo {
            mut plane_bounds,
            mut atlas_bounds,
            mut extents,
            distance,
            tex_size,
        } = compute_layout(binding_box, tex_size, pxrange, 1, cur_off, true);
        let mut res = Vec::with_capacity(14);
        res.append(&mut plane_bounds);
        res.append(&mut atlas_bounds);
        res.append(&mut extents);
        res.push(distance);
        res.push(tex_size as f32);
        res
    }

    pub fn compute_near_arcs_of_wasm(info: &[u8], scale: f32) -> Vec<u8> {
        let info: SvgInfo = bitcode::deserialize(info).unwrap();
        bitcode::serialize(&info.compute_near_arcs(scale)).unwrap()
    }

    pub fn compute_sdf_tex_of_wasm(
        info: &[u8],
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
        scale: f32,
    ) -> Vec<u8> {
        let info: SvgInfo = bitcode::deserialize(info).unwrap();
        bitcode::serialize(&info.compute_sdf_tex(tex_size, pxrange, is_outer_glow, cur_off, scale))
            .unwrap()
    }

    pub fn compute_sdf_cell_of_wasm(info: &[u8], scale: f32) -> Vec<u8> {
        let info: SvgInfo = bitcode::deserialize(info).unwrap();
        let cell = info.compute_near_arcs(scale);
        let blob = cell.encode_blob_arc();
        bitcode::serialize(&blob.encode_tex()).unwrap()
    }

    pub fn compute_positions_and_uv_of_wasm(info: &[u8],  ps: &[f32], uv: &[f32], half_extend: f32, ) -> PosInfo{
        let info: SvgInfo = bitcode::deserialize(info).unwrap();
        let mut info2 = PosInfo::default();
        info.compute_positions_and_uv(ps, uv, half_extend, &mut info2.out_ps, &mut info2.out_uv, &mut info2.out_indices);
        info2
    }
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Default)]
pub struct PosInfo{
    pub out_ps: Vec<f32>, 
    pub out_uv: Vec<f32>, 
    pub out_indices: Vec<u16>
}
pub struct SvgScenes {
    shapes: HashMap<u64, (SvgInfo, Attribute)>,
    view_box: Aabb,
}

impl SvgScenes {
    pub fn new(view_box: Aabb) -> Self {
        let view_box = Aabb::new(
            Point::new(view_box.mins.x, view_box.mins.y),
            Point::new(view_box.maxs.x, view_box.maxs.y),
        );
        Self {
            shapes: Default::default(),
            view_box,
        }
    }

    pub fn add_shape(&mut self, hash: u64, info: SvgInfo, attr: Attribute) {
        // let key = shape.get_hash();
        self.shapes.insert(hash, (info, attr));
    }

    pub fn has_shape(&self, hash: u64) -> bool {
        self.shapes.get(&hash).is_some()
    }

    pub fn set_view_box(&mut self, mins_x: f32, mins_y: f32, maxs_x: f32, maxs_y: f32) {
        self.view_box = Aabb::new(Point::new(mins_x, mins_y), Point::new(maxs_x, maxs_y));
    }
}

fn compute_direction(path: &Vec<Point>) -> bool {
    let mut max_x = -f32::INFINITY;
    let mut index = 0;

    for i in 0..path.len() {
        if path[i].x > max_x {
            max_x = path[i].x;
            index = i;
        }
    }

    let mut previous = path.len() - 1;
    if index != 0 {
        previous = index - 1;
    }

    let mut next = index + 1;
    if next >= path.len() {
        next = 0;
    }

    let a = path[index] - path[previous];
    let b = path[next] - path[index];
    let v =
        parry2d::na::Vector3::new(a.x, a.y, 0.0).cross(&parry2d::na::Vector3::new(b.x, b.y, 0.0));

    // 小于0时，为顺时针
    let clockwise = v.z < 0.0;
    clockwise
}

fn compute_outline<'a>(
    mut points: impl Iterator<Item = &'a Point>,
    verbs: impl Iterator<Item = &'a PathVerb>,
    sink: &mut impl OutlineSinkExt,
    _is_reverse: bool,
) {
    // log::debug!("p: {:?}", points);
    let mut prev_to = Vector2F::default();
    for path_verb in verbs {
        match path_verb {
            PathVerb::MoveTo => {
                let to = points.next().unwrap().into_vec2f();
                sink.move_to(to);
                prev_to = to;
            }
            PathVerb::MoveToRelative => {
                let to = points.next().unwrap().into_vec2f() + prev_to;
                sink.move_to(to);
                prev_to = to;
            }
            PathVerb::LineTo => {
                let to = points.next().unwrap().into_vec2f();
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::LineToRelative => {
                let to = points.next().unwrap().into_vec2f() + prev_to;
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::QuadTo | PathVerb::SmoothQuadTo => {
                let c = points.next().unwrap().into_vec2f();
                let to = points.next().unwrap().into_vec2f();
                sink.quadratic_curve_to(c, to);
                prev_to = to;
            }
            PathVerb::QuadToRelative | PathVerb::SmoothQuadToRelative => {
                let c = points.next().unwrap().into_vec2f() + prev_to;
                let to = points.next().unwrap().into_vec2f() + prev_to;
                sink.quadratic_curve_to(c, to);
                prev_to = to;
            }
            PathVerb::CubicTo | PathVerb::SmoothCubicTo => {
                let c1 = points.next().unwrap().into_vec2f();
                let c2 = points.next().unwrap().into_vec2f();
                let to = points.next().unwrap().into_vec2f();
                sink.cubic_curve_to(LineSegment2F::new(c1, c2), to);
                prev_to = to;
            }
            PathVerb::CubicToRelative | PathVerb::SmoothCubicToRelative => {
                let c1 = points.next().unwrap().into_vec2f() + prev_to;
                let c2 = points.next().unwrap().into_vec2f() + prev_to;
                let to = points.next().unwrap().into_vec2f() + prev_to;
                sink.cubic_curve_to(LineSegment2F::new(c1, c2), to);
                prev_to = to;
            }
            PathVerb::HorizontalLineTo => {
                let to = Vector2F::new(points.next().unwrap().x, prev_to.y());
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::HorizontalLineToRelative => {
                let to = Vector2F::new(points.next().unwrap().x + prev_to.x(), prev_to.y());
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::VerticalLineTo => {
                let to = Vector2F::new(prev_to.x(), points.next().unwrap().y);
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::VerticalLineToRelative => {
                let to = Vector2F::new(prev_to.x(), points.next().unwrap().y + prev_to.y());
                sink.line_to(to);
                prev_to = to;
            }
            PathVerb::EllipticalArcTo | PathVerb::EllipticalArcToRelative => {
                let radii = points.next().unwrap();

                let p = points.next().unwrap();

                let mut to = *points.next().unwrap();
                if let PathVerb::EllipticalArcToRelative = path_verb {
                    to = Point::new(to.x + prev_to.x(), to.y + prev_to.y());
                }
                // large_arc 决定弧线是大于还是小于 180 度，0 表示小角度弧，1 表示大角度弧。
                // sweep 表示弧线的方向，0 表示从起点到终点沿逆时针画弧，1 表示从起点到终点沿顺时针画弧。
                let (large_arc, sweep) = to_arc_flags(p.y);
                if float_equals(radii.x, radii.y, None) {
                    let d = ((to.x - prev_to.x()).powi(2) + (to.y - prev_to.y()).powi(2)).sqrt();
                    let mut theta = 2.0 * (d / (2.0 * radii.x)).asin();

                    if large_arc != (theta > PI) {
                        theta = TAU - theta;
                    }

                    if !sweep {
                        theta = -theta;
                    }
                    sink.arc2_to((theta * 0.25).tan(), Vector2F::new(to.x, to.y));
                } else {
                    let arc = lyon_geom::SvgArc {
                        from: point(prev_to.x(), prev_to.y()),
                        to: point(to.x, to.y),
                        radii: vector(radii.x, radii.y),
                        x_rotation: Angle::radians(p.x),
                        flags: ArcFlags { large_arc, sweep },
                    };

                    arc.for_each_quadratic_bezier(&mut |s| {
                        sink.quadratic_curve_to(
                            Vector2F::new(s.ctrl.x as f32, s.ctrl.y as f32),
                            Vector2F::new(s.to.x as f32, s.to.y as f32),
                        )
                    });
                }

                prev_to = Vector2F::new(to.x as f32, to.y as f32);
            }
            PathVerb::Close => {
                sink.close();
            }
        }
    }
    // is_close
}

fn to_arc_flags(flag: f32) -> (bool, bool) {
    log::debug!("flag: {}", flag);
    match flag as u32 {
        0 => (false, false),
        1 => (false, true),
        2 => (true, false),
        3 => (true, true),
        _ => panic!(),
    }
}

pub fn extents(mut binding_box: Aabb) -> Aabb {
    binding_box.mins.x = binding_box.mins.x - FARWAY;
    binding_box.mins.y = binding_box.mins.y - FARWAY;
    binding_box.maxs.x = binding_box.maxs.x + FARWAY;
    binding_box.maxs.y = binding_box.maxs.y + FARWAY;

    let width = binding_box.mins.x - binding_box.maxs.x;
    let height = binding_box.mins.y - binding_box.maxs.y;
    if width > height {
        binding_box.maxs.y = binding_box.mins.y + width;
    } else {
        binding_box.maxs.x = binding_box.mins.x + height;
    };
    binding_box
}

pub fn compute_near_arcs(view_box: Aabb, endpoints: &Vec<ArcEndpoint>, scale: f32) -> CellInfo {
    let extents = compute_cell_range(view_box, scale);
    // log::debug!("extents: {:?}", extents);
    // let extents = compute_cell_range(extents, scale);
    let mut min_width = f32::INFINITY;
    let mut min_height = f32::INFINITY;

    // let startid = ID.load(std::sync::atomic::Ordering::SeqCst);
    let mut p0 = Point::new(0., 0.);
    // log::debug!("extents2: {:?}", extents);
    let mut near_arcs = Vec::with_capacity(endpoints.len());
    let mut arcs = Vec::with_capacity(endpoints.len());
    // log::debug!("endpoints: {:?}", endpoints);
    let mut id = 0;
    for i in 0..endpoints.len() {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        let mut arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        arc.id = id;
        id += 1;
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);

        near_arcs.push(arc);
        arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
    }
    // let mut tempsegment = parry2d::shape::Segment::new(Point::new(0., 0.), Point::new(0., 0.));
    let mut tempsegment = PSegment::new(PPoint::new(0., 0.), PPoint::new(0., 0.));
    let mut result_arcs = vec![];
    let mut temp = Vec::with_capacity(arcs.len());
    let mut tempidxs = vec![];
    // log::debug!("arcs:{:?}", arcs.len());
    recursion_near_arcs_of_cell(
        // &near_arcs,
        &extents,
        &extents,
        &arcs,
        &mut min_width,
        &mut min_height,
        None,
        None,
        None,
        None,
        &mut result_arcs,
        &mut temp,
        &mut tempsegment,
        // id,
        &mut tempidxs,
    );

    CellInfo {
        extents,
        arcs: near_arcs,
        info: result_arcs,
        min_width,
        min_height,
        is_area: true,
    }
}

// 计算线段的 OBB
fn calculate_obb(p1: Point, p2: Point, width: f32) -> [Point; 4] {
    // 计算线段中心点
    let center = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);

    // 计算线段方向向量
    let direction = p2 - p1;
    let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
    let normalized_direction = direction.scale(1.0 / length);

    // 计算 OBB 的高度（沿线段方向）和宽度（垂直于线段方向）
    let half_length = (length + 0.4) / 2.0;
    let half_width = width / 2.0;

    // 计算 OBB 的四个顶点
    let obb_axis1 = normalized_direction.scale(half_length); // 沿线段方向的轴
    let obb_axis2 = Vector2::new(-normalized_direction.y, normalized_direction.x).scale(half_width); // 垂直于线段方向的轴

    let p1 = center + obb_axis1 + obb_axis2;
    let p2 = center + obb_axis1 - obb_axis2;
    let p3 = center - obb_axis1 - obb_axis2;
    let p4 = center - obb_axis1 + obb_axis2;

    [p1, p2, p3, p4]
}

#[test]
fn test() {
    // let p1 = (0.0f32, 10.0f32);
    // let p2 = (10.0f32, 0.0f32);
    // let r = 10.0f32;
    // let d = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
    // let mut theta = 2.0 * (d / (2.0 * r)).asin();

    // // large_arc 决定弧线是大于还是小于 180 度，0 表示小角度弧，1 表示大角度弧。
    // // sweep 表示弧线的方向，0 表示从起点到终点沿逆时针画弧，1 表示从起点到终点沿顺时针画弧。

    // let large_arc = false;
    // let sweet = true;
    // if large_arc != (theta > PI) {
    //     theta = TAU - theta;
    // }

    // if sweet {
    //     theta = -theta;
    // }

    // // 将弧度转换为角度
    // let theta_degrees = theta * 180.0 / PI;
    // log::debug!("圆心角（弧度）：{}", theta);
    // log::debug!("圆心角（度）：{}", theta_degrees);

    let p2 = Point::new(0.0, 0.0);
    let p1 = Point::new(4.0, 0.0);
    let width = 2.0;
    let r = vec![[1,2,3]; 4];

    let obb = calculate_obb(p1, p2, width);

    println!("OBB vertices:");
    for point in obb.iter() {
        println!("({}, {})", point.x, point.y);
    }
    let a =Arc::new(Point::new(99.66642, 155.80313), Point::new(104.87043, 152.61417), 0.015748031);
    println!("============ a: {:?}", a);
}


