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
// 这是用于描述路径的verbs（动词）的枚举类型。
// 每个变体都对应于SVG路径数据中的一个具体操作。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PathVerb {
    // 移动到绝对位置。这会将当前点设置为指定的坐标，无需绘制线条。
    MoveTo = 1,
    // 相对于当前位置移动。这与MoveTo类似，但参数是相对于当前点的偏移量。
    MoveToRelative = 2,
    // 绘制直线到绝对位置。这会在当前点和指定点之间绘制一条直线。
    LineTo = 3,
    // 绘制相对位置的直线。这与LineTo类似，但参数是相对于当前点的。
    LineToRelative = 4,
    // 绘制二次贝塞尔曲线到绝对位置。
    QuadTo = 5,
    // 绘制相对于当前位置的二次贝塞尔曲线。
    QuadToRelative = 6,
    // 绘制平滑的二次贝塞尔曲线到绝对位置。
    // 前一个点将作为对称点，用于保持曲线的平滑。
    SmoothQuadTo = 7,
    // 绘制平滑的二次贝塞尔曲线到相对位置。
    SmoothQuadToRelative = 8,
    // 绘制三次贝塞尔曲线到绝对位置。
    CubicTo = 9,
    // 绘制相对于当前位置的三次贝塞尔曲线。
    CubicToRelative = 10,
    // 绘制平滑的三次贝塞尔曲线到绝对位置。
    // 前一个点将作为对称点，保持曲线的平滑性。
    SmoothCubicTo = 11,
    // 绘制平滑的三次贝塞尔曲线到相对位置。
    SmoothCubicToRelative = 12,
    // 绘制水平线到绝对位置。这会在当前点横向移动到指定的x坐标。
    HorizontalLineTo = 13,
    // 绘制水平线到相对位置，这会在当前点横向移动指定的偏移量。
    HorizontalLineToRelative = 14,
    // 绘制垂直线到绝对位置。这会在当前点纵向移动到指定的y坐标。
    VerticalLineTo = 15,
    // 绘制垂直线到相对位置，这会在当前点纵向移动指定的偏移量。
    VerticalLineToRelative = 16,
    // 绘制椭圆弧到绝对位置。
    EllipticalArcTo = 17,
    // 绘制椭圆弧到相对位置。
    EllipticalArcToRelative = 18,
    // 关闭路径。这会连接当前点到路径起点，形成一个闭合区域。
    Close = 19,
}

// 允许PathVerb类型转换为f32值。
// 这对于不同类型的verbs在数值上进行比较或计算非常有用。
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
/// 一个圆的形状结构。
/// 这个结构描述了一个圆，包括其圆心坐标和半径。
/// 同时，它还包含一些属性，如起点位置和是否闭合。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Circle {
    /// 圆的半径。必须大于0。
    radius: f32,
    /// 圆心的x坐标。
    cx: f32,
    /// 圆心的y坐标。
    cy: f32,
    /// 圆的其他属性，如起点位置等。
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Circle {
    /// 创建一个新的圆。
    ///
    /// # 参数
    /// * `cx` - 圆心的x坐标。
    /// * `cy` - 圆心的y坐标。
    /// * `radius` - 圆的半径，必须大于0。
    ///
    /// # 返回值
    /// * 如果半径是有效的（大于0），则返回一个新的Circle实例。
    /// * 如果半径小于或等于0，返回一个错误信息。
    pub fn new(cx: f32, cy: f32, radius: f32) -> Result<Circle, String> {
        if radius <= 0.0 {
            return Err("radius of circle must be > 0".to_string());
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

    /// 获取圆的弧端点信息。
    /// 这些端点用于后续的绘制或计算。
    /// 目前返回的是一些固定的点，实际应用中可能需要根据具体的圆的参数生成。
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

    /// 计算圆的包围盒。
    ///
    /// # 返回值
    /// * 包围盒的Aabb结构体，包含圆的最小和最大坐标。
    fn binding_box(&self) -> Aabb {
        Aabb::new(
            Point::new(self.cx - self.radius, self.cy - self.radius),
            Point::new(self.cx + self.radius, self.cy + self.radius),
        )
    }

    /// 获取圆的SVG信息。
    ///
    /// # 返回值
    /// * 包含圆包围盒、弧端点、是否是区域、是否是反向、哈希值和纹理大小的SvgInfo结构体。
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
            tex_size,
        }
    }

    /// 获取圆的SVG信息，适用于WebAssembly。
    ///
    /// # 返回值
    /// * 包含序列化后的SVG信息、包围盒、是否是区域、哈希值和纹理大小的WasmSvgInfo结构体。
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

    /// 判断圆是否是一种区域。
    ///
    /// # 返回值
    /// * 总是返回true，因为圆本质上是一个闭合的区域。
    pub fn is_area(&self) -> bool {
        true
    }
}

