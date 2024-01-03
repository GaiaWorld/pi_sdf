use std::{collections::HashMap, io};

use allsorts::{
    cff::outline,
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use parry2d::bounding_volume::Aabb;
use svg::{
    node::element::path::{Command, Data, Position},
    parser::Event,
};
use usvg::{tiny_skia_path::PathVerb, Fill, NodeKind, Stroke, TreeParsing};

use crate::{
    glyphy::{
        blob::{recursion_near_arcs_of_cell, travel_data, BlobArc, EncodeError, TexData, TexInfo},
        geometry::{
            aabb::{AabbEXT, Direction},
            arc::{Arc, ArcEndpoint},
            arcs::glyphy_arc_list_extents,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::GLYPHY_INFINITY,
        vertex::GlyphInfo,
    },
    utils::{encode_uint_arc_data, GlyphVisitor, EMBOLDEN_MAX, MIN_FONT_SIZE, TOLERANCE},
    Point,
};

pub struct Svg {
    content: String,
}

impl Svg {
    pub fn form_path(path: &str) -> io::Result<Self> {
        let mut content = String::new();
        let _ = svg::open(path, &mut content)?;
        Ok(Self { content })
    }

    pub fn form_content(content: String) -> Self {
        Self { content }
    }

    pub fn get_svg_arc(&self, gi: &mut GlyphInfo, per_em: Option<f32>) -> BlobArc {
        // log::error!("get_char_arc: {:?}", char);
        let tolerance_per_em = if let Some(v) = per_em { v } else { TOLERANCE };

        let upem = 2048 as f32;
        let tolerance = upem * tolerance_per_em; /* in font design units */
        let faraway = upem / (MIN_FONT_SIZE * 2.0f32.sqrt());
        let embolden_max = upem * EMBOLDEN_MAX;

        let mut sink = GlyphVisitor::new(1.0);
        sink.accumulate.tolerance = tolerance;

        self.to_outline(&mut sink);

        let endpoints = &mut sink.accumulate.result;

        if endpoints.len() > 0 {
            // 用奇偶规则，计算 每个圆弧的 环绕数
            glyphy_outline_winding_from_even_odd(endpoints, false);
        }

        let mut extents = Aabb::new(
            Point::new(f32::INFINITY, f32::INFINITY),
            Point::new(f32::INFINITY, f32::INFINITY),
        );

        glyphy_arc_list_extents(&endpoints, &mut extents);

        let mut min_width = f32::INFINITY;
        let mut min_height = f32::INFINITY;

        let mut p0 = Point::new(0., 0.);
        // 添加 抗锯齿的 空隙
        extents.mins.x -= faraway + embolden_max;
        extents.mins.y -= faraway + embolden_max;
        extents.maxs.x += faraway + embolden_max;
        extents.maxs.y += faraway + embolden_max;

        let glyph_width = extents.maxs.x - extents.mins.x;
        let glyph_height = extents.maxs.y - extents.mins.y;
        if glyph_width > glyph_height {
            extents.maxs.y = extents.mins.y + glyph_width;
        } else {
            extents.maxs.y = extents.mins.x + glyph_height;
        };

        let mut near_arcs = Vec::with_capacity(endpoints.len());
        let mut arcs = Vec::with_capacity(endpoints.len());
        for i in 0..endpoints.len() {
            let endpoint = &endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = endpoint.p;
                continue;
            }
            let arc = Arc::new(p0, endpoint.p, endpoint.d);
            p0 = endpoint.p;

            near_arcs.push(arc);
            arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
        }

        let mut result_arcs = vec![];
        let mut temp = Vec::with_capacity(arcs.len());
        let (ab1, ab2) = extents.half(Direction::Col);
        recursion_near_arcs_of_cell(
            &extents,
            &ab1,
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
        recursion_near_arcs_of_cell(
            &extents,
            &ab2,
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

        let width_cells = (extents.width() / min_width).floor();
        let height_cells = (extents.height() / min_height).floor();

        let (unit_arcs, set) = encode_uint_arc_data(result_arcs, &extents, min_width, min_height);

        let [min_sdf, max_sdf] = travel_data(&unit_arcs);
        let blob_arc = BlobArc {
            min_sdf,
            max_sdf,
            cell_size: min_width,
            show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
            extents,
            data: unit_arcs,
            avg_fetch_achieved: 0.0,
            endpoints: endpoints.clone(),
        };

        extents.scale(1.0 / upem, 1.0 / upem);

        gi.nominal_w = width_cells;
        gi.nominal_h = height_cells;

        gi.extents.set(&extents);

        blob_arc
    }

    fn to_outline(&self, visitor: &mut impl OutlineSink) {
        for event in svg::read(&self.content).unwrap() {
            match event {
                Event::Tag(_, _, attributes) => {
                    for v in attributes.values() {
                        if let Ok(data) = Data::parse(v) {
                            for command in data.iter() {
                                // println!("command: {:?}", command);
                                let mut relative_pos = Vector2F::default();
                                match &command {
                                    &Command::Move(pos, p) => {
                                        // println!("M: {:?}, p: {:?}", pos, p);
                                        let p = Vector2F::new(p[0], p[1]);
                                        if pos == &Position::Relative {
                                            relative_pos = p + relative_pos;
                                            visitor.move_to(relative_pos)
                                        } else {
                                            visitor.move_to(p);
                                        }
                                    }
                                    &Command::Line(pos, p) => {
                                        let p = Vector2F::new(p[0], p[1]);
                                        if pos == &Position::Relative {
                                            relative_pos = p + relative_pos;
                                            visitor.line_to(relative_pos)
                                        } else {
                                            visitor.line_to(p);
                                        }
                                    }
                                    &Command::CubicCurve(pos, p) => {
                                        let line_segment = LineSegment2F::new(
                                            Vector2F::new(p[0], p[1]),
                                            Vector2F::new(p[2], p[3]),
                                        );
                                        let p = Vector2F::new(p[4], p[5]);

                                        if pos == &Position::Relative {
                                            relative_pos = p + relative_pos;
                                            visitor.cubic_curve_to(line_segment, relative_pos);
                                        } else {
                                            visitor.cubic_curve_to(line_segment, relative_pos);
                                        }
                                    }
                                    &Command::QuadraticCurve(pos, p) => {
                                        let ctrl = Vector2F::new(p[0], p[1]);
                                        let p = Vector2F::new(p[2], p[3]);

                                        if pos == &Position::Relative {
                                            relative_pos = p + relative_pos;
                                            visitor.quadratic_curve_to(ctrl, relative_pos)
                                        } else {
                                            visitor.quadratic_curve_to(ctrl, p);
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            visitor.close();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PathType {
    Close,
    Move,
    Line,
    Quad(Vector2F),
    Cubic(LineSegment2F),
}

impl PathType {
    pub fn outline(&self, sink: &mut impl OutlineSink, to: Vector2F) {
        match self {
            PathType::Close => sink.move_to(to),
            PathType::Move => {}
            PathType::Line => sink.line_to(to),
            PathType::Quad(ctrl) => {
                sink.quadratic_curve_to(*ctrl, to);
            }
            PathType::Cubic(ctrl) => {
                sink.cubic_curve_to(*ctrl, to);
            }
        }
    }
}

pub struct Attribute {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

pub struct Svg2 {
    pub(crate) tree: usvg::Tree,
    pub(crate) view_box: Aabb,
}

impl Svg2 {
    pub fn new(data: Vec<u8>) -> Self {
        let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).unwrap();

        let view_box = tree.view_box;
        let mut left = view_box.rect.left();
        let mut right = view_box.rect.right();

        let mut top = view_box.rect.top();
        let mut bottom = view_box.rect.bottom();

        let width = (right - left).abs();
        let height = (bottom - top).abs();
        if width > height {
            bottom += width - height;
        } else {
            right += height - width;
        }
        left -= 10.0;
        top -= 10.0;

        right += 10.0;
        bottom += 10.0;

        Self {
            tree,
            view_box: Aabb {
                mins: Point::new(left, top),
                maxs: Point::new(right, bottom),
            },
        }
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

    pub fn compute_near_arc(
        &mut self,
        endpoints: Vec<ArcEndpoint>,
    ) -> (BlobArc, HashMap<String, u64>) {
        let extents = self.view_box;
        // println!("extents: {:?}", extents);

        let mut min_width = f32::INFINITY;
        let mut min_height = f32::INFINITY;

        let mut p0 = Point::new(0., 0.);

        // 添加 抗锯齿的 空隙

        // println!("extents2: {:?}", extents);
        let mut near_arcs = Vec::with_capacity(endpoints.len());
        let mut arcs = Vec::with_capacity(endpoints.len());
        for i in 0..endpoints.len() {
            let endpoint = &endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = endpoint.p;
                continue;
            }
            let arc = Arc::new(p0, endpoint.p, endpoint.d);
            p0 = endpoint.p;

            near_arcs.push(arc);
            arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
        }

        let mut result_arcs = vec![];
        let mut temp = Vec::with_capacity(arcs.len());
        // println!("arcs:{:?}", arcs.l);
        let (ab1, ab2) = extents.half(Direction::Col);
        recursion_near_arcs_of_cell(
            &extents,
            &ab1,
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
        recursion_near_arcs_of_cell(
            &extents,
            &ab2,
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

        let (unit_arcs, map) = encode_uint_arc_data(result_arcs, &extents, min_width, min_height);

        let [min_sdf, max_sdf] = travel_data(&unit_arcs);
        let blob_arc = BlobArc {
            min_sdf,
            max_sdf,
            cell_size: min_width,
            show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
            extents,
            data: unit_arcs,
            avg_fetch_achieved: 0.0,
            endpoints: endpoints.clone(),
        };

        (blob_arc, map)
    }

    pub fn out_tex_data(&mut self, tex_data: &mut TexData) -> Result<(Vec<TexInfo>, Vec<Attribute>), EncodeError> {
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

        let root = &self.tree.root;
        for node in root.children() {
            match node.borrow().clone() {
                NodeKind::Group(_) => println!("Group"),
                NodeKind::Path(path) => {
                    // println!("data: {:?}", path.data);
                    println!("fill: {:?}", path.fill);
                    println!("stroke: {:?}", path.stroke);

                    let mut sink = GlyphVisitor::new(1.0);
                    sink.accumulate.tolerance =
                        path.data.bounds().height().min(path.data.bounds().width()) * TOLERANCE;
                    let mut points = path.data.points().iter().rev();

                    let mut last_path = PathType::Close;
                    for p in path.data.verbs().iter().rev() {
                        match p {
                            PathVerb::Move => {
                                let to = points.next().unwrap();
                                let to = Vector2F::new(to.x, to.y);

                                last_path.outline(&mut sink, to);
                                last_path = PathType::Move;
                            }
                            PathVerb::Line => {
                                let to = points.next().unwrap();
                                let to = Vector2F::new(to.x, to.y);

                                last_path.outline(&mut sink, to);
                                last_path = PathType::Line;
                            }
                            PathVerb::Quad => {
                                let to = points.next().unwrap();
                                let to = Vector2F::new(to.x, to.y);

                                let ctrl = points.next().unwrap();
                                let ctrl = Vector2F::new(ctrl.x, ctrl.y);

                                last_path.outline(&mut sink, to);
                                last_path = PathType::Quad(ctrl);
                            }
                            PathVerb::Cubic => {
                                let to = points.next().unwrap();
                                let to = Vector2F::new(to.x, to.y);

                                let ctrl1 = points.next().unwrap();
                                let ctrl1 = Vector2F::new(ctrl1.x, ctrl1.y);

                                let ctrl2 = points.next().unwrap();
                                let ctrl2 = Vector2F::new(ctrl2.x, ctrl2.y);

                                let ctrl = LineSegment2F::new(ctrl1, ctrl2);
                                last_path.outline(&mut sink, to);
                                last_path = PathType::Cubic(ctrl);
                            }
                            PathVerb::Close => {
                                last_path = PathType::Close;
                            }
                        }
                    }
                    sink.close();
                    attributes.push(Attribute { fill: path.fill, stroke: path.stroke });
                    assert_eq!(points.next(), None);
                    let (mut blob_arc, map) = self.compute_near_arc(sink.accumulate.result);
                    let size =
                        blob_arc.encode_data_tex(&map, data_tex, width0, offset_x0, offset_y0)?;
                    // println!("data_map: {}", map.len());
                    let mut info = blob_arc
                        .encode_index_tex(index_tex, width1, offset_x1, offset_y1, map, size)?;

                    info.index_offset = last_offset1;
                    info.data_offset = (*offset_x0, *offset_y0);

                    *offset_x0 += size / 8;
                    if size % 8 != 0 {
                        *offset_x0 += 1;
                    }

                    last_offset1 = (*offset_x1, *offset_y1);

                    infos.push(info);
                }
                NodeKind::Image(_) => println!("Image"),
                NodeKind::Text(_) => println!("Text"),
            }
        }

        Ok((infos, attributes))
    }
}
