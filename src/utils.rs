use std::collections::HashMap;

use ab_glyph_rasterizer::{point, Rasterizer};
use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
// use image::{EncodableLayout, ImageBuffer, Rgba};

// use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use usvg::{Color, Fill, NonZeroPositiveF32, Paint, Stroke};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    glyphy::{geometry::arcs::GlyphyArcAccumulator, sdf::glyphy_sdf_from_arc_list2},
    Point, shape::{SvgScenes, Rect, ArcOutline},
};
use parry2d::bounding_volume::Aabb;
use std::hash::Hasher;

use crate::{
    font::FontFace,
    glyphy::{
        blob::{line_encode, snap, BlobArc, Extents, UnitArc},
        geometry::{
            aabb::AabbEXT,
            arc::{Arc, ArcEndpoint},
            line::Line,
            point::PointExt,
            vector::VectorEXT,
        },
        util::{is_inf, GLYPHY_INFINITY},
    },
};

pub static MIN_FONT_SIZE: f32 = 10.0;

pub static TOLERANCE: f32 = 10.0 / 1024.;

pub static ENLIGHTEN_MAX: f32 = 0.0001; /* Per EM */

pub static EMBOLDEN_MAX: f32 = 0.0001; /* Per EM */

pub struct User {
    pub accumulate: GlyphyArcAccumulator,
    pub path_str: String,
    pub svg_paths: Vec<String>,
    pub svg_endpoints: Vec<[f32; 2]>,
}

#[wasm_bindgen]
pub struct GlyphVisitor {
    rasterizer: Rasterizer,
    pub(crate) accumulate: GlyphyArcAccumulator,
    #[cfg(feature = "debug")]
    pub(crate) path_str: String,
    #[cfg(feature = "debug")]
    pub(crate) svg_paths: Vec<String>,
    pub(crate) svg_endpoints: Vec<[f32; 2]>,

    scale: f32,
    pub(crate) start: Point,
    pub(crate) previous: Point,
    pub index: usize,
}

#[wasm_bindgen]
impl GlyphVisitor {
    pub fn new(scale: f32) -> Self {
        let accumulate = GlyphyArcAccumulator::new();
        let rasterizer = ab_glyph_rasterizer::Rasterizer::new(512, 512);
        Self {
            rasterizer,
            accumulate,
            #[cfg(feature = "debug")]
            path_str: "".to_string(),
            #[cfg(feature = "debug")]
            svg_paths: vec![],
            svg_endpoints: vec![],
            scale,
            start: Point::default(),
            previous: Point::default(),
            index: 0,
        }
    }
}

// impl GlyphVisitor {
//     pub fn get_pixmap(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
//         let mut img = ImageBuffer::from_fn(512, 512, |_, _| Rgba([255u8, 0, 0, 0]));

//         self.rasterizer.for_each_pixel_2d(|x, y, a| {
//             let rgba = img.get_pixel_mut(x, 512 - y - 1);
//             rgba[3] = (a * 255.0) as u8;
//         });

//         return img;
//     }
// }

impl OutlineSink for GlyphVisitor {
    fn move_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::debug!("M {} {} ", to.x, to.y);

        if self.scale > 0.02 {
            self.accumulate
                .move_to(Point::new(to.x as f32, to.y as f32));
            #[cfg(feature = "debug")]
            self.path_str.push_str(&format!("M {} {}", to.x, to.y));
            self.svg_endpoints.push([to.x as f32, to.y as f32]);
        }