impl Circle {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}
/// 矩形结构体描述了矩形的几何属性和相关信息。
/// 这个结构包含矩形的位置、尺寸和一些绘图属性。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Rect {
    /// 矩形的x坐标。
    x: f32,
    /// 矩形的y坐标。
    y: f32,
    /// 矩形的宽度。
    width: f32,
    /// 矩形的高度。
    height: f32,
    /// 绘图属性，包括起点位置和闭合状态。
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Rect {
    /// 创建一个新的矩形。
    ///
    /// # 参数
    /// * `x` - 矩形的x坐标。
    /// * `y` - 矩形的y坐标。
    /// * `width` - 矩形的宽度，必须为正数。
    /// * `height` - 矩形的高度，必须为正数。
    ///
    /// # 返回值
    /// * 返回一个新的Rect实例。
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

    /// 获取矩形的弧端点信息。
    /// 这些端点用于后续的绘制或计算。
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

    /// 计算矩形的哈希值。
    /// 哈希值基于矩形的尺寸和比例。
    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&[
            self.width / self.height,
            if self.width <= 64.0 {
                32.0
            } else {
                2.0f32.powf(self.width.log2().floor())
            },
            2.0,
        ]));
        hasher.finish()
    }

    /// 计算矩形的包围盒。
    /// 包围盒是包含矩形的最小轴对齐矩形。
    fn binding_box(&self) -> Aabb {
        let min_x = self.x.min(self.x + self.width);
        let min_y = self.y.min(self.y + self.height);
        let max_x = self.x.max(self.x + self.width);
        let max_y = self.y.max(self.y + self.height);

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    /// 获取矩形的SVG信息。
    /// 包括包围盒、弧端点、是否是区域、反向、哈希值和纹理大小等信息。
    pub fn get_svg_info(&self) -> SvgInfo {
        let binding_box = self.binding_box();
        let size = (binding_box.maxs.x - binding_box.mins.x)
            .max(binding_box.maxs.y - binding_box.mins.y)
            .ceil();
        let tex_size = if size <= 64.0 {
            32.0
        } else {
            2.0f32.powf(size.log2().floor())
        };
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
            tex_size,
        }
    }

    /// 获取矩形的SVG信息，适用于WebAssembly。
    /// 包括序列化后的SVG信息、包围盒、是否是区域、哈希值和纹理大小。
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

    /// 判断矩形是否是一个区域。
    /// 矩形本质上是一个闭合的区域，所以总是返回true。
    pub fn is_area(&self) -> bool {
        true
    }
}

