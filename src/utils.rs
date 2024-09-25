use core::fmt;
use std::collections::HashMap;

use ab_glyph_rasterizer::Rasterizer;
use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
// use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use usvg::{Color, Fill, NonZeroPositiveF64, Paint, Stroke};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    font::SdfInfo2,
    glyphy::{
        blob::TexInfo2,
        geometry::{aabb::Aabb, arcs::GlyphyArcAccumulator},
        sdf::{glyphy_sdf_from_arc_list2, glyphy_sdf_from_arc_list3},
        util::float2_equals,
    },
    Point,
};
use parry2d::math::Vector;
use std::hash::Hasher;

use crate::{
    font::FontFace,
    glyphy::{
        blob::{line_encode, snap, Extents, UnitArc},
        geometry::{
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

pub static SCALE: f32 = 2048.0;
pub struct User {
    pub accumulate: GlyphyArcAccumulator,
    pub path_str: String,
    pub svg_paths: Vec<String>,
    pub svg_endpoints: Vec<[f32; 2]>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct GlyphVisitor {
    _rasterizer: Rasterizer,
    pub(crate) accumulate: GlyphyArcAccumulator,
    #[cfg(feature = "debug")]
    pub(crate) path_str: String,
    #[cfg(feature = "debug")]
    pub(crate) svg_paths: Vec<String>,
    pub(crate) svg_endpoints: Vec<[f32; 2]>,

    scale: f32,
    // scale2: f32,
    pub(crate) start: Point,
    pub(crate) previous: Point,
    pub index: usize,
    pub(crate) bbox: Aabb,
    pub(crate) arcs: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl GlyphVisitor {
    pub fn new(scale: f32) -> Self {
        let accumulate = GlyphyArcAccumulator::new();
        let _rasterizer = ab_glyph_rasterizer::Rasterizer::new(512, 512);
        Self {
            _rasterizer,
            accumulate,
            #[cfg(feature = "debug")]
            path_str: "".to_string(),
            #[cfg(feature = "debug")]
            svg_paths: vec![],
            svg_endpoints: vec![],
            scale,
            // scale2,
            start: Point::default(),
            previous: Point::default(),
            index: 0,
            bbox: Aabb::new(
                Point::new(core::f32::MAX, core::f32::MAX),
                Point::new(core::f32::MIN, core::f32::MIN),
            ),
            arcs: 0,
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

pub trait OutlineSinkExt: OutlineSink {
    fn arc2_to(&mut self, d: f32, to: Vector2F);
}

impl OutlineSinkExt for GlyphVisitor {
    fn arc2_to(&mut self, d: f32, to: Vector2F) {
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::info!("+ L {} {} ", to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.arc_to(to, d);
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("L {} {}", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_line(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(to.x, to.y),
        //     );
        // }
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }
}

impl OutlineSink for GlyphVisitor {
    fn move_to(&mut self, to: Vector2F) {
        self.arcs += 1;
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::info!("M {} {} ", to.x, to.y);

        // if self.scale > 0.02 {
        self.accumulate.move_to(Point::new(to.x, to.y));
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("M {} {}", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // }
        self.bbox.extend_by(to.x, to.y);
        self.start = to;
        self.previous = to;
    }

    fn line_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::info!("+ L {} {} ", to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.line_to(to);
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("L {} {}", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_line(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(to.x, to.y),
        //     );
        // }
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }

    fn quadratic_curve_to(&mut self, control: Vector2F, to: Vector2F) {
        let control = Point::new(control.x(), control.y()) * self.scale;
        let to = Point::new(to.x(), to.y()) * self.scale;

        log::info!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.conic_to(control, to);
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_quad(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(control.x * self.scale, control.y * self.scale),
        //         point(to.x * self.scale, to.y * self.scale),
        //     );
        // }
        self.bbox.extend_by(control.x, control.y);
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }

    fn cubic_curve_to(&mut self, control: LineSegment2F, to: Vector2F) {
        // 字形数据没有三次贝塞尔曲线
        let control1 = Point::new(control.from_x(), control.from_y()) * self.scale;
        let control2 = Point::new(control.to_x(), control.to_y()) * self.scale;
        let to = Point::new(to.x(), to.y()) * self.scale;

        log::info!(
            "+ C {}, {}, {}, {}, {}, {}",
            control1.x,
            control1.y,
            control2.x,
            control2.y,
            to.x,
            to.y
        );

        // if self.scale > 0.02 {
        self.accumulate.cubic_to(control1, control2, to);
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_cubic(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(control1.x * self.scale, control1.y * self.scale),
        //         point(control1.x * self.scale, control1.y * self.scale),
        //         point(to.x * self.scale, to.y * self.scale),
        //     );
        // }
        self.bbox.extend_by(control1.x, control1.y);
        self.bbox.extend_by(control2.x, control2.y);
        self.bbox.extend_by(to.x, to.y);

        self.previous = to;
    }

    fn close(&mut self) {
        if self.previous != self.start {
            // log::debug!("+ L {} {} ", self.start.x, self.start.y);
            // if self.scale > 0.02 {
            self.accumulate.line_to(self.start);
            #[cfg(feature = "debug")]
            self.path_str
                .push_str(&format!("M {} {}", self.start.x, self.start.y));
            self.svg_endpoints.push([self.start.x, self.start.y]);
            // } else {
            //     let x = self.previous.x * self.scale;
            //     self.rasterizer.draw_line(
            //         point(x, (self.previous.y) * self.scale),
            //         point(self.start.x * self.scale, self.start.y * self.scale),
            //     )
            // }
        }
        log::info!("+ Z");
        // if self.scale > 0.02 {
        self.accumulate.close_path();
        #[cfg(feature = "debug")]
        {
            self.path_str.push_str("Z");
            self.svg_paths.push(self.path_str.clone());
            self.path_str.clear();
        }
        // }

        // let r = self.compute_direction();
        // let s = if r { "顺时针" } else { "逆时针" };
        // println!("{}", s);
        self.index = self.accumulate.result.len();
        // println!("close()");
    }
}

pub fn encode_uint_arc_data(
    result_arcs: Vec<(Vec<Arc>, Aabb)>,
    extents: &Aabb,
    _min_width: f32,
    _min_height: f32,
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
            let arc = &near_arcs[i];

            if i == 0 || !_p1.equals(&arc.p0) {
                let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
                near_endpoints.push(endpoint);
                _p1 = arc.p0;
            }

            let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
            near_endpoints.push(endpoint);
            _p1 = arc.p1;
        }
        // println!("near_endpoints: {:?}", near_endpoints.len());

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
                snap(
                    &Point::new(start.p[0], start.p[1]),
                    &extents,
                    glyph_width,
                    glyph_height,
                ),
                snap(
                    &Point::new(end.p[0], end.p[1]),
                    &extents,
                    glyph_width,
                    glyph_height,
                ),
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
                && near_endpoints[0].p[0] == near_endpoints[3].p[0]
                && near_endpoints[0].p[1] == near_endpoints[3].p[1]
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
                key.push(endpoint.p[0]);
                key.push(endpoint.p[1]);
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

                unit_arc.s_dist = compute_sdf(p, &near_arcs, is_area);
            }
        }
        let key = data[begin_y][begin_x].get_key();
        let ptr: *const UnitArc = &data[begin_y][begin_x];
        // 使用map 去重每个格子的数据纹理
        map.insert(key, ptr as u64);
    }
    (data, map)
}

pub fn encode_sdf(
    global_arcs: &Vec<Arc>,
    result_arcs: Vec<(Vec<usize>, Aabb)>,
    extents: &Aabb,
    width_cells: usize,
    height_cells: usize,
    distance: f32, // sdf在这个值上alpha 衰减为 0
    width: Option<f32>,
    is_outer_glow: bool,
    is_svg: bool,
    is_reverse: Option<bool>,
) -> Vec<u8> {
    // // todo 为了兼容阴影minimip先强制索引纹理为32 * 32
    // let mut width_cells = 32 as usize;
    // // 格子行的数量
    // let mut height_cells = 32 as usize;

    let glyph_width = extents.width();
    let glyph_height = extents.height();

    // 格子列的数量
    let min_width = glyph_width / width_cells as f32;
    // 格子行的数量
    let min_height = glyph_height / height_cells as f32;

    let mut data = vec![0; width_cells * height_cells];

    for (near_arcs, cell) in result_arcs {
        // println!("near_endpoints: {:?}", near_endpoints.len());

        // if cell.
        let begin = cell.mins - extents.mins;
        let end = cell.maxs - extents.mins;
        let begin_x = (begin.x / min_width).round() as usize;
        let begin_y = (begin.y / min_height).round() as usize;

        let end_x = (end.x / min_width).round() as usize;
        let end_y = (end.y / min_height).round() as usize;

        // If the arclist is two arcs that can be combined in encoding if reordered, do that.
        for i in begin_x..end_x {
            for j in begin_y..end_y {
                let p = Point::new(
                    (i as f32 + 0.5) * min_width + extents.mins.x,
                    (j as f32 + 0.5) * min_height + extents.mins.y,
                );
                let r = compute_sdf2(
                    global_arcs,
                    p,
                    &near_arcs,
                    distance,
                    width,
                    is_outer_glow,
                    is_reverse,
                );
                // svg 不需要颠倒纹理
                if is_svg {
                    data[j * width_cells + i] = r;
                } else {
                    data[(height_cells - j - 1) * width_cells + i] = r;
                }
            }
        }
    }
    data
}

pub fn encode_sdf2(
    global_arcs: &Vec<Arc>,
    arcs_info: Vec<(Vec<usize>, Aabb)>,
    extents: &Aabb,
    tex_size: usize,
    distance: f32, // sdf在这个值上alpha 衰减为 0
    width: Option<f32>,
    is_outer_glow: bool,
    is_svg: bool,
    is_reverse: Option<bool>,
) -> Vec<u8> {
    let glyph_width = extents.width();
    // let glyph_height = extents.height();

    let unit_d = glyph_width / tex_size as f32;

    let mut data = vec![0; tex_size * tex_size];

    for (near_arcs, cell) in arcs_info {
        if let Some(ab) = cell.collision(extents) {
            //
            let begin = ab.mins - extents.mins;
            let end = ab.maxs - extents.mins;

            let mut begin_x = begin.x / unit_d;
            if (begin_x.fract() - 0.5).abs() < 0.001 {
                begin_x -= 0.01;
            }
            let begin_x = begin_x.round() as usize;

            let mut begin_y = begin.y / unit_d;
            if (begin_y.fract() - 0.5).abs() < 0.001 {
                begin_y -= 0.01;
            }
            let begin_y = begin_y.round() as usize;

            let mut end_x = end.x / unit_d;
            if (end_x.fract() - 0.5).abs() < 0.001 {
                end_x -= 0.01;
            }
            let end_x = end_x.round() as usize;

            let mut end_y = end.y / unit_d;
            if (end_y.fract() - 0.5).abs() < 0.001 {
                end_y -= 0.01;
            }
            let end_y = end_y.round() as usize;
            // println!("{:?}", (begin_x, begin_y, end_x, end_y));
            // If the arclist is two arcs that can be combined in encoding if reordered, do that.
            for i in begin_x..end_x {
                for j in begin_y..end_y {
                    let p = Point::new(
                        (i as f32 + 0.5) * unit_d + extents.mins.x,
                        (j as f32 + 0.5) * unit_d + extents.mins.y,
                    );
                    // if j == 29 && i == 25 {
                    //     println!(
                    //         "============== cell: {:?}, extents: {:?}, ab: {:?}, unit_d: {:?}",
                    //         cell, extents, ab, unit_d
                    //     );
                    //     println!("begin: {}, end: {}", begin.y / unit_d, end.y / unit_d)
                    // }
                    let r = compute_sdf2(
                        global_arcs,
                        p,
                        &near_arcs,
                        distance,
                        width,
                        is_outer_glow,
                        is_reverse,
                    );
                    // svg 不需要颠倒纹理
                    if is_svg {
                        data[j * tex_size + i] = r;
                    } else {
                        // println!("{:?}", (r, j, i));
                        data[(tex_size - j - 1) * tex_size + i] = r;
                    }
                }
            }
        }
    }
    data
}

fn compute_sdf(p: Point, near_arcs: &Vec<Arc>, is_area: Option<bool>) -> u8 {
    let sdf = glyphy_sdf_from_arc_list2(near_arcs, p).0;
    
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

    a as u8
}

fn compute_sdf2(
    global_arcs: &Vec<Arc>,
    p: Point,
    near_arcs: &Vec<usize>,
    distance: f32,
    width: Option<f32>,
    is_outer_glow: bool,
    is_reverse: Option<bool>,
) -> u8 {
    let mut sdf = glyphy_sdf_from_arc_list3(near_arcs, p.clone(), global_arcs).0;
    sdf = (sdf * 10000.0).round() * 0.0001;
    // let p2 = Point::new(85.0, 82.0) - p;
    // if p2.norm_squared() < 0.1{
    //     println!("p : {:?}", (p, sdf, distance));
    //     for i in near_arcs{
    //         println!("{:?}", global_arcs[*i]);
    //     }
    // }
    let p2 = Point::new(85.5, 84.5) - p;
    if p2.norm_squared() < 0.1 {
        println!("p : {:?}", (p, sdf, distance));
        for i in near_arcs {
            println!("{:?}", global_arcs[*i]);
        }
    }
    let p2 = Point::new(85.5, 85.5) - p;
    if p2.norm_squared() < 0.1 {
        println!("p : {:?}", (p, sdf, distance));
        for i in near_arcs {
            println!("{:?}", global_arcs[*i]);
        }
    }
    if let Some(is_reverse) = is_reverse {
        if is_reverse {
            sdf = -sdf;
        }
    }
    if let Some(_) = width {
        sdf = sdf.abs(); // - (width * 0.5);
    }

    if is_outer_glow {
        let radius = distance;
        // println!("{:?}", (radius, sdf));
        sdf = ((radius - sdf) / radius).clamp(0.0, 1.0).powf(5.0);
        // println!("{:?}", (radius, sdf));
        return (sdf * 127.0).round() as u8;
        // println!("{:?}", (radius, sdf));
    } else {
        sdf = sdf / distance;
        let a = ((1.0 - sdf) * 127.0).round() as u8;
        // println!("sdf: {:?}", (sdf, a, distance));
        return a;
    }
}

pub fn compute_layout(
    extents: &mut Aabb,
    tex_size: usize,
    pxrange: u32,
    units_per_em: u16,
    cur_off: u32,
    is_svg: bool,
) -> (Aabb, Aabb, f32, usize) {
    // map 无序导致每次计算的数据不一样
    // let bbox = extents.clone();
    // println!("")
    let extents_w = extents.width();
    let extents_h = extents.height();
    let scale = 1.0 / units_per_em as f32;
    let plane_bounds = extents.scaled(&Vector::new(scale, scale));

    let px_distance = extents_w.max(extents_h) / tex_size as f32;
    let distance = px_distance * pxrange as f32;
    let expand = px_distance * cur_off as f32;
    // println!("distance: {}", distance);
    extents.mins.x -= expand;
    extents.mins.y -= expand;
    extents.maxs.x += expand;
    extents.maxs.y += expand;

    // let pxrange = (pxrange >> 2 << 2) + 4;
    let tex_size = tex_size + (cur_off * 2) as usize;
    let mut atlas_bounds = Aabb::new_invalid();
    atlas_bounds.mins.x = cur_off as f32;
    atlas_bounds.mins.y = cur_off as f32;
    atlas_bounds.maxs.x = tex_size as f32 - cur_off as f32 - 1.0;
    atlas_bounds.maxs.y = tex_size as f32 - cur_off as f32 - 1.0;

    let temp = extents_w - extents_h;
    if temp > 0.0 {
        extents.maxs.y += temp;
        if is_svg {
            // println!("============= is_svg: {}", (temp / extents.height() * tex_size as f32 - 1.0));
            atlas_bounds.maxs.y -= (temp / extents.height() * tex_size as f32 ).trunc();
        } else {
            // 字体的y最终需要上下颠倒
            atlas_bounds.mins.y += (temp / extents.height() * tex_size as f32).ceil();
        }
    } else {
        extents.maxs.x -= temp;
        atlas_bounds.maxs.x -= (temp.abs() / extents.width() * tex_size as f32).trunc();
    }

    // plane_bounds.scale(
    //     atlas_bounds.width() / 32.0 / plane_bounds.width(),
    //     atlas_bounds.height() / 32.0 / plane_bounds.width(),
    // );
    println!(
        "plane_bounds: {:?}, atlas_bounds: {:?}, tex_size: {}",
        plane_bounds, atlas_bounds, tex_size
    );
    (Aabb(plane_bounds), atlas_bounds, distance, tex_size)
}

pub fn compute_cell_range(mut bbox: Aabb, scale: f32) -> Aabb {
    let scale = scale * 0.5;
    let w = bbox.width();
    let h = bbox.height();

    let temp = w - h;
    if temp > 0.0 {
        bbox.maxs.y += temp;
        // atlas_bounds.maxs.y -= (temp / extents.height() * tex_size as f32).round();
    } else {
        bbox.maxs.x -= temp;
    }

    let w = bbox.width();
    let extents = scale * w;

    bbox.mins.x -= extents;
    bbox.mins.y -= extents;
    bbox.maxs.x += extents;
    bbox.maxs.y += extents;

    bbox
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
// pub fn get_char_arc_debug(char: String) -> BlobArc {
//     // console_error_panic_hook::set_once();

//     let _ = console_log::init_with_level(log::Level::Debug);
//     // let buffer = include_bytes!("../source/msyh.ttf").to_vec();
//     let buffer: Vec<u8> = vec![];
//     // log::debug!("1111111111");
//     #[cfg(not(target_arch = "wasm32"))]
//     let mut ft_face = FontFace::new(Share::new(buffer));
//     #[cfg(target_arch = "wasm32")]
//     let mut ft_face = FontFace::new(buffer);

//     // log::debug!("22222222char: {}", char);
//     let char = char.chars().next().unwrap();
//     // log::debug!("13333333");
//     let result = ft_face.to_outline(char);
//     let (arcs, _map) = FontFace::encode_uint_arc(ft_face.max_box.clone(), result);
//     // log::debug!("44444444444");

//     let mut shapes = SvgScenes::new(Aabb::new(Point::new(0.0, 0.0), Point::new(400.0, 400.0)));
//     // 矩形
//     let mut rect = Rect::new(120.0, 70.0, 100.0, 50.0);
//     // 填充颜色 默认0. 0. 0. 0.
//     rect.attribute.set_fill_color(0, 0, 255);
//     // 描边颜色 默认 0. 0. 0.
//     rect.attribute.set_stroke_color(0, 0, 0);
//     // 描边宽度，默认0.0
//     rect.attribute.set_stroke_width(2.0);
//     shapes.add_shape(rect.get_hash(), rect.get_svg_info(), rect.get_attribute());
//     arcs
// }

// #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
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
        pts.push([ep.p[0], ep.p[1]]);

        if ep.d == GLYPHY_INFINITY {
            if current_point.is_none() || !float2_equals(&ep.p, current_point.as_ref().unwrap()) {
                if _cmd.len() > 0 {
                    cmd_array.push(_cmd);
                    _cmd = vec![];
                }
                _cmd.push(format!(" M ${}, ${}", ep.p[0], ep.p[1]));
                current_point = Some(ep.p);
            }
        } else if ep.d == 0.0 {
            assert!(current_point.is_some());
            if current_point.is_some() && !float2_equals(&ep.p, current_point.as_ref().unwrap()) {
                _cmd.push(format!(" L {}, {}", ep.p[0], ep.p[1]));
                current_point = Some(ep.p);
            }
        } else {
            assert!(current_point.is_some());
            let mut _current_point = current_point.as_ref().unwrap();
            if !float2_equals(&ep.p, _current_point) {
                let arc = Arc::new(
                    Point::new(_current_point[0], _current_point[1]),
                    Point::new(ep.p[0], ep.p[1]),
                    ep.d,
                );
                let center = arc.center();
                let radius = arc.radius();
                let start_v =
                    Vector::new(_current_point[0] - center[0], _current_point[1] - center[1]);
                let start_angle = start_v.sdf_angle();

                let end_v = Vector::new(ep.p[0] - center[0], ep.p[1] - center[1]);
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
            stroke.width = NonZeroPositiveF64::new(width as f64).unwrap();
        } else {
            let mut stroke = Stroke::default();
            stroke.width = NonZeroPositiveF64::new(width as f64).unwrap();
            self.stroke = Some(stroke);
        }
    }

    pub fn set_stroke_dasharray(&mut self, dasharray: Vec<f64>) {
        if let Some(stroke) = &mut self.stroke {
            stroke.dasharray = Some(dasharray)
        } else {
            let mut stroke = Stroke::default();
            stroke.dasharray = Some(dasharray);
            self.stroke = Some(stroke);
        }
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[derive(Debug, Clone)]
// pub struct GlyphInfo {
//     pub char: char,
//     pub plane_bounds: Aabb,
//     pub atlas_bounds: Aabb,
//     pub advance: f32,
//     pub sdf_tex: Vec<u8>,
//     pub tex_size: u32,
// }

// #[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub char: char,
    pub plane_bounds: [f32; 4],
    pub atlas_bounds: [f32; 4],
    pub advance: f32,
    pub sdf_tex: Vec<u8>,
    pub tex_size: u32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct OutlineInfo {
    pub(crate) char: char,
    pub(crate) endpoints: Vec<ArcEndpoint>,
    pub bbox: Aabb,
    pub advance: u16,
    pub units_per_em: u16,
    pub extents: Aabb,
}

impl OutlineInfo {
    pub fn compute_near_arcs(&mut self, scale: f32) -> CellInfo {
        FontFace::compute_near_arcs(self.extents, scale, &mut self.endpoints)
    }

    pub fn compute_sdf_tex(
        &mut self,
        result_arcs: CellInfo,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32
    ) -> SdfInfo2 {
        // println!("bbox: {:?}", self.bbox);
        let mut extents = self.extents;
        let (plane_bounds, atlas_bounds, distance, tex_size) = compute_layout(
            &mut extents,
            tex_size,
            pxrange,
            self.units_per_em,
            cur_off,
            false,
        );
        let CellInfo { arcs, info, .. } = result_arcs;
        let pixmap = encode_sdf2(
            &arcs,
            info,
            &extents,
            tex_size,
            distance,
            None,
            is_outer_glow,
            false,
            None,
        );

        SdfInfo2 {
            tex_info: TexInfo2 {
                char: self.char,
                advance: self.advance as f32 / self.units_per_em as f32,
                sdf_offset_x: 0,
                sdf_offset_y: 0,
                plane_min_x: plane_bounds.mins.x,
                plane_min_y: plane_bounds.mins.y,
                plane_max_x: plane_bounds.maxs.x,
                plane_max_y: plane_bounds.maxs.y,
                atlas_min_x: atlas_bounds.mins.x,
                atlas_min_y: atlas_bounds.mins.y,
                atlas_max_x: atlas_bounds.maxs.x,
                atlas_max_y: atlas_bounds.maxs.y,
            },
            sdf_tex: pixmap,
            tex_size,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl OutlineInfo {
    pub fn compute_near_arcs2(&mut self, scale: f32) -> Vec<u8> {
        bitcode::serialize(&FontFace::compute_near_arcs(
            self.bbox,
            scale,
            &mut self.endpoints,
        ))
        .unwrap()
    }

    pub fn compute_sdf_tex2(
        &mut self,
        result_arcs: &[u8],
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32
    ) -> Vec<u8> {
        let info: CellInfo = bitcode::deserialize(result_arcs).unwrap();
        bitcode::serialize(&self.compute_sdf_tex(info, tex_size, pxrange, is_outer_glow, cur_off)).unwrap()
    }
}

pub fn arc_to_point(arcs: Vec<Arc>) -> Vec<ArcEndpoint> {
    let mut arc_endpoints = Vec::with_capacity(arcs.len());
    let mut _p1 = Point::new(0.0, 0.0);

    for i in 0..arcs.len() {
        let arc = &arcs[i];

        if i == 0 || !_p1.equals(&arc.p0) {
            let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
            arc_endpoints.push(endpoint);
            _p1 = arc.p0;
        }

        let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
        arc_endpoints.push(endpoint);
        _p1 = arc.p1;
    }
    arc_endpoints
}

pub fn point_to_arc(endpoints: Vec<ArcEndpoint>) -> Vec<Arc> {
    let mut p0 = Point::new(0., 0.);
    let mut arcs = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        // let endpoint = &result[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);

        arcs.push(arc);
    }
    arcs
}

#[derive(Debug, Clone)]
pub struct CellInfo {
    pub(crate) extents: Aabb,
    pub arcs: Vec<Arc>,
    pub info: Vec<(Vec<usize>, Aabb)>,
    // pub(crate) _min_width: f32,
    // pub(crate) _min_height: f32,
}

impl Serialize for CellInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let unit_size = self.extents.width() / 32.0;
        let start_point = self.extents.mins;

        let mut s = serializer.serialize_struct("CellInfo", 6)?;
        s.serialize_field("mins_x", &self.extents.mins.x)?;
        s.serialize_field("mins_y", &self.extents.mins.y)?;
        s.serialize_field("maxs_x", &self.extents.maxs.x)?;
        s.serialize_field("maxs_y", &self.extents.maxs.y)?;
        s.serialize_field("arcs", &self.arcs)?;

        let mut info = Vec::with_capacity(self.info.len());
        for (arcs, ab) in &self.info {
            let offset = (ab.mins - start_point) / unit_size;
            let w = ab.width() / unit_size;
            let h = ab.height() / unit_size;
            let mut temp = Vec::with_capacity(arcs.len());
            for arc in arcs {
                temp.push(*arc as u8);
            }

            info.push((
                temp,
                offset.x.round() as u8,
                offset.y.round() as u8,
                w.round() as u8,
                h.round() as u8,
            ));
        }
        // println!("CellInfo: {}", info.len() )
        s.serialize_field("info", &info)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for CellInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CellInfoVisitor;

        impl<'de> Visitor<'de> for CellInfoVisitor {
            type Value = CellInfo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CellInfo")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<CellInfo, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mins_x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mins_y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let maxs_x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let maxs_y1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let arcs = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let src_info: Vec<(Vec<u8>, u8, u8, u8, u8)> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let extents = Aabb::new(Point::new(mins_x, mins_y), Point::new(maxs_x, maxs_y1));
                let unit_size = extents.width() / 32.0;

                let mut info = Vec::with_capacity(src_info.len());
                for (indexs, offset_x, offset_y, w, h) in src_info {
                    let min = Point::new(
                        extents.mins.x + unit_size * offset_x as f32,
                        extents.mins.y + unit_size * offset_y as f32,
                    );
                    let max =
                        Point::new(min.x + unit_size * w as f32, min.y + unit_size * h as f32);
                    let mut arc_index = Vec::with_capacity(indexs.len());
                    for i in indexs {
                        arc_index.push(i as usize);
                    }
                    info.push((arc_index, Aabb::new(min, max)));
                }
                Ok(CellInfo {
                    extents,
                    arcs,
                    info,
                    // min_width: 0.0,
                    // min_height: 0.0,
                })
            }
        }

        const FIELDS: &'static [&'static str] =
            &["arcs", "mins_x", "mins_y", "maxs_x", "maxs_y", "info"];
        deserializer.deserialize_struct("Point", FIELDS, CellInfoVisitor)
    }
}
