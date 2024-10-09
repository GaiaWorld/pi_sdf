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

use crate::glyphy::blob::recursion_near_arcs_of_cell;
use crate::glyphy::geometry::arc::Arc;
use crate::glyphy::util::GLYPHY_INFINITY;
use crate::utils::{compute_cell_range, CellInfo, LayoutInfo, OutlineSinkExt, SdfInfo2, TexInfo2};
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
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Segment {
    pub fn new(a_x: f32, a_y: f32, b_x: f32, b_y: f32) -> Self {
        let mut attribute = Attribute::default();
        let a = Point::new(a_x, a_y);
        let b = Point::new(b_x, b_y);
        attribute.start = a;
        attribute.is_close = false;
        Self {
            segment: MSegment::new(a, b),
            attribute,
        }
    }

    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        vec![
            ArcEndpoint::new(self.segment.a.x, self.segment.a.y, f32::INFINITY),
            ArcEndpoint::new(self.segment.b.x, self.segment.b.y, 0.0),
            // ArcEndpoint::new(self.segment.a.x, self.segment.a.y, 0.0),
        ]
    }

    fn _get_stroke_dasharray_arc_endpoints(&self, step: [f32; 2]) -> Vec<ArcEndpoint> {
        let length = self.segment.length();
        let part = step[0] + step[1];
        let num = length / part;
        let mmod = num - num.trunc();
        let dir = (self.segment.b - self.segment.a).normalize();

        let real = dir * step[0];
        let a_virtual = dir * step[1];

        let mut arcs = vec![ArcEndpoint::new(
            self.segment.a.x,
            self.segment.a.y,
            f32::INFINITY,
        )];

        for _ in 0..num as usize {
            let last = arcs.last().unwrap();
            let x = last.p[0] + real[0];
            let y = last.p[1] + real[1];
            let p1 = ArcEndpoint::new(x, y, 0.0);
            let x = p1.p[0] + a_virtual[0];
            let y = p1.p[1] + a_virtual[1];
            let p2 = ArcEndpoint::new(x, y, f32::INFINITY);

            arcs.push(p1);
            arcs.push(p2);
        }

        let last = arcs.last().unwrap();
        if mmod > step[0] / part {
            let x = last.p[0] + real[0];
            let y = last.p[1] + real[1];
            let p = ArcEndpoint::new(x, y, 0.0);
            arcs.push(p);
        } else {
            let x = self.segment.b.x;
            let y = self.segment.b.y;
            let p = ArcEndpoint::new(x, y, 0.0);
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
        // println!("=====e.area():{}", e.area());
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
            self.rx, self.ry, self.cx, self.cy, 4.0,
        ]));
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        Aabb::new(
            Point::new(self.cx - self.rx, self.cy - self.rx),
            Point::new(self.cx + self.rx, self.cy + self.rx),
        )
    }

    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
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

        if !is_close {
            let mut points = self.points.iter().rev();
            let _ = points.next();
            for p in points {
                result.push(ArcEndpoint::new(p.x, p.y, 0.0));
            }
        }

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

        // println!("{:?}", (&points, &verbs));

        let mut attribute = Attribute::default();
        attribute.start = points[0];

        let mut r = Self {
            verbs,
            points,
            attribute,
            is_reverse,
        };
        r.attribute.is_close = r.is_close();

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

        // println!("{:?}", (&points, &verbs));

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

        println!("attribute.is_close: {:?}", r.attribute.is_close);
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct SvgInfo {
    binding_box: Vec<f32>,
    arc_endpoints: Vec<ArcEndpoint>,
    is_area: bool,
    is_reverse: Option<bool>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl SvgInfo {
    pub fn new(
        binding_box: &[f32],
        arc_endpoints: Vec<f32>,
        is_area: bool,
        is_reverse: Option<bool>,
    ) -> SvgInfo {
        assert_eq!(arc_endpoints.len() % 3, 0);
        let mut arc_endpoints2 = Vec::with_capacity(arc_endpoints.len() / 3);
        arc_endpoints
            .chunks(3)
            .for_each(|v| arc_endpoints2.push(ArcEndpoint::new(v[0], v[1], v[2])));
        SvgInfo {
            binding_box: binding_box.to_vec(),
            arc_endpoints: arc_endpoints2,
            is_area,
            is_reverse,
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

    pub fn compute_near_arcs_of_wasm(&self, scale: f32) -> Vec<u8> {
        bitcode::serialize(&self.compute_near_arcs(scale)).unwrap()
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

    pub fn compute_sdf_tex_of_wasm(
        &self,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
        scale: f32,
    ) -> Vec<u8> {
        bitcode::serialize(&self.compute_sdf_tex(tex_size, pxrange, is_outer_glow, cur_off, scale))
            .unwrap()
    }
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
    // println!("p: {:?}", points);
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
    println!("flag: {}", flag);
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

pub fn compute_near_arcs<'a>(view_box: Aabb, endpoints: &Vec<ArcEndpoint>, scale: f32) -> CellInfo {
    let extents = compute_cell_range(view_box, scale);
    // println!("extents: {:?}", extents);
    // let extents = compute_cell_range(extents, scale);
    let mut min_width = f32::INFINITY;
    let mut min_height = f32::INFINITY;

    let mut p0 = Point::new(0., 0.);
    // println!("extents2: {:?}", extents);
    let mut near_arcs = Vec::with_capacity(endpoints.len());
    let mut arcs = Vec::with_capacity(endpoints.len());
    // println!("endpoints: {:?}", endpoints);
    for i in 0..endpoints.len() {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);

        near_arcs.push(arc);
        arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
    }

    let mut result_arcs = vec![];
    let mut temp = Vec::with_capacity(arcs.len());
    // println!("arcs:{:?}", arcs.len());
    recursion_near_arcs_of_cell(
        &near_arcs,
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

#[test]
fn test() {
    let p1 = (0.0f32, 10.0f32);
    let p2 = (10.0f32, 0.0f32);
    let r = 10.0f32;
    let d = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
    let mut theta = 2.0 * (d / (2.0 * r)).asin();

    // large_arc 决定弧线是大于还是小于 180 度，0 表示小角度弧，1 表示大角度弧。
    // sweep 表示弧线的方向，0 表示从起点到终点沿逆时针画弧，1 表示从起点到终点沿顺时针画弧。

    let large_arc = false;
    let sweet = true;
    if large_arc != (theta > PI) {
        theta = TAU - theta;
    }

    if sweet {
        theta = -theta;
    }

    // 将弧度转换为角度
    let theta_degrees = theta * 180.0 / PI;
    println!("圆心角（弧度）：{}", theta);
    println!("圆心角（度）：{}", theta_degrees);
}