        self.start = to;
        self.previous = to;
    }

    fn line_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::debug!("+ L {} {} ", to.x, to.y);
        if self.scale > 0.02 {
            self.accumulate.line_to(to);
            #[cfg(feature = "debug")]
            self.path_str.push_str(&format!("L {} {}", to.x, to.y));
            self.svg_endpoints.push([to.x as f32, to.y as f32]);
        } else {
            self.rasterizer.draw_line(
                point(self.previous.x * self.scale, self.previous.y * self.scale),
                point(to.x, to.y),
            );
        }

        self.previous = to;
    }

    fn quadratic_curve_to(&mut self, control: Vector2F, to: Vector2F) {
        let control = Point::new(control.x(), control.y());
        let to = Point::new(to.x(), to.y());

        log::debug!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
        if self.scale > 0.02 {
            self.accumulate.conic_to(control, to);
            self.svg_endpoints.push([to.x, to.y]);
        } else {
            self.rasterizer.draw_quad(
                point(self.previous.x * self.scale, self.previous.y * self.scale),
                point(control.x * self.scale, control.y * self.scale),
                point(to.x * self.scale, to.y * self.scale),
            );
        }
        self.previous = to;
    }

    fn cubic_curve_to(&mut self, control: LineSegment2F, to: Vector2F) {
        // 字形数据没有三次贝塞尔曲线
        let control1 = Point::new(control.from_x(), control.from_y());
        let control2 = Point::new(control.to_x(), control.to_y());
        let to = Point::new(to.x(), to.y());

        log::debug!(
            "+ C {}, {}, {}, {}, {}, {}",
            control1.x,
            control1.y,
            control2.x,
            control2.y,
            to.x,
            to.y
        );

        if self.scale > 0.02 {
            self.accumulate.cubic_to(control1, control2, to);
            self.svg_endpoints.push([to.x, to.y]);
        } else {
            self.rasterizer.draw_cubic(
                point(self.previous.x * self.scale, self.previous.y * self.scale),
                point(control1.x * self.scale, control1.y * self.scale),
                point(control1.x * self.scale, control1.y * self.scale),
                point(to.x * self.scale, to.y * self.scale),
            );
        }
    }

    fn close(&mut self) {
        if self.previous != self.start {
            log::debug!("+ L {} {} ", self.start.x, self.start.y);
            if self.scale > 0.02 {
                self.accumulate.line_to(self.start);
                #[cfg(feature = "debug")]
                self.path_str
                    .push_str(&format!("M {} {}", self.start.x, self.start.y));
                self.svg_endpoints
                    .push([self.start.x as f32, self.start.y as f32]);
            } else {
                let x = self.previous.x * self.scale;
                self.rasterizer.draw_line(
                    point(x, (self.previous.y) * self.scale),
                    point(self.start.x * self.scale, self.start.y * self.scale),
                )
            }
        }
        log::debug!("+ Z");
        if self.scale > 0.02 {
            self.accumulate.close_path();
            #[cfg(feature = "debug")]
            {
                self.path_str.push_str("Z");
                self.svg_paths.push(self.path_str.clone());
                self.path_str.clear();
            }
        }

        // let r = self.compute_direction();
        // let s = if r { "顺时针" } else { "逆时针" };
        // println!("{}", s);
        self.index = self.accumulate.result.len();
        // println!("close()");
    }
}

