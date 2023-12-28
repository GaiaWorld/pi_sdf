use std::io;

use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use parry2d::{bounding_volume::Aabb, math::Point};
use svg::{
    node::element::path::{Command, Data, Position},
    parser::Event,
};

use crate::{
    glyphy::{
        blob::{recursion_near_arcs_of_cell, travel_data, BlobArc},
        geometry::{
            aabb::{AabbEXT, Direction},
            arc::Arc,
            arcs::glyphy_arc_list_extents,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::GLYPHY_INFINITY,
        vertex::GlyphInfo,
    },
    utils::{encode_uint_arc_data, GlyphVisitor, EMBOLDEN_MAX, MIN_FONT_SIZE, TOLERANCE},
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

        let unit_arcs = encode_uint_arc_data(result_arcs, &extents, min_width, min_height);

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