impl Rect {
    /// 获取矩形的绘图属性。
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Segment {
    segment: MSegment,
    /// 绘制属性，控制线段的外观和行为。
    pub(crate) attribute: Attribute,
    /// 分割段，用于在WebAssembly中高效处理线段的分段操作。
    step: Option<Vec<f32>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Segment {
    /// 创建一个新的线段。
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

    /// 获取线段的弧线端点。
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

    /// 获取线段的弧线端点，适用于虚线绘制。
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

    /// 获取线段的哈希值。
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

    /// 获取线段的包围盒。
    fn binding_box(&self) -> Aabb {
        let min_x = self.segment.a.x.min(self.segment.b.x);
        let min_y = self.segment.a.y.min(self.segment.b.y);
        let max_x = self.segment.a.x.max(self.segment.b.x);
        let max_y = self.segment.a.y.max(self.segment.b.y);

        Aabb::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    /// 获取线段的 SVG 信息。
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
            tex_size,
        }
    }

    /// 获取线段的 SVG 信息，适用于WebAssembly环境。
    /// 返回一个包含序列化后的SVG信息、包围盒、区域标记、哈希值和纹理尺寸的结构体。
    /// 使用bitcode库进行序列化，以便在WebAssembly中高效传输和处理。
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

    /// 判断线段是否表示一个区域。
    /// 当前实现中，线段不表示区域，总是返回false。
    pub fn is_area(&self) -> bool {
        false
    }
}

impl Segment {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

/// 代表一个椭圆形状的结构体。
///
/// 该结构体包含椭圆的中心坐标(cx, cy)以及长半轴(rx)和短半轴(ry)的长度。
/// 同时，包含绘制属性，用于控制椭圆的外观和渲染行为。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Ellipse {
    cx: f32,                         // 椭圆的中心x坐标。
    cy: f32,                         // 椭圆的中心y坐标。
    rx: f32,                         // 椭圆的长半轴长度，控制x方向半径。
    ry: f32,                         // 椭圆的短半轴长度，控制y方向半径。
    pub(crate) attribute: Attribute, // 绘制属性，定义椭圆的外观和行为特性。
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Ellipse {
    /// 创建一个新的椭圆实例。
    ///
    /// # 参数
    /// - `cx`: 椭圆的中心x坐标。
    /// - `cy`: 椭圆的中心y坐标。
    /// - `rx`: 椭圆的长半轴长度。
    /// - `ry`: 椭圆的短半轴长度。
    ///
    /// # 返回值
    /// 一个新的椭圆实例。
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

    /// 获取椭圆的弧线端点。
    /// 这些端点用于绘制椭圆的边界，包括起始点和终点。
    /// returned一个包含这些端点的向量。
    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
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

    /// 获取椭圆的哈希值。
    ///
    /// 该哈希值用于缓存和快速比较，基于椭圆的长轴和短轴长度的比例计算。
    pub fn get_hash(&self) -> u64 {
        let mut hasher = pi_hash::DefaultHasher::default();
        hasher.write(bytemuck::cast_slice(&[self.rx / self.ry, 4.0]));
        hasher.finish()
    }

    /// 计算椭圆的包围盒。
    /// 包围盒由椭圆中心与半轴长度决定，范围覆盖椭圆的所有点。
    fn binding_box(&self) -> Aabb {
        Aabb::new(
            Point::new(self.cx - self.rx, self.cy - self.ry),
            Point::new(self.cx + self.rx, self.cy + self.ry),
        )
    }

    /// 获取椭圆的SVG信息。
    ///
    /// 该方法返回一个包含绘制箱、弧线端点、区域标记、哈希值和纹理尺寸的信息，
    /// 用于生成和展示椭圆的SVG内容。
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
            tex_size,
        }
    }

    /// 获取用于WebAssembly环境的SVG信息。
    ///
    /// 序列化SVG信息以便在Web环境中高效传输和处理，
    /// 返回一个包含序列化数据、包围盒、区域标记、哈希值和纹理尺寸的结构体。
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

    /// 判断椭圆是否表示一个区域。
    /// 当前实现中，椭圆不作为区域，总是返回false。
    pub fn is_area(&self) -> bool {
        true
    }
}

impl Ellipse {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}
/// 多边形数据结构。
/// 一个多边形由多个顶点点坐标组成，用于描述平面上的封闭图形。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Polygon {
    points: Vec<Point>,
    pub(crate) attribute: Attribute,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Polygon {
    /// 创建一个新的多边形实例。
    ///
    /// 参数是一个包含顶点坐标的向量，每两个元素表示一个点的x和y坐标。
    /// 多边形将按顺时针或逆时针顺序排列顶点，以确保正确绘制。
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

    /// 获取多边形的弧线端点信息。
    /// 这些端点用于绘制多边形的边界，包括起始点和终点。
    /// 返回一个包含这些端点的向量。
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

    /// 计算多边形的哈希值。
    /// 哈希值用于缓存和快速比较，基于多边形的顶点坐标计算得出。
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

    /// 计算多边形的包围盒。
    /// 包围盒由多边形的最小和最大坐标决定，包含多边形的所有顶点。
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

    /// 获取多边形的SVG信息。
    ///
    /// 该方法返回一个包含绘制箱、弧线端点、区域标记、哈希值和纹理尺寸的信息，
    /// 用于生成和展示多边形的SVG内容。
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
            tex_size,
        }
    }

    /// 获取用于WebAssembly环境的SVG信息。
    ///
    /// 序列化SVG信息以便在Web环境中高效传输和处理，
    /// 返回一个包含序列化数据、包围盒、区域标记、哈希值和纹理尺寸的结构体。
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

    /// 判断多边形是否表示一个区域。
    ///
    /// 当前实现中，多边形用于区域填充，总是返回true。
    pub fn is_area(&self) -> bool {
        true
    }
}

