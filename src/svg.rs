use std::collections::HashMap;

use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use parry2d::bounding_volume::Aabb;

use usvg::{
    // tiny_skia_path::{self, PathVerb},
    NodeKind, PathCommand, TreeParsing
};

use crate::{
    glyphy::{
        blob::{recursion_near_arcs_of_cell, travel_data, BlobArc, EncodeError, TexData, TexInfo},
        geometry::{aabb::AabbEXT, arc::{Arc, ArcEndpoint}},
        util::GLYPHY_INFINITY,
    }, shape::PathVerb, utils::{encode_uint_arc_data, Attribute, GlyphVisitor}, Point
};

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

pub struct Svg {
    pub(crate) tree: usvg::Tree,
    pub(crate) view_box: Aabb,
}

impl Svg {
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
                mins: Point::new(left as f32, top as f32),
                maxs: Point::new(right as f32, bottom as f32),
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

    pub fn compute_near_arc(&self, endpoints: Vec<ArcEndpoint>, is_area: bool) -> (BlobArc, HashMap<u64, u64>) {
        compute_near_arc_impl(self.view_box, endpoints, is_area)
    }

    // pub fn out_tex_data(
    //     &mut self,
    //     tex_data: &mut TexData,
    // ) -> Result<(Vec<TexInfo>, Vec<Attribute>), EncodeError> {
    //     let mut infos = vec![];
    //     let mut attributes = vec![];

    //     let data_tex = &mut tex_data.data_tex;
    //     let width0 = tex_data.data_tex_width;
    //     let offset_x0 = &mut tex_data.data_offset_x;
    //     let offset_y0 = &mut tex_data.data_offset_y;

    //     let index_tex = &mut tex_data.index_tex;
    //     let width1 = tex_data.index_tex_width;
    //     let offset_x1 = &mut tex_data.index_offset_x;
    //     let offset_y1 = &mut tex_data.index_offset_y;
    //     let mut last_offset1 = (*offset_x1, *offset_x1);

    //     let sdf_tex = &mut tex_data.sdf_tex;
    //     let sdf_tex1 = &mut tex_data.sdf_tex1;
    //     let sdf_tex2 = &mut tex_data.sdf_tex2;
    //     let sdf_tex3 = &mut tex_data.sdf_tex3;

    //     let root = &self.tree.root;
    //     for node in root.children() {
    //         match node.borrow().clone() {
    //             NodeKind::Group(_) => println!("Group"),
    //             NodeKind::Path(path) => {
    //                 // println!("data: {:?}", path);
    //                 println!("fill: {:?}", path.fill);
    //                 println!("stroke: {:?}", path.stroke);
    //                 let mut sink = GlyphVisitor::new(1.0, 1.0);
    //                 // 圆弧拟合贝塞尔曲线的精度，值越小越精确
    //                 sink.accumulate.tolerance = 0.1;
    //                 let is_close = compute_outline(
    //                     path.data.points().iter().rev(),
    //                     path.data.commands().iter().rev(),
    //                     &mut sink,
    //                 );
    //                 if !is_close {
    //                     for s in path.data.segments() {
    //                         match s {
    //                             tiny_skia_path::PathSegment::MoveTo(_) => {}
    //                             tiny_skia_path::PathSegment::LineTo(to) => {
    //                                 sink.line_to(Vector2F::new(to.x, to.y))
    //                             }
    //                             tiny_skia_path::PathSegment::QuadTo(c, to) => sink
    //                                 .quadratic_curve_to(
    //                                     Vector2F::new(c.x, c.y),
    //                                     Vector2F::new(to.x, to.y),
    //                                 ),
    //                             tiny_skia_path::PathSegment::CubicTo(c1, c2, to) => sink
    //                                 .cubic_curve_to(
    //                                     LineSegment2F::new(
    //                                         Vector2F::new(c1.x, c1.y),
    //                                         Vector2F::new(c2.x, c2.y),
    //                                     ),
    //                                     Vector2F::new(to.x, to.y),
    //                                 ),
    //                             tiny_skia_path::PathSegment::Close => {}
    //                         }
    //                     }
    //                 } else {
    //                     sink.close();
    //                 }
    //                 let p = path.data.points().first().unwrap();
    //                 attributes.push(Attribute {
    //                     fill: path.fill,
    //                     stroke: path.stroke,
    //                     is_close,
    //                     start: Point::new(p.x, p.y),
    //                 });