pub fn encode_uint_arc_data(
    result_arcs: Vec<(Vec<&Arc>, Aabb)>,
    extents: &Aabb,
    min_width: f32,
    min_height: f32,
    is_area: Option<bool>,
    // units_per_em: u16,
) -> (Vec<Vec<UnitArc>>, HashMap<u64, u64>) {
    let glyph_width = extents.width();
    let glyph_height = extents.height();
    // 格子列的数量;
    // todo 为了兼容阴影minimip先强制索引纹理为32 * 32
    let mut width_cells = 32 as usize;
    // 格子行的数量
    let mut height_cells = 32 as usize;

    if is_area.is_some() {
        if glyph_width > 128.0 {
            width_cells = 64;
            height_cells = 64;
        }
    }

    // 格子列的数量
    let min_width = glyph_width / width_cells as f32;
    // 格子行的数量
    let min_height = glyph_height / height_cells as f32;

    let mut data = vec![
        vec![
            UnitArc {
                parent_cell: Extents {
                    min_x: 0.,
                    min_y: 0.,
                    max_x: 0.,
                    max_y: 0.
                },
                offset: 0,
                sdf: 0.0,
                #[cfg(feature = "debug")]
                show: "".to_owned(),
                data: Vec::with_capacity(8),
                origin_data: vec![],
                key: u64::MAX,
                s_dist: 0,
                s_dist_1: 0,
                s_dist_2: 0,
                s_dist_3: 0
            };
            width_cells
        ];
        height_cells
    ];

    // let glyph_width = extents.width();
    // let glyph_height = extents.height();
    let c = extents.center();
    let unit = glyph_width.max(glyph_height);

    let mut map = HashMap::with_capacity(32 * 32);
    // 二分计算时，个格子的大小会不一样
    // 统一以最小格子细分
    for (near_arcs, cell) in result_arcs {
        let mut near_endpoints = Vec::with_capacity(8);
        let mut _p1 = Point::new(0.0, 0.0);

        for i in 0..near_arcs.len() {
            let arc = near_arcs[i];

            if i == 0 || !_p1.equals(&arc.p0) {
                let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
                near_endpoints.push(endpoint);
                _p1 = arc.p0;
            }

            let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
            near_endpoints.push(endpoint);
            _p1 = arc.p1;
        }

        let begin = cell.mins - extents.mins;
        let end = cell.maxs - extents.mins;
        let begin_x = (begin.x / min_width).round() as usize;
        let begin_y = (begin.y / min_height).round() as usize;

        let end_x = (end.x / min_width).round() as usize;
        let end_y = (end.y / min_height).round() as usize;
        let parent_cell = Extents {
            min_x: cell.mins.x,
            min_y: cell.mins.y,
            max_x: cell.maxs.x,
            max_y: cell.maxs.y,
        };

        let mut line_result = None;
        let mut arc_result = None;
        // 如果是线段段都编码
        if near_endpoints.len() == 2 && near_endpoints[1].d == 0.0 {
            let start = &near_endpoints[0];
            let end = &near_endpoints[1];

            let mut line = Line::from_points(
                snap(&start.p, &extents, glyph_width, glyph_height),
                snap(&end.p, &extents, glyph_width, glyph_height),
            );
            // Shader的最后 要加回去
            line.c -= line.n.dot(&c.into_vector());
            // shader 的 decode 要 乘回去
            line.c /= unit;

            let line_key = near_endpoints[0].get_line_key(&near_endpoints[1]);
            let le = line_encode(line);

            let mut line_data = ArcEndpoint::new(0.0, 0.0, 0.0);
            line_data.line_key = Some(line_key);
            line_data.line_encode = Some(le);

            line_result = Some((line_data, start.clone(), end.clone()));
        } else {
            if near_endpoints.len() == 4
                && is_inf(near_endpoints[2].d)
                && near_endpoints[0].p.x == near_endpoints[3].p.x
                && near_endpoints[0].p.y == near_endpoints[3].p.y
            {
                let e0 = near_endpoints[2].clone();
                let e1 = near_endpoints[3].clone();
                let e2 = near_endpoints[1].clone();

                near_endpoints.clear();
                near_endpoints.push(e0);
                near_endpoints.push(e1);
                near_endpoints.push(e2);
            }

            // 编码到纹理：该格子 对应 的 圆弧数据
            let mut hasher = pi_hash::DefaultHasher::default();
            let mut key = Vec::with_capacity(20);
            for endpoint in &near_endpoints {
                key.push(endpoint.p.x);
                key.push(endpoint.p.y);
                key.push(endpoint.d);
            }
            hasher.write(bytemuck::cast_slice(&key));
            let result = hasher.finish();

            arc_result = Some(result);
        }

        // If the arclist is two arcs that can be combined in encoding if reordered, do that.
        for i in begin_x..end_x {
            for j in begin_y..end_y {
                let unit_arc = &mut data[j][i];
                if let Some((line_data, start, end)) = line_result.as_ref() {
                    unit_arc.data.push(line_data.clone());
                    // println!("1row: {}, col: {} line_data: {:?}n \n", row, col, unit_arc.data.len());
                    unit_arc.origin_data.push(start.clone());
                    unit_arc.origin_data.push(end.clone());
                    unit_arc.parent_cell = parent_cell;
                } else {
                    let key = arc_result.as_ref().unwrap();
                    unit_arc.data.extend_from_slice(&near_endpoints);
                    unit_arc.parent_cell = parent_cell;
                    unit_arc.key = key.clone();
                }
                let p = Point::new(
                    (i as f32 + 0.5) * min_width + extents.mins.x,
                    (j as f32 + 0.5) * min_height + extents.mins.y,
                );
                let sdf = glyphy_sdf_from_arc_list2(&near_arcs, p).0;
                let a = if let Some(is_area) = is_area {
                    let sdf1 = if !is_area {
                        (256.0 - (sdf.abs()) * 32.0).clamp(0.0, 255.0)
                    } else {
                        (256.0 - sdf * 32.0).clamp(0.0, 255.0)
                    };
                    sdf1
                } else {
                    // let sdf = glyphy_sdf_from_arc_list2(&near_arcs, p).0;
                    // let temp = units_per_em as f32 / 2048.0;
                    // (0.5 - (sdf / (glyph_width / 64.0))).clamp(0.0, 1.0) * 255.0
                    (128.0 - sdf).clamp(0.0, 255.0)
                };
                // println!(
                //     "========== i: {}, j: {}, sdf: {}, near_arcs: {:?}, p: {:?}",
                //     i, j, sdf, near_arcs, p
                // );
                unit_arc.s_dist = a as u8;
            }
        }
        let key = data[begin_y][begin_x].get_key();
        let ptr: *const UnitArc = &data[begin_y][begin_x];
        // 使用map 去重每个格子的数据纹理
        map.insert(key, ptr as u64);
    }
    (data, map)
}