impl Polygon {
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}
/// 多线段数据结构。
/// 由多个线段组成，用于描述在平面上的路径或线条。
/// 多线段可以闭合也可以开放，具体取决于构造时的定义。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Polyline {
    points: Vec<Point>, // 点的坐标集合，每一点由x和y座标组成。
    pub(crate) attribute: Attribute, // 多线段的属性，包括闭合标记和起点等信息。
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Polyline {
    /// 创建一个新的多线段实例。
    ///
    /// 参数是包含各点的x和y坐标的向量，每两个元素组成一个点。
    /// 构造时会自动逆序点序列以确保正确的绘制方向。
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

    /// 判断多线段是否为闭合图形。
    /// 检查首尾点是否相近，若是则认为闭合。
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

    /// 获取多线段的弧端点信息。
    /// 为每个点分配一个弧端点，起点设置无限远，其他点设置为0。
    /// 目前未考虑线段的方向性雅克。
    pub fn get_arc_endpoints(&self) -> Vec<ArcEndpoint> {
        let is_close = self.attribute.is_close;

        let mut points = self.points.iter();
        let mut result = Vec::with_capacity(points.len() + 1);

        let start = points.next().unwrap();
        result.push(ArcEndpoint::new(start.x, start.y, f32::INFINITY));

        for p in points {
            result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        }

        // 目前未考虑线段的反向绘制
        // if !is_close {
        //     let mut points = self.points.iter().rev();
        //     let _ = points.next();
        //     for p in points {
        //         result.push(ArcEndpoint::new(p.x, p.y, 0.0));
        //     }
        // }

        result
    }

    /// 计算多线段的哈希值。
    /// 基于各点的坐标生成唯一的哈希值，用于缓存和快速比较。
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

    /// 计算多线段的包围盒。
    /// 包围盒由所有点的最小和最大坐标决定，包含所有顶点。
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

    /// 获取多线段的SVG信息。
    /// 返回绘制箱、弧端点、区域标记、哈希值和纹理尺寸，
    /// 用于生成和展示多线段的SVG内容。
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

    /// 获取用于WebAssembly环境的SVG信息。
    /// 序列化SVG信息，以便在Web环境中高效传输和处理。
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

    /// 判断多线段是否表示一个区域。
    /// 多线段用于绘制线条，不表示区域，总是返回false。
    pub fn is_area(&self) -> bool {
        false
    }
}

impl Polyline {
    /// 获取多线段的属性。
    pub fn get_attribute(&self) -> Attribute {
        self.attribute.clone()
    }
}

