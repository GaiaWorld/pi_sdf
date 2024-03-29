// use ab_glyph_rasterizer::Point;
use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use erased_serde::serialize_trait_object;
use image::EncodableLayout;
use kurbo::Shape;
use parry2d::{bounding_volume::Aabb, shape::Segment as MSegment};
use serde::{Deserialize, Serialize};
// use usvg::tiny_skia_path::PathSegment;

use crate::utils::Attribute;
use crate::{
    glyphy::{
        blob::{EncodeError, TexData, TexInfo},
        geometry::{arc::ArcEndpoint, point::PointExt},
        util::float_equals,
    },
    svg::compute_near_arc_impl,
    utils::GlyphVisitor,
    Point,
};
use std::{collections::HashMap, fmt::Debug, hash::Hasher};

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PathVerb {
    // 绝对点
    MoveTo = 1,
    // 相对点
    MoveToRelative,
    LineTo,
    LineToRelative,
    QuadTo,
    QuadToRelative,
    SmoothQuadTo,
    SmoothQuadToRelative,
    CubicTo,
    CubicToRelative,
    SmoothCubicTo,
    SmoothCubicToRelative,
    HorizontalLineTo,
    HorizontalLineToRelative,
    VerticalLineTo,
    VerticalLineToRelative,
    EllipticalArcTo,
    EllipticalArcToRelative,
    Close,
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
pub trait ArcOutline: Send + Sync + Debug + ShapeClone {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint>;
    fn get_attribute(&self) -> Attribute;
    fn get_hash(&self) -> u64;
    fn binding_box(&self) -> Aabb;
}

pub trait ShapeClone {
    fn clone_box(&self) -> Box<dyn ArcOutline>;
}

impl<T> ShapeClone for T
where
    T: 'static + ArcOutline + Clone,
{
    fn clone_box(&self) -> Box<dyn ArcOutline> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ArcOutline> {
    fn clone(&self) -> Box<dyn ArcOutline> {
        self.clone_box()
    }
}

// serialize_trait_object!(ArcOutline);
// impl Serialize for Box<dyn ArcOutline> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer {
//         todo!()
//     }
// }

// impl Deserialize for Box<dyn ArcOutline> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de> {
//         todo!()
//     }
// }

#[derive(Clone, Debug)]
pub struct Circle {
    radius: f32,
    cx: f32,
    cy: f32,
    pub attribute: Attribute,
}

impl Circle {
    pub fn new(cx: f32, cy: f32, radius: f32) -> Result<Self, String> {
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
}

impl ArcOutline for Circle {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
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

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write([self.cx, self.cy, self.radius, 1.0].as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        Aabb {
            mins: Point::new(self.cx - self.radius, self.cy - self.radius),
            maxs: Point::new(self.cx + self.radius, self.cy + self.radius),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    pub attribute: Attribute,
}

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
}

impl ArcOutline for Rect {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
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

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write([self.x, self.y, self.width, self.height, 2.0].as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let min_x = self.x.min(self.x - self.width);
        let min_y = self.y.min(self.y - self.height);
        let max_x = self.x.max(self.x - self.width);
        let max_x = self.y.max(self.y - self.height);
        Aabb { mins: Point::new(min_x, min_y), maxs:Point::new( max_x, max_x) }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    segment: MSegment,
    pub attribute: Attribute,
}

impl Segment {
    pub fn new(a: Point, b: Point) -> Self {
        let mut attribute = Attribute::default();
        attribute.start = a;
        attribute.is_close = false;
        Self {
            segment: MSegment::new(a, b),
            attribute,
        }
    }
}

impl ArcOutline for Segment {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        vec![
            ArcEndpoint::new(self.segment.a.x, self.segment.a.y, f32::INFINITY),
            ArcEndpoint::new(self.segment.b.x, self.segment.b.y, 0.0),
            ArcEndpoint::new(self.segment.a.x, self.segment.a.y, 0.0),
        ]
    }

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(
            [
                self.segment.a.x,
                self.segment.a.y,
                self.segment.b.x,
                self.segment.b.y,
                3.0,
            ]
            .as_bytes(),
        );
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let min_x = self.segment.a.x.min(self.segment.b.x);
        let min_y = self.segment.a.y.min(self.segment.b.y);
        let max_x = self.segment.a.x.max(self.segment.b.x);
        let max_x = self.segment.a.y.max(self.segment.b.y);
        Aabb { mins: Point::new(min_x, min_y), maxs:Point::new( max_x, max_x) }
    }
}

#[derive(Clone, Debug)]
pub struct Ellipse {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    pub attribute: Attribute,
}

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
}

impl ArcOutline for Ellipse {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let center = kurbo::Point::new(self.cx as f64, self.cy as f64);
        let e = kurbo::Ellipse::new(center, (self.rx as f64, self.ry as f64), 0.0);

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
        println!("=====e.area():{}", e.area());
        if e.area() > 0.0 {
            let temp = verbs[0];
            let len = verbs.len();
            verbs[0] = verbs[len - 1];
            verbs[len - 1] = temp;
            compute_outline(points.iter().rev(), verbs.iter().rev(), &mut sink)
        } else {
            compute_outline(points.iter(), verbs.iter(), &mut sink)
        }

        sink.accumulate.result
    }

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write([self.rx, self.ry, self.cx, self.cy, 4.0].as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        Aabb {
            mins: Point::new(self.cx - self.rx, self.cy - self.rx),
            maxs: Point::new(self.cx + self.rx, self.cy + self.rx),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Polygon {
    points: Vec<Point>,
    pub attribute: Attribute,
}

impl Polygon {
    pub fn new(mut points: Vec<Point>) -> Self {
        if !compute_direction(&points) {
            points.reverse();
        };
        let mut attribute = Attribute::default();
        attribute.is_close = true;
        attribute.start = points[0];

        Self { points, attribute }
    }
}

impl ArcOutline for Polygon {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let len = self.points.len();
        let mut points = self.points.iter();

        let mut result = Vec::with_capacity(len + 1);
        let start = points.next().unwrap();

        result.push(ArcEndpoint::new(start.x, start.y, f32::INFINITY));

        for p in points {
            result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        }

        let end = result.last().unwrap();
        if !float_equals(end.p.x, start.x, None) || !float_equals(end.p.y, start.y, None) {
            result.push(ArcEndpoint::new(start.x, start.y, 0.0));
        }

        result
    }

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 2);
        for p in &self.points {
            key.push(p.x);
            key.push(p.y);
        }
        key.push(5.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(key.as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        self.points.iter().for_each(|v|{
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        });
        
        Aabb { mins: Point::new(min_x, min_y), maxs:Point::new( max_x, max_x) }
    }
}

#[derive(Clone, Debug)]
pub struct Polyline {
    points: Vec<Point>,
    pub attribute: Attribute,
}

impl Polyline {
    pub fn new(mut points: Vec<Point>) -> Self {
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
}

impl ArcOutline for Polyline {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
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

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 2);
        for p in &self.points {
            key.push(p.x);
            key.push(p.y);
        }
        key.push(6.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(key.as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        self.points.iter().for_each(|v|{
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        });
        
        Aabb { mins: Point::new(min_x, min_y), maxs:Point::new( max_x, max_x) }
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    verbs: Vec<PathVerb>,
    points: Vec<Point>,
    pub attribute: Attribute,
}

impl Path {
    pub fn new(mut verbs: Vec<PathVerb>, mut points: Vec<Point>) -> Self {
        if points.len() > 2 && !compute_direction(&points) {
            points.reverse();
            verbs.reverse();

            let temp = verbs[0];
            let len = verbs.len();
            verbs[0] = verbs[len - 1];
            verbs[len - 1] = temp;
        };

        let mut attribute = Attribute::default();
        attribute.start = points[0];

        let mut r = Self {
            verbs,
            points,
            attribute,
        };
        r.attribute.is_close = r.is_close();

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
}

impl ArcOutline for Path {
    fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let mut sink = GlyphVisitor::new(1.0);
        // 圆弧拟合贝塞尔曲线的精度，值越小越精确
        sink.accumulate.tolerance = 0.1;

        let is_close = self.is_close();
        compute_outline(self.points.iter(), self.verbs.iter(), &mut sink);
        if !is_close {
            compute_outline(
                self.points[0..self.points.len() - 1].iter().rev(),
                self.verbs[1..self.verbs.len()].iter().rev(),
                &mut sink,
            );
        }
        sink.accumulate.result
    }

    fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }

    fn get_hash(&self) -> u64 {
        let mut key = Vec::with_capacity(self.points.len() * 4);
        for (p, v) in self.points.iter().zip(self.verbs.iter()) {
            key.push(p.x);
            key.push(p.y);
            key.push((*v).into());
        }
        key.push(7.0);
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(key.as_bytes());
        hasher.finish()
    }

    fn binding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;


        
        Aabb { mins: Point::new(min_x, min_y), maxs:Point::new( max_x, max_x) }
    }
}

pub struct SvgScenes {
    pub shapes: HashMap<u64, Box<dyn ArcOutline>>,
    pub view_box: Aabb,
}

impl SvgScenes {
    pub fn new(view_box: Aabb) -> Self {
        // 添加空隙
        let view_box = Aabb {
            mins: Point::new(view_box.mins.x, view_box.mins.y),
            maxs: Point::new(view_box.maxs.x, view_box.maxs.y),
        };
        Self {
            shapes: Default::default(),
            view_box,
        }
    }

    pub fn add_shape(&mut self, hash: u64, shape: Box<dyn ArcOutline>) {
        // let key = shape.get_hash();
        self.shapes.insert(hash, shape);
    }

    pub fn verties(&self) -> [f32; 16] {
        [
            self.view_box.mins.x,
            self.view_box.mins.y,
            0.0,
            0.0,
            self.view_box.mins.x,
            self.view_box.maxs.y,
            0.0,
            1.0,
            self.view_box.maxs.x,
            self.view_box.mins.y,
            1.0,
            0.0,
            self.view_box.maxs.x,
            self.view_box.maxs.y,
            1.0,
            1.0,
        ]
    }

    pub fn out_tex_data(
        &self,
        tex_data: &mut TexData,
    ) -> Result<(Vec<TexInfo>, Vec<Attribute>), EncodeError> {
        let mut infos = vec![];
        let mut attributes = vec![];

        let data_tex = &mut tex_data.data_tex;
        let width0 = tex_data.data_tex_width;
        let offset_x0 = &mut tex_data.data_offset_x;
        let offset_y0 = &mut tex_data.data_offset_y;

        let index_tex = &mut tex_data.index_tex;
        let width1 = tex_data.index_tex_width;
        let offset_x1 = &mut tex_data.index_offset_x;
        let offset_y1 = &mut tex_data.index_offset_y;
        let mut last_offset1 = (*offset_x1, *offset_x1);

        let sdf_tex = &mut tex_data.sdf_tex;
        let sdf_tex1 = &mut tex_data.sdf_tex1;
        let sdf_tex2 = &mut tex_data.sdf_tex2;
        let sdf_tex3 = &mut tex_data.sdf_tex3;

        for node in self.shapes.values() {
            let (mut blob_arc, map) =
                compute_near_arc_impl(self.view_box, node.get_arc_endpoints());
            let size = blob_arc.encode_data_tex(&map, data_tex, width0, offset_x0, offset_y0)?;
            println!("data_map: {}", map.len());
            let mut info = blob_arc.encode_index_tex(
                index_tex, width1, offset_x1, offset_y1, map, size, sdf_tex, sdf_tex1, sdf_tex2,
                sdf_tex3,
            )?;

            info.index_offset = last_offset1;
            info.data_offset = (*offset_x0, *offset_y0);
            println!(
                "info.index_offset: {:?}, info.data_offset: {:?}",
                info.index_offset, info.data_offset
            );
            *offset_x0 += size / 8;
            if size % 8 != 0 {
                *offset_x0 += 1;
            }

            last_offset1 = (*offset_x1, *offset_y1);

            infos.push(info);
            attributes.push(node.get_attribute())
        }

        Ok((infos, attributes))
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
    sink: &mut impl OutlineSink,
) {
    let mut prev_to = Vector2F::default();
    for p in verbs {
        match p {
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
                panic!("EllipticalArcTo is not surpport!!!")
            }

            PathVerb::Close => {
                sink.close();
            }
        }
    }
    // is_close
}
