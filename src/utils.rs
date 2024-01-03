use std::collections::{HashMap, HashSet};

use ab_glyph_rasterizer::{point, Rasterizer};
use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use image::{ImageBuffer, Rgba};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{glyphy::geometry::arcs::GlyphyArcAccumulator, Point};
use parry2d::bounding_volume::Aabb;

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
    pub(crate) path_str: String,
    pub(crate) svg_paths: Vec<String>,
    pub(crate) svg_endpoints: Vec<[f32; 2]>,

    scale: f32,
    pub(crate) start: Point,
    previous: Point,
}

#[wasm_bindgen]
impl GlyphVisitor {
    pub fn new(scale: f32) -> Self {
        let accumulate = GlyphyArcAccumulator::new();
        let rasterizer = ab_glyph_rasterizer::Rasterizer::new(512, 512);
        Self {
            rasterizer,
            accumulate,
            path_str: "".to_string(),
            svg_paths: vec![],
            svg_endpoints: vec![],
            scale,
            start: Point::default(),
            previous: Point::default(),
        }
    }
}

impl GlyphVisitor {
    pub fn get_pixmap(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut img = ImageBuffer::from_fn(512, 512, |_, _| Rgba([255u8, 0, 0, 0]));

        self.rasterizer.for_each_pixel_2d(|x, y, a| {
            let rgba = img.get_pixel_mut(x, 512 - y - 1);
            rgba[3] = (a * 255.0) as u8;
        });

        return img;
    }
}

impl OutlineSink for GlyphVisitor {
    fn move_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::info!("M {} {} ", to.x, to.y);

        if self.scale > 0.02 {
            self.accumulate
                .move_to(Point::new(to.x as f32, to.y as f32));
            self.path_str.push_str(&format!("M {} {}", to.x, to.y));
            self.svg_endpoints.push([to.x as f32, to.y as f32]);
        }

        self.start = to;
        self.previous = to;
    }

    fn line_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::info!("+ L {} {} ", to.x, to.y);
        if self.scale > 0.02 {
            self.accumulate.line_to(to);
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

        log::info!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
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

        log::info!(
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
            log::info!("+ L {} {} ", self.start.x, self.start.y);
            if self.scale > 0.02 {
                self.accumulate.line_to(self.start);
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
        log::info!("+ Z");
        if self.scale > 0.02 {
            self.accumulate.close_path();
            self.path_str.push_str("Z");
            self.svg_paths.push(self.path_str.clone());
            self.path_str.clear();
        }
        // println!("close()");
    }
}

pub fn encode_uint_arc_data(
    result_arcs: Vec<(Vec<&Arc>, Aabb)>,
    extents: &Aabb,
    min_width: f32,
    min_height: f32,
) -> (Vec<Vec<UnitArc>>, HashMap<String, u64>) {
    let glyph_width = extents.width();
    let glyph_height = extents.height();

    let width_cells = (glyph_width / min_width).round() as usize;
    let height_cells = (glyph_height / min_height).round() as usize;

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
                show: "".to_owned(),
                data: Vec::with_capacity(8),
                origin_data: vec![],
                key: "".to_string()
            };
            width_cells
        ];
        height_cells
    ];

    let glyph_width = extents.width();
    let glyph_height = extents.height();
    let c = extents.center();
    let unit = glyph_width.max(glyph_height);

    let mut map = HashMap::with_capacity(32 * 32);

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
            let mut key = "".to_string();
            for endpoint in &near_endpoints {
                key.push_str(&format!(
                    "{}_{}_{}_",
                    endpoint.p.x, endpoint.p.y, endpoint.d
                ));
            }
            arc_result = Some(key);
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
                    unit_arc.key = key.clone();
                }
            }
        }
        let key = data[begin_y][begin_x].get_key();
        let ptr: *const UnitArc = &data[begin_y][begin_x];
        map.insert(key, ptr as u64);
    }
    (data, map)
}

#[wasm_bindgen]
pub fn get_char_arc_debug(char: String) -> BlobArc {
    console_error_panic_hook::set_once();

    let _ = console_log::init_with_level(log::Level::Debug);
    let buffer = include_bytes!("../source/msyh.ttf").to_vec();
    log::info!("1111111111");
    let mut ft_face = FontFace::new(buffer);
    log::info!("22222222char: {}", char);
    let char = char.chars().next().unwrap();
    log::info!("13333333");
    let (arcs, set) = ft_face.get_char_arc(char);
    log::info!("44444444444");
    arcs
}

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