/// 提供路径操作的结构体，封装了路径的动词（如移动、线、曲线等）和点的数据。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Path {
    /// 所有的路径动词，如移动（Move）、直线（Line）、弧线（Arc）等。
    verbs: Vec<PathVerb>,
    /// 仅用于特殊情况，主要是为了与C接口兼容。
    points: Vec<Point>,
    /// 路径的属性，如画笔颜色、宽度等。
    pub(crate) attribute: Attribute,
    /// 标记路径是否反向绘制。
    is_reverse: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Path {
    /// 创建一个新的Path实例。
    ///
    /// # 参数
    /// - `verbs`: 表示路径动词的向量，例如移动、线、弧等操作。
    /// - `points`: 表示路径上的点的向量，按照x1, y1, x2, y2的顺序排列。
    ///
    /// # 返回值
    /// 一个新的Path实例。
    pub fn new(verbs: Vec<u8>, points: Vec<f32>) -> Self {
        // 初始化调试日志（仅测试环境有效）
        // let _ = console_log::init_with_level(log::Level::Warn);

        // 将动词向量从u8转换为PathVerb类型
        let verbs: Vec<PathVerb> = unsafe { transmute(verbs) };

        Self::new1(verbs, points)
    }

    /// 创建一个新的Path实例，使用预转换的PathVerb向量。
    ///
    /// # 参数
    /// - `verbs`: 已经转换好的PathVerb向量。
    /// - `points`: 表示路径上的点的向量，按照x1, y1, x2, y2的顺序排列。
    ///
    /// # 返回值
    /// 一个新的Path实例。
    pub fn new1(verbs: Vec<PathVerb>, points: Vec<f32>) -> Self {
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
        //
        //     // let temp = verbs[0];
        //     // let len = verbs.len();
        //     // verbs[0] = verbs[len - 1];
        //     // verbs[len - 1] = temp;
        // }

        // 初始化Path属性参数
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
    /// 判断路径是否为闭合形状。
    /// 在一些情况下，路径可能需要闭合以表示区域。
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

    /// 获取并计算路径的弧端点列表。
    /// 这个过程包括生成路径的重建、参数计算以及包围盒确定。
    ///
    /// # 返回值
    /// 包含三个元素的元组：
    /// 1. 包含所有弧端点的向量。
    /// 2. 路径的包围盒。
    /// 3. 哑声记号指示器。
    fn get_arc_endpoints(&self) -> (Vec<ArcEndpoint>, Aabb, usize) {
        let mut sink = GlyphVisitor::new(1.0);
        // 设置圆弧拟合贝塞尔曲线的精度，参数值越小精度越高。
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

    /// 计算路径的哈希值。
    /// 将路径的顶点和动词转换为字节流，哈希后生成唯一标识符。
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

    /// 计算路径的包围盒。
    ///
    /// # 返回值
    /// 包围盒由路径顶点的最小和最大坐标决定，用来表示路径占用的矩形区域。
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

    /// 获取路径的SVG信息。
    /// 计算生成的SVG信息包括包围盒、路径点、区域标记、哈希值和纹理尺寸，
    /// 为后续生成和展示SVG内容做准备。
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
            tex_size,
        }
    }

    /// 获取导用于WebAssembly的SVG信息。
    /// 这个方法将SVG信息转换为适用于Web环境的格式，方便在WebAssembly中使用和展示。
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

    /// 判断路径是否代表一个区域。
    /// 该路径是否闭合决定了是否代表区域，闭合则可能表示区域。
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
    /// SVG绘图的包围盒，用于确定绘制的矩形范围
    pub binding_box: Vec<f32>,
    /// 弧端点的详细信息，用于构建SVG路径
    arc_endpoints: Vec<ArcEndpoint>,
    /// 是否表示一个区域，闭合路径通常表示区域
    pub is_area: bool,
    /// 是否被标记为反向的标志，某些情况下可能需要反转
    is_reverse: Option<bool>,
    /// 唯一的哈希值，用于标识不同的路径内容
    pub hash: u64,
    /// 纹理尺寸，用于在Web环境中进行绘制
    pub tex_size: f32,
}

impl SvgInfo {
    pub fn new(
        binding_box: &[f32],
        arc_endpoints: Vec<f32>,
        is_area: bool,
        is_reverse: Option<bool>,
    ) -> SvgInfo {
        // 检查弧端点数组是否为3的倍数，确保每个弧端点都有x、y和方向参数。
        assert_eq!(arc_endpoints.len() % 3, 0);

        // 初始化哈希计算器，用于生成唯一的哈希值。
        let mut hasher = pi_hash::DefaultHasher::default();
        // 将弧端点数据转换为字节流进行哈希计算。
        hasher.write(bytemuck::cast_slice(&arc_endpoints));
        // 完成哈希计算，获取哈希值。
        let hash = hasher.finish();

        // 将弧端点的一维数组转换为ArcEndpoint对象的向量。
        let mut arc_endpoints2 = Vec::with_capacity(arc_endpoints.len() / 3);
        arc_endpoints
            .chunks(3)
            .for_each(|v| arc_endpoints2.push(ArcEndpoint::new(v[0], v[1], v[2])));
        // 返回新的SvgInfo实例。
        SvgInfo {
            binding_box: binding_box.to_vec(),
            arc_endpoints: arc_endpoints2,
            is_area,
            is_reverse,
            hash,
            tex_size: 0.0,
        }
    }

