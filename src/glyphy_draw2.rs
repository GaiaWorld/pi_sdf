use parry2d::{bounding_volume::Aabb, math::Point};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    glyphy::{
        blob_new::{
            encode_to_tex, get_line_key, line_encode, recursion_near_arcs_of_cell, snap,
            travel_data, BlobArc, Extents, UnitArc,
        },
        geometry::{
            aabb::AabbEXT,
            arc::{Arc, ArcEndpoint},
            arcs::glyphy_arc_list_extents,
            line::Line,
            point::PointExt,
            vector::VectorEXT,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::{is_inf, GLYPHY_INFINITY},
        vertex::GlyphInfo,
    },
    utils::{FontFace, GlyphVisitor},
};

pub static MIN_FONT_SIZE: f32 = 10.0;

pub static TOLERANCE: f32 = 10.0 / 1024.;

pub static ENLIGHTEN_MAX: f32 = 0.0001; /* Per EM */

pub static EMBOLDEN_MAX: f32 = 0.0001; /* Per EM */

// 取 char对应的 arc
// 实现 encode_ft_glyph

// 取 char对应的 arc
// 实现 encode_ft_glyph
//
// #[wasm_bindgen]
pub fn get_char_arc(
    gi: &mut GlyphInfo,
    ft_face: &mut FontFace,
    char: char,
    tolerance_per_em: Option<f32>,
) -> BlobArc {
    // log::error!("get_char_arc: {:?}", char);
    let tolerance_per_em = if let Some(v) = tolerance_per_em {
        v
    } else {
        TOLERANCE
    };

    let upem = ft_face.font.head_table().unwrap().unwrap().units_per_em as f32;
    let tolerance = upem * tolerance_per_em; /* in font design units */
    let faraway = upem / (MIN_FONT_SIZE * 2.0f32.sqrt());
    let embolden_max = upem * EMBOLDEN_MAX;

    let mut sink = GlyphVisitor::new(1.0);
    sink.accumulate.tolerance = tolerance;

    ft_face.to_outline(char, &mut sink);

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

    let begin = std::time::Instant::now();

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

    // for arc in &near_arcs {
    //     arcs.push(unsafe { std::mem::transmute(arc) });
    // }

    let mut result_arc = vec![];
    let mut temp = Vec::with_capacity(arcs.len());
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
        &mut result_arc,
        &mut temp
    );

    let width_cells = (extents.width() / min_width).floor() as usize;
    let height_cells = (extents.height() / min_height).floor() as usize;
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
            };
            width_cells
        ];
        height_cells
    ];

    // println!("result_arc: {:?}", result_arc.len());

    let glyph_width = extents.width();
    let glyph_height = extents.height();
    let c = extents.center();
    let unit = glyph_width.max(glyph_height);

    for (near_arcs, cell) in result_arc {
        let mut near_endpoints = vec![];
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
        let begin_x = (begin.x / min_width).floor() as usize;
        let begin_y = (begin.y / min_height).floor() as usize;

        let end_x = (end.x / min_width).floor() as usize;
        let end_y = (end.y / min_height).floor() as usize;
        let parent_cell = Extents {
            min_x: cell.mins.x,
            min_y: cell.mins.y,
            max_x: cell.maxs.x,
            max_y: cell.maxs.y,
        };
        for i in begin_x..end_x {
            for j in begin_y..end_y {
                let unit_arc = &mut data[j][i];

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

                    let line_key = get_line_key(&near_endpoints[0], &near_endpoints[1]);
                    let le = line_encode(line);

                    let mut line_data = ArcEndpoint::new(0.0, 0.0, 0.0);
                    line_data.line_key = Some(line_key);
                    line_data.line_encode = Some(le);

                    unit_arc.data.push(line_data);
                    // println!("1row: {}, col: {} line_data: {:?}n \n", row, col, unit_arc.data.len());
                    unit_arc.origin_data.push(start.clone());
                    unit_arc.origin_data.push(end.clone());
                    unit_arc.parent_cell = parent_cell;
                    continue;
                }

                // If the arclist is two arcs that can be combined in encoding if reordered, do that.
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
                for i in 0..near_endpoints.len() {
                    let endpoint = near_endpoints[i].clone();
                    unit_arc.data.push(endpoint);
                }
                unit_arc.parent_cell = parent_cell;
                // if (i == 7 && j == 8) || (i == 8 && j == 8) {
                //     log::info!("i: {}, j: {}, cell: {:?}, near_arcs: {:?}", i, j, cell, near_arcs);
                //     log::info!("near_endpoints: {:?}, data: {:?}", near_endpoints, unit_arc.data);
                // }
            }
        }
    }
    println!("格子计算: {:?}", begin.elapsed());

    let mut arcs = BlobArc {
        cell_size: min_width,
        width_cells: width_cells as f32,
        height_cells: height_cells as f32,
        tex_data: None,
        show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
        extents,
        data,
        avg_fetch_achieved: 0.0,
        endpoints: endpoints.clone(),
    };
    let [min_sdf, max_sdf] = travel_data(&arcs);

    arcs.tex_data = Some(encode_to_tex(
        &mut arcs,
        extents,
        glyph_width,
        glyph_height,
        width_cells as f32,
        height_cells as f32,
        min_sdf,
        max_sdf,
    ));

    extents.scale(1.0 / upem, 1.0 / upem);

    gi.nominal_w = width_cells as f32;
    gi.nominal_h = height_cells as f32;

    gi.extents.set(&extents);

    arcs
}

#[wasm_bindgen]
pub fn get_char_arc_debug(char: String) -> BlobArc {
    console_error_panic_hook::set_once();

    let _ = console_log::init_with_level(log::Level::Debug);
    let buffer = include_bytes!("../source/msyh.ttf").to_vec();
    log::info!("1111111111");
    let mut ft_face = FontFace::new(buffer);
    let mut gi = GlyphInfo::new();
    log::info!("22222222char: {}", char);
    let char = char.chars().next().unwrap();
    log::info!("13333333");
    let arcs = get_char_arc(&mut gi, &mut ft_face, char, None);
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