#[wasm_bindgen]
pub fn get_char_arc_debug(char: String) -> BlobArc {
    // console_error_panic_hook::set_once();

    let _ = console_log::init_with_level(log::Level::Debug);
    // let buffer = include_bytes!("../source/msyh.ttf").to_vec();
    let buffer = vec![];
    log::debug!("1111111111");
    let mut ft_face = FontFace::new(buffer);
    log::debug!("22222222char: {}", char);
    let char = char.chars().next().unwrap();
    log::debug!("13333333");
    let outline = ft_face.to_outline(char);
    let (arcs, _map) = FontFace::get_char_arc( ft_face.max_box.clone(), outline);
    log::debug!("44444444444");

    let mut shapes = SvgScenes::new(Aabb::new(Point::new(0.0, 0.0), Point::new(400.0, 400.0)));
    // 矩形
    let mut rect = Rect::new(120.0, 70.0, 100.0, 50.0);
    // 填充颜色 默认0. 0. 0. 0.
    rect.attribute.set_fill_color(0, 0, 255);
    // 描边颜色 默认 0. 0. 0. 
    rect.attribute.set_stroke_color(0, 0, 0);
    // 描边宽度，默认0.0
    rect.attribute.set_stroke_width(2.0);
    shapes.add_shape(rect.get_hash(), Box::new(rect));
    arcs
}

// #[wasm_bindgen]
// pub fn compute_svg_debug() -> BlobArc {
//     // console_error_panic_hook::set_once();

//     let _ = console_log::init_with_level(log::Level::Debug);
//     let buffer = include_bytes!("../svg.svg").to_vec();
//     let mut svg = Svg::new(buffer);
//     let sink = compute_endpoints();
//     let (arcs, _) = svg.compute_near_arc(sink[0].accumulate.result.clone());
//     arcs
// }