    pub fn compute_layout(&self, tex_size: usize, pxrange: u32, cur_off: u32) -> LayoutInfo {
        // 计算图形布局信息，以确定绘制的位置和大小。
        compute_layout(&self.binding_box, tex_size, pxrange, 1, cur_off, true)
    }

    pub fn compute_near_arcs(&self, scale: f32) -> CellInfo {
        // 计算近似路径的信息，用于后续绘制或处理。
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
        // 获取布局信息，如绘制平面的边界和宽度等。
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = self.compute_layout(tex_size, pxrange, cur_off);

        // 获取近似路径的弧线信息。
        let CellInfo { arcs, info, .. } = compute_near_arcs(
            Aabb::new(
                Point::new(self.binding_box[0], self.binding_box[1]),
                Point::new(self.binding_box[2], self.binding_box[3]),
            ),
            &self.arc_endpoints,
            scale,
        );

        // 打印调试信息，输出弧线和信息。
        // println!("============1111111111111111 : {:?}",(&arcs, &info));

        // 编码SDF纹理，生成位图数据。
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

        // 返回SDF信息，包含位图数据、纹理尺寸以及纹理纹理详细信息。
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
        // 计算单个SDF单元格，用于高效拼接和显示。
        let cell = self.compute_near_arcs(scale);
        let blob = cell.encode_blob_arc();
        blob.encode_tex()
    }