    //                 let (mut blob_arc, map) = self.compute_near_arc(sink.accumulate.result,false);
    //                 let size =
    //                     blob_arc.encode_data_tex(&map, data_tex, width0, offset_x0, offset_y0)?;
    //                 // println!("data_map: {}", map.len());
    //                 let mut info = blob_arc.encode_index_tex(
    //                     index_tex, width1, offset_x1, offset_y1, map, size, sdf_tex, sdf_tex1,
    //                     sdf_tex2, sdf_tex3,
    //                 )?;

    //                 info.index_offset_x = last_offset1.0;
    //                 info.index_offset_y = last_offset1.1;
    //                 info.data_offset_x = *offset_x0;
    //                 info.data_offset_y = *offset_y0;
    //                 // println!(
    //                 //     "info.index_offset: {:?}, info.data_offset: {:?}",
    //                 //     (info.index_offset_x, info.index_offset_y), (info.data_offset_x, info.data_offset_y)
    //                 // );
    //                 *offset_x0 += size / 8;
    //                 if size % 8 != 0 {
    //                     *offset_x0 += 1;
    //                 }

    //                 last_offset1 = (*offset_x1, *offset_y1);

    //                 infos.push(info);
    //             }
    //             NodeKind::Image(_) => println!("Image"),
    //             NodeKind::Text(_) => println!("Text"),
    //         }
    //     }

    //     Ok((infos, attributes))
    // }
}

// fn compute_outline<'a>(
//     mut points: impl Iterator<Item = &'a f64>,
//     verbs: impl Iterator<Item = &'a PathCommand>,
//     sink: &mut impl OutlineSink,
// ) -> bool {
//     let mut last_path = PathType::Close;
//     let mut is_colse = false;
//     for p in verbs {
//         match p {
//             PathVerb::Move => {
//                 let to = points.next().unwrap();
//                 let to = Vector2F::new(to.x, to.y);

//                 last_path.outline(sink, to);
//                 last_path = PathType::Move;
//             }
//             PathVerb::Line => {
//                 let to = points.next().unwrap();
//                 let to = Vector2F::new(to.x, to.y);

//                 last_path.outline(sink, to);
//                 last_path = PathType::Line;
//             }
//             PathVerb::Quad => {
//                 let to = points.next().unwrap();
//                 let to = Vector2F::new(to.x, to.y);

//                 let ctrl = points.next().unwrap();
//                 let ctrl = Vector2F::new(ctrl.x, ctrl.y);

//                 last_path.outline(sink, to);
//                 last_path = PathType::Quad(ctrl);
//             }
//             PathVerb::Cubic => {
//                 let to = points.next().unwrap();
//                 let to = Vector2F::new(to.x, to.y);

//                 let ctrl1 = points.next().unwrap();
//                 let ctrl1 = Vector2F::new(ctrl1.x, ctrl1.y);

//                 let ctrl2 = points.next().unwrap();
//                 let ctrl2 = Vector2F::new(ctrl2.x, ctrl2.y);

//                 let ctrl = LineSegment2F::new(ctrl1, ctrl2);
//                 last_path.outline(sink, to);
//                 last_path = PathType::Cubic(ctrl);
//             }
//             PathVerb::Close => {
//                 last_path = PathType::Close;
//                 is_colse = true;
//             }
//             PathCommand::MoveTo => todo!(),
//             PathCommand::LineTo => todo!(),
//             PathCommand::CurveTo => todo!(),
//             PathCommand::ClosePath => todo!(),
//         }
//     }
//     assert_eq!(points.next(), None);
//     println!("is_close: {}", is_colse);
//     is_colse
// }

pub fn compute_near_arc_impl(
    view_box: Aabb,
    endpoints: Vec<ArcEndpoint>,
    is_area: bool,
) -> (BlobArc, HashMap<u64, u64>) {
    let extents = view_box;
    // println!("extents: {:?}", extents);
    let mut min_width = f32::INFINITY;
    let mut min_height = f32::INFINITY;

    let mut p0 = Point::new(0., 0.);
    // println!("extents2: {:?}", extents);
    let mut near_arcs = Vec::with_capacity(endpoints.len());
    let mut arcs = Vec::with_capacity(endpoints.len());
    println!("endpoints: {:?}", endpoints);
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
    println!("arcs:{:?}", arcs.len());
    recursion_near_arcs_of_cell(
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

    let (unit_arcs, map) = encode_uint_arc_data(result_arcs, &extents, min_width, min_height, Some(is_area));

    let [min_sdf, max_sdf] = travel_data(&unit_arcs);
    let blob_arc = BlobArc {
        min_sdf,
        max_sdf,
        cell_size: extents.width() / unit_arcs.len() as f32,
        #[cfg(feature = "debug")]
        show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
  
        extents,
        data: unit_arcs,
        avg_fetch_achieved: 0.0,
        endpoints: endpoints.clone(),
    };

    (blob_arc, map)
}
