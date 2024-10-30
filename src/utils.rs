use core::fmt;
use std::collections::HashMap;
// use std::collections::BTreeMap as HashMap;

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
    glyphy::{
        blob::{travel_data, BlobArc},
        geometry::{aabb::Aabb, arc::ID, arcs::GlyphyArcAccumulator},
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TexInfo2 {
    pub sdf_offset_x: usize,
    pub sdf_offset_y: usize,
    pub advance: f32,
    pub char: char,
    pub plane_min_x: f32,
    pub plane_min_y: f32,
    pub plane_max_x: f32,
    pub plane_max_y: f32,
    pub atlas_min_x: f32,
    pub atlas_min_y: f32,
    pub atlas_max_x: f32,
    pub atlas_max_y: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SdfInfo2 {
    pub tex_info: TexInfo2,
    pub sdf_tex: Vec<u8>,
    pub tex_size: u32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineInfo {
    pub(crate) char: char,
    pub(crate) endpoints: Vec<ArcEndpoint>,
    pub bbox: Vec<f32>,
    pub advance: u16,
    pub units_per_em: u16,
    pub extents: Vec<f32>,
}

impl OutlineInfo {
    pub fn compute_near_arcs(&self, scale: f32) -> CellInfo {
        FontFace::compute_near_arcs(
            Aabb::new(
                Point::new(self.extents[0], self.extents[1]),
                Point::new(self.extents[2], self.extents[3]),
            ),
            scale,
            &self.endpoints
        )
    }

    pub fn compute_layout(&self, tex_size: usize, pxrange: u32, cur_off: u32) -> LayoutInfo {
        compute_layout(
            &self.extents,
            tex_size,
            pxrange,
            self.units_per_em,
            cur_off,
            false,
        )
    }

    pub fn compute_sdf_tex(
        &self,
        result_arcs: CellInfo,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
    ) -> SdfInfo2 {
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = self.compute_layout(tex_size, pxrange, cur_off);
        let extents = Aabb::new(
            Point::new(extents[0], extents[1]),
            Point::new(extents[2], extents[3]),
        );
        let CellInfo { arcs, info, .. } = result_arcs;
        let pixmap = encode_sdf(
            &arcs,
            info,
            &extents,
            tex_size as usize,
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
                plane_min_x: plane_bounds[0],
                plane_min_y: plane_bounds[1],
                plane_max_x: plane_bounds[2],
                plane_max_y: plane_bounds[3],
                atlas_min_x: atlas_bounds[0],
                atlas_min_y: atlas_bounds[1],
                atlas_max_x: atlas_bounds[2],
                atlas_max_y: atlas_bounds[3],
            },
            sdf_tex: pixmap,
            tex_size,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl OutlineInfo {
    pub fn compute_near_arcs_of_wasm(outline: &[u8], scale: f32) -> Vec<u8> {
        let outline: OutlineInfo = bitcode::deserialize(outline).unwrap();
        bitcode::serialize(&outline.compute_near_arcs(scale)).unwrap()
    }

    pub fn compute_sdf_tex_of_wasm(
        result_arcs: &[u8],
        extents: &[f32],
        units_per_em: u16,
        advance: u16,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
    ) -> Vec<u8> {
        let result_arcs: CellInfo = bitcode::deserialize(result_arcs).unwrap();
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = compute_layout(extents, tex_size, pxrange, units_per_em, cur_off, false);
        let extents = Aabb::new(
            Point::new(extents[0], extents[1]),
            Point::new(extents[2], extents[3]),
        );
        let CellInfo { arcs, info, .. } = result_arcs;
        let pixmap = encode_sdf(
            &arcs,
            info,
            &extents,
            tex_size as usize,
            distance,
            None,
            is_outer_glow,
            false,
            None,
        );

        bitcode::serialize(&SdfInfo2 {
            tex_info: TexInfo2 {
                char: ' ',
                advance: advance as f32 / units_per_em as f32,
                sdf_offset_x: 0,
                sdf_offset_y: 0,
                plane_min_x: plane_bounds[0],
                plane_min_y: plane_bounds[1],
                plane_max_x: plane_bounds[2],
                plane_max_y: plane_bounds[3],
                atlas_min_x: atlas_bounds[0],
                atlas_min_y: atlas_bounds[1],
                atlas_max_x: atlas_bounds[2],
                atlas_max_y: atlas_bounds[3],
            },
            sdf_tex: pixmap,
            tex_size,
        })
        .unwrap()
    }

    pub fn compute_layout_of_wasm(
        extents: &[f32],
        units_per_em: u16,
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
        } = compute_layout(extents, tex_size, pxrange, units_per_em, cur_off, false);
        let mut res = Vec::with_capacity(14);
        res.append(&mut plane_bounds);
        res.append(&mut atlas_bounds);
        res.append(&mut extents);
        res.push(distance);
        res.push(tex_size as f32);
        res
    }
}
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

pub trait OutlineSinkExt: OutlineSink {
    fn arc2_to(&mut self, d: f32, to: Vector2F);
}

impl OutlineSinkExt for GlyphVisitor {
    fn arc2_to(&mut self, d: f32, to: Vector2F) {
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::debug!("+ A {} {} ", to.x, to.y);
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
        log::debug!("M {} {} ", to.x, to.y);

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
        log::debug!("+ L {} {} ", to.x, to.y);
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

        log::debug!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
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

        log::debug!(
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
            log::debug!("+ L {} {} ", self.start.x, self.start.y);
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
        log::debug!("+ Z");
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
        // log::debug!("{}", s);
        self.index = self.accumulate.result.len();
        // log::debug!("close()");
    }
}

pub fn encode_sdf(
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
            begin_x = (begin_x * 10000.0).round() * 0.0001;
            let begin_x = begin_x.round() as usize;

            let mut begin_y = begin.y / unit_d;
            begin_y = (begin_y * 10000.0).round() * 0.0001;
            let begin_y = begin_y.round() as usize;

            let mut end_x = end.x / unit_d;
            end_x = (end_x * 10000.0).round() * 0.0001;
            let end_x = end_x.round() as usize;

            let mut end_y = end.y / unit_d;
            end_y = (end_y * 10000.0).round() * 0.0001;
            let end_y = end_y.round() as usize;
            // log::debug!("{:?}", (begin_x, begin_y, end_x, end_y));
            // If the arclist is two arcs that can be combined in encoding if reordered, do that.
            for i in begin_x..end_x {
                for j in begin_y..end_y {
                    let p = Point::new(
                        (i as f32 + 0.5) * unit_d + extents.mins.x,
                        (j as f32 + 0.5) * unit_d + extents.mins.y,
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
                    // if j == 6 && (i == 7 || i == 6) {
                    //     log::debug!("p: {}, i: {}, j: {}", p, i, j);
                    //     log::debug!("============== cell: {:?}, extents: {:?}, ab: {:?}, unit_d: {:?}", cell, extents, ab, unit_d);
                    //     log::debug!("begin: {}, end: {}", begin.y / unit_d, end.y / unit_d);
                    //     log::debug!("sdf: {:?}", r);
                    //     for a in &near_arcs {
                    //         log::debug!("{:?}", global_arcs[*a])
                    //     }
                    // }

                    // svg 不需要颠倒纹理
                    if is_svg {
                        data[j * tex_size + i] = r.0;
                    } else {
                        // log::debug!("{:?}", (r, j, i));
                        data[(tex_size - j - 1) * tex_size + i] = r.0;
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
) -> (u8, f32, f32) {
    let mut sdf = glyphy_sdf_from_arc_list3(near_arcs, p.clone(), global_arcs).0;
    // 去除浮点误差
    sdf = (sdf * 10000.0).round() * 0.0001;
    // let p2 = Point::new(85.0, 82.0) - p;
    // if p2.norm_squared() < 0.1{
    //     log::debug!("p : {:?}", (p, sdf, distance));
    //     for i in near_arcs{
    //         log::debug!("{:?}", global_arcs[*i]);
    //     }
    // }
    // let p2 = Point::new(85.5, 84.5) - p;
    // if p2.norm_squared() < 0.1 {
    //     log::debug!("p : {:?}", (p, sdf, distance));
    //     for i in near_arcs {
    //         log::debug!("{:?}", global_arcs[*i]);
    //     }
    // }
    // let p2 = Point::new(85.5, 85.5) - p;
    // if p2.norm_squared() < 0.1 {
    //     log::debug!("p : {:?}", (p, sdf, distance));
    //     for i in near_arcs {
    //         log::debug!("{:?}", global_arcs[*i]);
    //     }
    // }
    if let Some(is_reverse) = is_reverse {
        if is_reverse {
            sdf = -sdf;
        }
    }
    if let Some(_) = width {
        sdf = sdf.abs(); // - (width * 0.5);
    }

    if is_outer_glow {
        let sdf2 = (1.0 - (sdf / distance)).powf(1.99);
        return ((sdf2 * 255.0).round() as u8, sdf, sdf2);
        // log::debug!("{:?}", (radius, sdf));
    } else {
        let sdf2 = sdf / distance;
        let a = ((1.0 - sdf2) * 127.0).round() as u8;
        // log::debug!("sdf: {:?}", (sdf, a, distance));
        return (a, sdf, sdf2);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayoutInfo {
    pub plane_bounds: Vec<f32>,
    pub atlas_bounds: Vec<f32>,
    pub extents: Vec<f32>,
    pub distance: f32,
    pub tex_size: u32,
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub(crate) fn compute_layout(
    extents: &[f32],
    tex_size: usize,
    pxrange: u32,
    units_per_em: u16,
    cur_off: u32,
    is_svg: bool,
) -> LayoutInfo {
    // map 无序导致每次计算的数据不一样
    let mut extents2 = Aabb::new(
        Point::new(extents[0], extents[1]),
        Point::new(extents[2], extents[3]),
    );
    let extents_w = extents2.width();
    let extents_h = extents2.height();
    let scale = 1.0 / units_per_em as f32;
    let plane_bounds = extents2.scaled(&Vector::new(scale, scale));

    let px_distance = extents_w.max(extents_h) / tex_size as f32;
    let distance = px_distance * pxrange as f32;
    let expand = px_distance * cur_off as f32;
    // log::debug!("distance: {}", distance);
    extents2.mins.x -= expand;
    extents2.mins.y -= expand;
    extents2.maxs.x += expand;
    extents2.maxs.y += expand;

    // let pxrange = (pxrange >> 2 << 2) + 4;
    let tex_size = tex_size + (cur_off * 2) as usize;
    let mut atlas_bounds = Aabb::new_invalid();
    atlas_bounds.mins.x = cur_off as f32;
    atlas_bounds.mins.y = cur_off as f32;
    atlas_bounds.maxs.x = tex_size as f32 - cur_off as f32;
    atlas_bounds.maxs.y = tex_size as f32 - cur_off as f32;

    let temp = extents_w - extents_h;
    if temp > 0.0 {
        extents2.maxs.y += temp;
        if is_svg {
            // log::debug!("============= is_svg: {}", (temp / extents.height() * tex_size as f32 - 1.0));
            atlas_bounds.maxs.y -= (temp / extents2.height() * tex_size as f32).trunc();
        } else {
            // 字体的y最终需要上下颠倒
            atlas_bounds.mins.y += (temp / extents2.height() * tex_size as f32).ceil();
        }
    } else {
        extents2.maxs.x -= temp;
        atlas_bounds.maxs.x -= (temp.abs() / extents2.width() * tex_size as f32).trunc();
    }
    // plane_bounds.scale(
    //     atlas_bounds.width() / 32.0 / plane_bounds.width(),
    //     atlas_bounds.height() / 32.0 / plane_bounds.width(),
    // );

    log::debug!(
        "plane_bounds: {:?}, atlas_bounds: {:?}, tex_size: {}",
        plane_bounds, atlas_bounds, tex_size
    );

    LayoutInfo {
        plane_bounds: vec![
            plane_bounds.mins.x,
            plane_bounds.mins.y,
            plane_bounds.maxs.x,
            plane_bounds.maxs.y,
        ],
        atlas_bounds: vec![
            atlas_bounds.mins.x,
            atlas_bounds.mins.y,
            atlas_bounds.maxs.x,
            atlas_bounds.maxs.y,
        ],
        extents: vec![
            extents2.mins.x,
            extents2.mins.y,
            extents2.maxs.x,
            extents2.maxs.y,
        ],
        distance,
        tex_size: tex_size as u32,
    }
}

pub fn compute_cell_range(mut bbox: Aabb, scale: f32) -> Aabb {
    let scale = scale * 0.5;
    let w = bbox.width();
    let h = bbox.height();

    let temp = w - h;
    if temp > 0.0 {
        bbox.maxs.y += temp;
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

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone)]
pub struct CellInfo {
    pub extents: Aabb,
    pub(crate) arcs: Vec<Arc>,
    pub(crate) info: Vec<(Vec<usize>, Aabb)>,
    pub(crate) min_width: f32,
    pub(crate) min_height: f32,
    pub(crate) is_area: bool,
}

impl CellInfo {
    pub fn encode_blob_arc(&self) -> BlobArc {
        let extents = &self.extents;

        let result_arcs = &self.info;
        let global_arcs = &self.arcs;
        let glyph_width = extents.width();
        let glyph_height = extents.height();
        // // 格子列的数量;
        // // todo 为了兼容阴影minimip先强制索引纹理为32 * 32
        let width_cells = (glyph_width / self.min_width).round() as usize;
        // 格子行的数量
        let height_cells = (glyph_height / self.min_height).round() as usize;

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

        let mut map = HashMap::new();
        // 二分计算时，个格子的大小会不一样
        // 统一以最小格子细分
        for (near_arcs, cell) in result_arcs {
            let mut near_endpoints = Vec::with_capacity(8);
            let mut _p1 = Point::new(0.0, 0.0);

            for i in 0..near_arcs.len() {
                let arc = &global_arcs[near_arcs[i]];

                if i == 0 || !_p1.equals(&arc.p0) {
                    let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
                    near_endpoints.push(endpoint);
                    _p1 = arc.p0;
                }

                let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
                near_endpoints.push(endpoint);
                _p1 = arc.p1;
            }
            // log::debug!("near_endpoints: {:?}", near_endpoints.len());

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
                        // log::debug!("1row: {}, col: {} line_data: {:?}n \n", row, col, unit_arc.data.len());
                        unit_arc.origin_data.push(start.clone());
                        unit_arc.origin_data.push(end.clone());
                        unit_arc.parent_cell = parent_cell;
                    } else {
                        let key = arc_result.as_ref().unwrap();
                        unit_arc.data.extend_from_slice(&near_endpoints);
                        unit_arc.parent_cell = parent_cell;
                        unit_arc.key = key.clone();
                    }
                    // log::debug!("i: {}, j: {}, unit_arc: {:?}", i, j, unit_arc);
                }
            }
            let key = data[begin_y][begin_x].get_key();
            let ptr: *const UnitArc = &data[begin_y][begin_x];
            // 使用map 去重每个格子的数据纹理
            map.insert(key, ptr as u64);
        }
        let [min_sdf, max_sdf] = travel_data(&data);

        BlobArc {
            min_sdf,
            max_sdf,
            cell_size: min_width,
            #[cfg(feature = "debug")]
            show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
            extents: *extents,
            data: data,
            avg_fetch_achieved: 0.0,
            endpoints: vec![],
            data_tex_map: map,
        }
    }
}

impl Serialize for CellInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let unit_size = self.extents.width() / 32.0;
        let start_point = self.extents.mins;

        let mut s = serializer.serialize_struct("CellInfo", 9)?;
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
                temp.push(*arc as u16);
            }

            info.push((
                temp,
                offset.x.round() as u8,
                offset.y.round() as u8,
                w.round() as u8,
                h.round() as u8,
            ));
        }
        // log::debug!("CellInfo: {}", info.len() )
        s.serialize_field("info", &info)?;
        s.serialize_field("min_width", &self.min_width)?;
        s.serialize_field("min_height", &self.min_height)?;
        s.serialize_field("is_area", &self.is_area)?;
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
                let src_info: Vec<(Vec<u16>, u8, u8, u8, u8)> = seq
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
                let min_width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let min_height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let is_area = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                Ok(CellInfo {
                    extents,
                    arcs,
                    info,
                    min_width,
                    min_height,
                    is_area,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "mins_x",
            "mins_y",
            "maxs_x",
            "maxs_y",
            "arcs",
            "info",
            "min_width",
            "min_height",
            "is_area",
        ];
        deserializer.deserialize_struct("Point", FIELDS, CellInfoVisitor)
    }
}