    pub fn compute_positions_and_uv(&self, ps: &[f32], uv: &[f32], thickness: f32, out_ps: &mut Vec<f32>, out_uv: &mut Vec<f32>, out_indices: &mut Vec<u16>){
        if self.is_area {
            return;
        }
        let thickness = thickness * 2.0;
        let ps_w = ps[2] - ps[0];
        let ps_h = ps[3] - ps[1];
        let uv_w = uv[2] - uv[0];
        let uv_h = uv[3] - uv[1];

        let mut prev = None;
        let mut curr = None;
        let mut next = None;
        let mut p0 = Point::new(0., 0.);
        let half_thickness = thickness / 2.0;

        for i in 0..self.arc_endpoints.len() {
            let endpoint = &self.arc_endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = Point::new(endpoint.p[0], endpoint.p[1]);
                continue;
            }
            let p1 = Point::new(endpoint.p[0], endpoint.p[1]);
            if curr.is_none() {
                curr = Some(Arc::new(p0, p1, endpoint.d));
            }
            next = self
                .arc_endpoints
                .get(i + 1)
                .map(|v| Arc::new(p1, Point::new(v.p[0], v.p[1]), v.d));

            // 计算起点和终点的平均法线
            let start_normal = calculate_joint_normal(&prev, &curr, &p0);
            let end_normal = calculate_joint_normal(&curr, &next, &p1);

            let verts = if float_equals(endpoint.d, 0.0, None) {
                // 生成带修正法线的线段
                let start_offset = [
                    start_normal[0] * half_thickness,
                    start_normal[1] * half_thickness,
                ];
                let end_offset = [
                    end_normal[0] * half_thickness,
                    end_normal[1] * half_thickness,
                ];

                vec![
                    Point::new(p0[0] + start_offset[0], p0[1] + start_offset[1]),
                    Point::new(p0[0] - start_offset[0], p0[1] - start_offset[1]),
                    Point::new(p1[0] + end_offset[0], p1[1] + end_offset[1]),
                    Point::new(p1[0] - end_offset[0], p1[1] - end_offset[1]),
                ]
            } else {
                // 修改后的圆弧生成逻辑（需调整首末顶点）
                let mut arc_verts = curr.as_ref().unwrap().generate_arc_vertices(thickness);

                // 替换首末顶点的法线方向
                if !arc_verts.is_empty() {
                    // 修改第一个顶点对
                    arc_verts[0] = Point::new(
                        p0[0] + start_normal[0] * half_thickness,
                        p0[1] + start_normal[1] * half_thickness,
                    );
                    arc_verts[1] = Point::new(
                        p0[0] - start_normal[0] * half_thickness,
                        p0[1] - start_normal[1] * half_thickness,
                    );

                    // 修改最后一个顶点对
                    let last = arc_verts.len() - 1;
                    arc_verts[last - 1] = Point::new(
                        p1[0] + end_normal[0] * half_thickness,
                        p1[1] + end_normal[1] * half_thickness,
                    );
                    arc_verts[last] = Point::new(
                        p1[0] - end_normal[0] * half_thickness,
                        p1[1] - end_normal[1] * half_thickness,
                    );
                }
                arc_verts
            };
            println!("========= verts: {:?}, arc: {:?}", verts, curr);
            let s = 0.0;
            for i in 0..verts.len() / 4 {
                let start = (out_ps.len() / 2) as u16;

                let p0_x = verts[i].x;
                let uv0_x = (p0_x - ps[0]) / ps_w * uv_w + uv[0] - uv[0] * s;
                out_ps.push(p0_x);
                out_uv.push(uv0_x);

                let p0_y = verts[i].y;
                let uv0_y = (p0_y - ps[1]) / ps_h * uv_h + uv[1];
                out_ps.push(p0_y);
                out_uv.push(uv0_y);

                let p1_x = verts[i + 1].x;
                let uv1_x = (p1_x - ps[0]) / ps_w * uv_w + uv[0] - uv[0] * s;
                out_ps.push(p1_x);
                out_uv.push(uv1_x);

                let p1_y = verts[i + 1].y;
                let uv1_y = (p1_y - ps[1]) / ps_h * uv_h + uv[1];
                out_ps.push(p1_y);
                out_uv.push(uv1_y);

                let p2_x = verts[i + 2].x;
                let uv2_x = (p2_x - ps[0]) / ps_w * uv_w + uv[0] - uv[0] * s;
                out_ps.push(p2_x);
                out_uv.push(uv2_x);

                let p2_y = verts[i + 2].y;
                let uv2_y = (p2_y - ps[1]) / ps_h * uv_h + uv[1];
                out_ps.push(p2_y);
                out_uv.push(uv2_y);

                let p3_x = verts[i + 3].x;
                let uv3_x = (p3_x - ps[0]) / ps_w * uv_w + uv[0] - uv[0] * s;
                out_ps.push(p3_x);
                out_uv.push(uv3_x);

                let p3_y = verts[i + 3].y;
                let uv3_y = (p3_y - ps[1]) / ps_h * uv_h + uv[1];
                out_ps.push(p3_y);
                out_uv.push(uv3_y);

                out_indices.push(start);
                out_indices.push(start + 1);
                out_indices.push(start + 2);
                out_indices.push(start + 1);
                out_indices.push(start + 2);
                out_indices.push(start + 3);
            }

            p0 = p1;
            prev = curr;
            curr = next;
        }
    }
}