pub fn to_arc_cmds(endpoints: &Vec<ArcEndpoint>) -> (Vec<Vec<String>>, Vec<[f32; 2]>) {
    let mut _cmd = vec![];
    let mut cmd_array = vec![];
    let mut current_point = None;
    let mut pts = vec![];
    for ep in endpoints {
        pts.push([ep.p.x, ep.p.y]);

        if ep.d == GLYPHY_INFINITY {
            if current_point.is_none() || !ep.p.equals(current_point.as_ref().unwrap()) {
                if _cmd.len() > 0 {
                    cmd_array.push(_cmd);
                    _cmd = vec![];
                }
                _cmd.push(format!(" M ${}, ${}", ep.p.x, ep.p.y));
                current_point = Some(ep.p);
            }
        } else if ep.d == 0.0 {
            assert!(current_point.is_some());
            if current_point.is_some() && !ep.p.equals(current_point.as_ref().unwrap()) {
                _cmd.push(format!(" L {}, {}", ep.p.x, ep.p.y));
                current_point = Some(ep.p);
            }
        } else {
            assert!(current_point.is_some());
            let mut _current_point = current_point.as_ref().unwrap();
            if !ep.p.equals(_current_point) {
                let arc = Arc::new(_current_point.clone(), ep.p, ep.d);
                let center = arc.center();
                let radius = arc.radius();
                let start_v = _current_point - center;
                let start_angle = start_v.sdf_angle();

                let end_v = ep.p - (center);
                let end_angle = end_v.sdf_angle();

                // 大于0，顺时针绘制
                let cross = start_v.sdf_cross(&end_v);

                _cmd.push(arc_to_svg_a(
                    center.x,
                    center.y,
                    radius,
                    start_angle,
                    end_angle,
                    cross < 0.0,
                ));

                _current_point = &ep.p;
            }
        }
    }
    if _cmd.len() > 0 {
        cmd_array.push(_cmd);
        _cmd = vec![]
    }

    return (cmd_array, pts);
}

pub fn arc_to_svg_a(
    x: f32,
    y: f32,
    radius: f32,
    _start_angle: f32,
    end_angle: f32,
    anticlockwise: bool,
) -> String {
    // 计算圆弧结束点坐标
    let end_x = x + radius * end_angle.cos();
    let end_y = y + radius * end_angle.sin();

    // large-arc-flag 的值为 0 或 1，决定了弧线是大于还是小于或等于 180 度
    let large_arc_flag = '0'; // endAngle - startAngle <= Math.PI ? '0' : '1';

    // sweep-flag 的值为 0 或 1，决定了弧线是顺时针还是逆时针方向
    let sweep_flag = if anticlockwise { '0' } else { '1' };

    // 返回 SVG "A" 命令参数
    return format!(
        "A {} {} 0 {} {} {} {}",
        radius, radius, large_arc_flag, sweep_flag, end_x, end_y
    );
}

pub fn create_indices() -> [u16; 6] {
    [0, 1, 2, 1, 2, 3]
}

#[derive(Debug, Default, Clone)]
pub struct Attribute {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub is_close: bool,
    pub start: Point,
}

unsafe impl Send for Attribute {}
unsafe impl Sync for Attribute {}
impl Attribute {
    pub fn set_fill_color(&mut self, r: u8, g: u8, b: u8) {
        let fill = Fill::from_paint(Paint::Color(Color::new_rgb(r, g, b)));
        self.fill = Some(fill);
    }

    pub fn set_stroke_color(&mut self, r: u8, g: u8, b: u8) {
        if let Some(stroke) = &mut self.stroke {
            stroke.paint = Paint::Color(Color::new_rgb(r, g, b));
        } else {
            let mut stroke = Stroke::default();
            stroke.paint = Paint::Color(Color::new_rgb(r, g, b));
            self.stroke = Some(stroke);
        }
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        if let Some(stroke) = &mut self.stroke {
            stroke.width = NonZeroPositiveF32::new(width).unwrap();
        } else {
            let mut stroke = Stroke::default();
            stroke.width = NonZeroPositiveF32::new(width).unwrap();
            self.stroke = Some(stroke);
        }
    }

    pub fn set_stroke_dasharray(&mut self, dasharray: Vec<f32>) {
        if let Some(stroke) = &mut self.stroke {
            stroke.dasharray = Some(dasharray)
        } else {
            let mut stroke = Stroke::default();
            stroke.dasharray = Some(dasharray);
            self.stroke = Some(stroke);
        }
    }
}