/// 计算节点处的法线向量。
///
/// 这个函数用于在曲线的节点处 calculates 则 calculates 法线向量。它考虑了前后两个曲线段（prev_arc 和 next_arc），分别来自前一段的末端和后一段的起点。
///
/// 参数：
/// - `prev_arc`: 前一段的曲线段（如果存在）
/// - `next_arc`: 后一段的曲线段（如果存在）
/// - `point`: 当前节点的位置坐标
///
/// 返回值：
/// 返回一个归一化的法线向量数组，具有两个元素，分别对应x和y方向的分量。
fn calculate_joint_normal(
    prev_arc: &Option<Arc>,
    next_arc: &Option<Arc>,
    point: &Point,
) -> [f32; 2] {
    let mut normal = [0.0, 0.0];

    // 前向法线（来自前一段的末端）
    if let Some(prev) = prev_arc {
        if prev.d < 1e-4 {
            // 如果曲线段的长度非常短，可以视为直线段
            // 使用方向向量的垂直方向作为法线
            let dir = [prev.p1[0] - prev.p0[0], prev.p1[1] - prev.p0[1]];
            let perp = [-dir[1], dir[0]];
            let len = (perp[0].powi(2) + perp[1].powi(2)).sqrt();
            normal[0] += perp[0] / len;
            normal[1] += perp[1] / len;
        } else {
            // 否则，使用节点点相对于圆心的方向作为法线
            let dx = point[0] - prev.center[0];
            let dy = point[1] - prev.center[1];
            let len = (dx.powi(2) + dy.powi(2)).sqrt();
            normal[0] += dx / len;
            normal[1] += dy / len;
        }
    }

    // 后向法线（来自后一段的起点）
    if let Some(next) = next_arc {
        if next.d < 1e-4 {
            // 如果下一段的长度非常短，同样视为直线段
            let dir = [next.p1[0] - next.p0[0], next.p1[1] - next.p0[1]];
            let perp = [-dir[1], dir[0]];
            let len = (perp[0].powi(2) + perp[1].powi(2)).sqrt();
            normal[0] += perp[0] / len;
            normal[1] += perp[1] / len;
        } else {
            // 使用节点点相对于下一段圆心的方向作为法线
            let dx = point[0] - next.center[0];
            let dy = point[1] - next.center[1];
            let len = (dx.powi(2) + dy.powi(2)).sqrt();
            normal[0] += dx / len;
            normal[1] += dy / len;
        }
    }

    // 归一化平均法线
    // 对平均后的法线向量进行归一化处理
    let len = (normal[0].powi(2) + normal[1].powi(2)).sqrt();
    if len > 0.0 {
        [normal[0] / len, normal[1] / len]
    } else {
        [0.0, 0.0]  // 如果长度为零，返回默认法线向量（这里为零向量）
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

    pub fn compute_positions_and_uv_of_wasm(
        info: &[u8],
        ps: &[f32],
        uv: &[f32],
        half_extend: f32,
    ) -> PosInfo {
        let info: SvgInfo = bitcode::deserialize(info).unwrap();
        let mut info2 = PosInfo::default();
        info.compute_positions_and_uv(
            ps,
            uv,
            half_extend,
            &mut info2.out_ps,
            &mut info2.out_uv,
            &mut info2.out_indices,
        );
        info2
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Default)]
pub struct PosInfo {
    pub out_ps: Vec<f32>,
    pub out_uv: Vec<f32>,
    pub out_indices: Vec<u16>,
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

// 计算路径绕行方向的函数
// 通过分析路径中的点，确定路径是顺时针还是逆时针绕行
// 输入：路径的点的集合，类型为Vec<Point>
// 输出：布尔值，true表示顺时针，false表示逆时针
fn compute_direction(path: &Vec<Point>) -> bool {
    // 初始化最大x坐标和索引
    let mut max_x = -f32::INFINITY;
    let mut index = 0;

    // 遍历路径中的每个点，找出x坐标最大的点，并记录其索引
    for i in 0..path.len() {
        if path[i].x > max_x {
            max_x = path[i].x;
            index = i;
        }
    }

    // 获取前一个点的索引，假设路径是封闭的，第一个点也连接到最后一个点
    let mut previous = path.len() - 1;
    if index != 0 {
        previous = index - 1;
    }

    // 获取下一个点的索引，假设路径是封闭的，最后一个点连接到第一个点
    let mut next = index + 1;
    if next >= path.len() {
        next = 0;
    }

    // 计算当前点与前一个点和下一个点的向量
    // a为从前一个点指向当前点的向量
    // b为从当前点指向下一个点的向量
    let a = path[index] - path[previous];
    let b = path[next] - path[index];

    // 计算向量a和向量b的叉积
    //叉积会给出一个在z轴方向的向量，通过它的符号来判断绕行方向
    let v =
        parry2d::na::Vector3::new(a.x, a.y, 0.0).cross(&parry2d::na::Vector3::new(b.x, b.y, 0.0));

    // 如果叉积在z轴的分量为负，表示绕行方向为顺时针
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
}
