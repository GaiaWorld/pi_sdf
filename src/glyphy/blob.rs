use std::collections::{ HashMap};

// use hashlink::LinkedHashMap;
use parry2d::{
    bounding_volume::Aabb,
    math::{Point, Vector},
};

use super::{
    geometry::{
        arc::{Arc, ArcEndpoint},
        arcs::glyphy_arc_list_extents,
        line::Line,
        vector::VectorEXT,
    },
    sdf::glyphy_sdf_from_arc_list,
    util::{is_inf, GLYPHY_INFINITY},
};
use crate::glyphy::geometry::aabb::AabbEXT;
use crate::glyphy::geometry::point::PointExt;

const MAX_GRID_SIZE: f32 = 63.0;

const GLYPHY_MAX_D: f32 = 0.5;

const MAX_X: f32 = 4095.;
const MAX_Y: f32 = 4095.;

#[derive(Clone, Debug)]
pub struct UnitArc {
    pub(crate) offset: usize, // 此单元（去重后）在数据纹理中的 像素偏移（不是字节偏移）；

    pub(crate) sdf: f32, // 方格中心对应的sdf

    pub(crate) show: String, // 用于Canvas显示的字符串

    pub(crate) data: Vec<ArcEndpoint>,

    pub(crate) origin_data: Vec<ArcEndpoint>, // 原始数据, 用于显示 点 (因为data 对 1, 0 做了优化)
}


#[derive(Debug)]
pub struct BlobArc {
    cell_size: f32,

    pub(crate) width_cells: f32,
    pub(crate) height_cells: f32,

    pub tex_data: Option<TexData>,

    // 显示
    show: String,

    extents: Aabb,
    data: Vec<Vec<UnitArc>>,
    avg_fetch_achieved: f32,
}

/**
 * 找 距离 cell 最近的 圆弧，放到 near_endpoints 返回
 * Uses idea that all close arcs to cell must be ~close to center of cell.
 * @returns {f32} 1 外；-1 内
 */
pub fn closest_arcs_to_cell(
    // cell 坐标
    cx: f32,
    cy: f32,

    // cell 的 左上 和 右下 顶点 坐标
    c0: Point<f32>,
    c1: Point<f32>, /* corners */
    // 近距离的判断
    mut faraway: f32,

    enlighten_max: f32,
    embolden_max: f32,

    // 改字体 所有的 圆弧
    endpoints: &Vec<ArcEndpoint>,
    loop_start_indies: &Vec<usize>,

    // 输出参数
    near_endpoints: &mut Vec<ArcEndpoint>,
) -> (f32, Vec<ArcEndpoint>) {
    let num_endpoints = endpoints.len();

    // This can be improved:
    let synth_max = enlighten_max.max(embolden_max);
    faraway = faraway.max(synth_max);

    // cell 的 中心
    let c = c0.midpoint(&c1);
    // 所有的 圆弧到 中心 的 距离
    let (mut min_dist, start_index) = glyphy_sdf_from_arc_list(endpoints, c);

    let side = if min_dist >= 0.0 { 1 } else { -1 };
    min_dist = min_dist.abs();
    let mut near_arcs = vec![];

    // 最近的意思：某个半径的 圆内
    let half_diagonal = (c - c0).norm();

    let added = half_diagonal;
    // let added = min_dist + half_diagonal + synth_max;

    let radius_squared = added * added;

    if min_dist - half_diagonal <= faraway {
        let mut p0 = Point::new(0., 0.);
        for i in 0..num_endpoints {
            let endpoint = &endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = endpoint.p;
                continue;
            }
            let arc = Arc::new(p0, endpoint.p, endpoint.d);
            p0 = endpoint.p;

            if arc.squared_distance_to_point(c) <= radius_squared {
                near_arcs.push(arc);
            }
        }
    }

    let mut p1 = Point::new(0.0, 0.);
    for i in 0..near_arcs.len() {
        let arc = &near_arcs[i];

        if (i == 0 || !p1.equals(&arc.p0)) {
            let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
            near_endpoints.push(endpoint);
            p1 = arc.p0;
        }

        let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
        near_endpoints.push(endpoint);
        p1 = arc.p1;
    }

    // 全外 或者 全内 时
    let mut effect_endpoints: Vec<ArcEndpoint> = vec![];
    if near_arcs.len() == 0 {
        let [loop_start, loop_end] = get_loop_idnex(start_index, loop_start_indies);
        effect_endpoints =
            choose_best_arcs(start_index, loop_start, loop_end, endpoints, side, c0, c1);
    }

    return (side as f32 * min_dist, effect_endpoints);
}

// 取 index 所在的 循环的 起始和结束索引
// loop_start_indies 的 第一个 元素 肯定是 0
// loop_start_indies 的 最后一个 元素 是用于 回环的 哨兵
pub fn get_loop_idnex(index: usize, loop_start_indies: &Vec<usize>) -> [usize; 2] {
    if loop_start_indies[0] != 0 {
        panic!(
            "loop_start_indies[0] != 0, loop_start_indies[0] = {}",
            loop_start_indies[0]
        );
    }

    if index < 0 || index >= loop_start_indies[loop_start_indies.len() - 1] {
        panic!("`index < 0 || index >= loop_start_indies[loop_start_indies.length - 1], index = {}, loop_start_indies[loop_start_indies.length - 1] = {}", index, loop_start_indies[loop_start_indies.len() - 1]);
    }

    for i in 0..loop_start_indies.len() {
        let mut curr = loop_start_indies[i];
        if curr > index {
            let prev = loop_start_indies[i - 1];

            curr -= 1;
            return [prev, curr];
        }
    }
    panic!(
        "get_loop_idnex error, no reach: index = {}, loop_start_indies = {:?}",
        index, loop_start_indies
    );
}

// 选择 最佳的 圆弧
// sart_index 在 [loop_start, loop_end] 标注 的 环上
pub fn choose_best_arcs(
    start_index: usize,
    loop_start: usize,
    loop_end: usize,
    endpoints: &Vec<ArcEndpoint>,
    sdf_sign: i32,
    cp0: Point<f32>,
    cp1: Point<f32>,
) -> Vec<ArcEndpoint> {
    let mut index = get_curr_index(start_index, loop_start, loop_end);
    let (same_count, arcs) = is_best_arcs(
        index, loop_start, loop_end, 2, &endpoints, sdf_sign, cp0, cp1,
    );
    if same_count == 4 {
        return arcs;
    }

    index = get_prev_index(start_index, loop_start, loop_end);
    let (same_count2, arcs2) = is_best_arcs(
        index, loop_start, loop_end, 2, &endpoints, sdf_sign, cp0, cp1,
    );
    if same_count2 == 4 {
        return arcs2;
    }

    index = get_next_index(start_index, loop_start, loop_end);
    let (same_count3, arcs3) = is_best_arcs(
        index, loop_start, loop_end, 2, &endpoints, sdf_sign, cp0, cp1,
    );
    if same_count3 == 4 {
        return arcs3;
    }

    index = get_curr_index(start_index, loop_start, loop_end);
    let (same_count4, arcs4) = is_best_arcs(
        index, loop_start, loop_end, 3, &endpoints, sdf_sign, cp0, cp1,
    );
    if same_count4 == 4 {
        return arcs4;
    }

    index = get_prev_index(start_index, loop_start, loop_end);
    let (same_count5, arcs5) = is_best_arcs(
        index, loop_start, loop_end, 3, &endpoints, sdf_sign, cp0, cp1,
    );
    if same_count5 == 4 {
        return arcs5;
    }

    let mut new_arcs = vec![];
    for i in &arcs {
        new_arcs.push(i.clone());
    }
    let msg = format!("choose_best_arcs error: start_index = {}, sdf_sign = {}, cp0 = ({}, {}), cp1 = ({}, {}), arcs = ", start_index, sdf_sign, cp0.x, cp0.y, cp1.x, cp1.y) ;

    log::warn!("{} {:?}, all endpoints = {:?}", msg, new_arcs, endpoints);
    // throw new Error(msg);

    return arcs;
}

// 选择 最佳的 圆弧
pub fn is_best_arcs(
    mut index: usize,
    loop_start: usize,
    loop_end: usize,
    num: usize,
    endpoints: &Vec<ArcEndpoint>,
    sdf_sign: i32,
    cp0: Point<f32>,
    cp1: Point<f32>,
) -> (usize, Vec<ArcEndpoint>) {
    let mut r = vec![];

    for _i in 0..num {
        let endpoint = &endpoints[index];
        r.push(ArcEndpoint::new(endpoint.p.x, endpoint.p.y, endpoint.d));
        index = get_next_index(index, loop_start, loop_end);
    }

    r[0].d = GLYPHY_INFINITY;
    let same_count = is_quad_same_sign(cp0, cp1, &r, sdf_sign);
    return (same_count, r);
}

pub fn get_curr_index(index: usize, _loop_start: usize, _loop_end: usize) -> usize {
    return index;
}

// 沿着环 [loop_start, loop_end] 找 index的 下一个索引
pub fn get_next_index(mut index: usize, loop_start: usize, loop_end: usize) -> usize {
    // index must in [loop_start, loop_end]
    if index < loop_start || index > loop_end {
        panic!(
            "get_next_index error: index = {}, loop_start = {}, loop_end = {}",
            index, loop_start, loop_end
        );
    }

    index += 1;
    if index > loop_end {
        index = loop_start + 1;
    }
    return index;
}

// 沿着环 [loop_start, loop_end] 找 index 的 上一个索引
pub fn get_prev_index(mut index: usize, loop_start: usize, loop_end: usize) -> usize {
    // let index= index as i32;
    // index must in [loop_start, loop_end]
    if index < loop_start || index > loop_end {
        panic!(
            "get_prev_index error: index = {}, loop_start = {}, loop_end = {}",
            index, loop_start, loop_end
        );
    }

    let mut index_copy = index as i32 - 1;
    if index_copy < loop_start as i32 {
        index_copy = loop_end as i32 - 1;
    }
    index = index_copy as usize;
    return index as usize;
}

// 正方形的四个角落是否 全部 在 给定圆弧的 外面/里面
// 返回有几个点的 符号 和 sdf_sign 相同
pub fn is_quad_same_sign(
    cp0: Point<f32>,
    cp1: Point<f32>,
    endpoints: &Vec<ArcEndpoint>,
    sdf_sign: i32,
) -> usize {
    let mut i = 0;
    for p in vec![cp0, Point::new(cp0.x, cp1.y), Point::new(cp1.x, cp0.y), cp1] {
        if is_point_same_sign(p, endpoints, sdf_sign) {
            i += 1;
        }
    }
    return i;
}

// 验证 sdf 四个角落 的点 是否和 给定的sdf 符号相同
pub fn is_point_same_sign(point: Point<f32>, endpoints: &Vec<ArcEndpoint>, sdf_sign: i32) -> bool {
    let (min_dist, _) = glyphy_sdf_from_arc_list(endpoints, point);

    let v = if min_dist > 0.0 {
        1
    } else if min_dist < 0.0 {
        -1
    } else {
        0
    };

    return v == sdf_sign;
}
// // 一
pub fn glyphy_arc_list_encode_blob2(
    endpoints: &Vec<ArcEndpoint>,
    faraway: f32,
    grid_unit: f32,
    enlighten_max: f32,
    embolden_max: f32,
    pextents: &mut Aabb,
) -> BlobArc {
    let mut extents = Aabb::new_invalid();

    let mut loop_start_indies = [].to_vec();
    for i in 0..endpoints.len() {
        let ep = &endpoints[i];
        if ep.d == GLYPHY_INFINITY {
            loop_start_indies.push(i);
        }
    }
    // 最后一个是用于 回环的 哨兵
    loop_start_indies.push(endpoints.len());

    glyphy_arc_list_extents(&endpoints, &mut extents);

    if extents.is_empty() {
        // 不可显示 字符，比如 空格，制表符 等
        pextents.set(&extents);

        return BlobArc {
            width_cells: 1.0,
            height_cells: 1.0,
            cell_size: 1.0,

            show: "".to_owned(),

            tex_data: None,

            extents: extents.clone(),
            data: vec![],
            avg_fetch_achieved: 0.0,
        };
    }

    // 添加 抗锯齿的 空隙
    extents.mins.x -= faraway + embolden_max;
    extents.mins.y -= faraway + embolden_max;
    extents.maxs.x += faraway + embolden_max;
    extents.maxs.y += faraway + embolden_max;

    let mut glyph_width = extents.maxs.x - extents.mins.x;
    let mut glyph_height = extents.maxs.y - extents.mins.y;
    let unit = glyph_width.max(glyph_height);

    // 字符 的 glyph 被分成 grid_w * grid_h 个 格子
    let grid_w = MAX_GRID_SIZE.min((glyph_width / grid_unit).ceil());
    let grid_h = MAX_GRID_SIZE.min((glyph_height / grid_unit).ceil());

    if (glyph_width > glyph_height) {
        glyph_height = grid_h * unit / grid_w;
        extents.maxs.y = extents.mins.y + glyph_height;
    } else {
        glyph_width = grid_w * unit / grid_h;
        extents.maxs.x = extents.mins.x + glyph_width;
    }

    let cell_unit = unit / grid_w.max(grid_h);

    // 每个 格子的 最近的 圆弧
    let mut near_endpoints: Vec<ArcEndpoint> = vec![];

    let origin = Point::new(extents.mins.x, extents.mins.y);
    let total_arcs = 0;

    let mut result_arcs = vec![];
    for row in 0..grid_h as i32 {
        let mut row_arcs = vec![
            UnitArc {
                offset: 0,
                sdf: 0.0,
                show: "".to_owned(),
                data: vec![],
                origin_data: vec![],
            };
            grid_w as usize
        ];
        for col in 0..grid_w as i32 {
            let unit_arc = &mut row_arcs[col as usize];

            let cp0 = origin.add_vector(&Vector::new(
                (col as f32 + 0.0) * cell_unit,
                (row as f32 + 0.0) * cell_unit,
            ));
            let cp1 = origin.add_vector(&Vector::new(
                (col as f32 + 1.0) * cell_unit,
                (row as f32 + 1.0) * cell_unit,
            ));

            near_endpoints.clear();

            if (col == 20 && row == 0) {
                log::warn!(
                    "col: {}, row: {}, cp0: {}, {}, cp1: {}, {}",
                    col,
                    row,
                    cp0.x,
                    cp0.y,
                    cp1.x,
                    cp1.y
                )
            }

            // 判断 每个 格子 最近的 圆弧
            let (sdf, effect_endpoints) = closest_arcs_to_cell(
                col as f32,
                row as f32,
                cp0,
                cp1,
                faraway,
                enlighten_max,
                embolden_max,
                endpoints,
                &loop_start_indies,
                &mut near_endpoints,
            );
            unit_arc.sdf = sdf;

            if (near_endpoints.len() == 0) {
                near_endpoints = effect_endpoints;
            }

            // 线段，终点的 d = 0
            if (near_endpoints.len() == 2 && near_endpoints[1].d == 0.0) {
                // unit_arc.data.push(near_endpoints[0]);
                // unit_arc.data.push(near_endpoints[1]);

                let start = &near_endpoints[0];
                let end = &near_endpoints[1];

                let mut line = Line::from_points(
                    snap(&start.p, &extents, glyph_width, glyph_height),
                    snap(&end.p, &extents, glyph_width, glyph_height),
                );

                // c 第一个网格的中心
                let c = Point::new(
                    extents.mins.x + glyph_width * 0.5,
                    extents.mins.y + glyph_height * 0.5,
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

                unit_arc.origin_data.push(start.clone());
                unit_arc.origin_data.push(end.clone());

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

            // row_arcs.push(unit_arc);
        }
        result_arcs.push(row_arcs);
    }

    pextents.set(&extents);

    let mut data = BlobArc {
        cell_size: cell_unit,
        width_cells: grid_w,
        height_cells: grid_h,

        show: "".to_owned(),

        tex_data: None,

        data: result_arcs,
        extents: extents.clone(),
        avg_fetch_achieved: 1. + total_arcs as f32 / (grid_w * grid_h),
    };

    let [min_sdf, max_sdf] = travel_data(&data);

    data.show.push_str(&format!(
        "<br> 格子数：宽 = {}, 高 = {} <br>",
        grid_w, grid_h
    ));

    data.tex_data = Some(encode_to_tex(
        &mut data,
        extents,
        glyph_width,
        glyph_height,
        grid_w,
        grid_h,
        min_sdf,
        max_sdf,
    ));

    return data;
}

#[derive(Debug)]
pub struct TexData {
    pub index_tex: Vec<u8>, // 字节数 = 2 * 像素个数
    pub data_tex: Vec<u8>,  // 字节数 = 4 * 像素个数

    pub grid_w: f32,
    pub grid_h: f32,

    pub cell_size: f32,

    pub max_offset: usize,
    pub min_sdf: f32,
    pub sdf_step: f32,
}

// 两张纹理，索引纹理 和 数据纹理
//
// 数据纹理：
//     32bit: [p.x, p.y, d]
//     按 数据 去重
//素，每像素 2B
// uniform: [max_offset, min_sdf,  索引纹理：共 grid_w * grid_h 个像sdf_step]
pub fn encode_to_tex(
    data: &mut BlobArc,
    extents: Aabb,
    glyph_width: f32,
    glyph_height: f32,
    grid_w: f32,
    grid_h: f32,
    min_sdf: f32,
    max_sdf: f32,
) -> TexData {
    let (data_map, data_tex) = encode_data_tex(data, extents, glyph_width, glyph_height);

    let max_offset = data_tex.len() / 4;
    // 计算sdf的 梯度等级
    let mut level = (2usize.pow(14) / max_offset) - 1;
    if (level < 1) {
        level = 1;
    }
    let sdf_range = max_sdf - min_sdf + 0.1;
    // 量化：将 sdf_range 分成 level 个区间，看 sdf 落在哪个区间
    let sdf_step = sdf_range / level as f32;

    // 2 * grid_w * grid_h 个 Uint8
    let mut indiecs = vec![];
    for i in 0..data.data.len() {
        let row = &mut data.data[i];
        for j in 0..row.len() {
            let unit_arc = &mut row[j];
            let key = get_key(&unit_arc);
            if !key.is_empty() {
                let map_arc_data = data_map.get(&key);
                if (map_arc_data.is_none()) {
                    panic!("unit_arc not found");
                }
                let map_arc_data = map_arc_data.as_ref().unwrap();

                let mut num_points = map_arc_data.data.len();
                if num_points > 3 {
                    num_points = 0;
                }

                let offset = map_arc_data.offset;
                let sdf = unit_arc.sdf;

                let cell_size = data.cell_size;
                let is_interval = sdf.abs() <= cell_size * 0.5f32.sqrt();

                let [encode, sdf_index] = encode_to_uint16(
                    is_interval,
                    num_points as f32,
                    offset as f32,
                    max_offset as f32,
                    sdf,
                    min_sdf,
                    sdf_step,
                );

                indiecs.push(encode);

                let r = decode_from_uint16(encode, max_offset as f32, min_sdf, sdf_step);

                if r.num_points != num_points as f32
                    || r.offset != offset as f32
                    || is_interval != r.is_interval
                {
                    // log::warn!(`encode index error: min_sdf = ${min_sdf}, max_sdf = ${max_sdf}, max_offset = ${max_offset}`);
                    // console.error(`encode index error: encode_to_uint16: is_interval = ${is_interval}, num_points = ${num_points}, offset = ${offset}, sdf = ${sdf}, encode = ${encode}`);
                    // console.error(`encode index error: decode_from_uint16: is_interval = ${r.is_interval}, num_points = ${r.num_points}, offset = ${r.offset}, sdf = ${r.sdf}`);
                    // console.error(``);

                    panic!("encode index error")
                }

                // if (i === 16 && j === 15) {
                // 	console.warn(`encode index: num_points: ${num_points}, offset: ${offset}, sdf: ${sdf}, encode: ${encode}`);
                // }

                // 解码后的 sdf
                let dsdf = min_sdf + sdf_index * sdf_step;
                // unit_arc.show = `${num_points}:${dsdf.toFixed(1)}`;
                unit_arc.show = format!("{}", num_points);
                // unit_arc.show = `${offset}`;
            }
        }
    }

    let cell_size = data.cell_size;
    data.show.push_str(&format!("<br> var max_offset = {:.2}, min_sdf = {:.2}, max_sdf = {:.2}, sdf_step = {:.2}, cell_size = {:.2} <br>", max_offset, min_sdf, max_sdf, sdf_step, cell_size) );

    let mut level_sdf = vec![];
    for i in 0..level {
        let sdf = min_sdf + sdf_step * i as f32;
        level_sdf.push(format!("{:.2}", sdf));
    }
    // data.show += `<br> sdf_level: ${level_sdf.join(", ")} <br>`;

    let mut index_tex = vec![0; 2 * indiecs.len()];
    for i in 0..indiecs.len() {
        let d = indiecs[i];
        index_tex[2 * i] = (d as i32 & 0xff) as u8;
        index_tex[2 * i + 1] = (d as i32 >> 8) as u8;
    }

    return TexData {
        data_tex,
        index_tex,

        // unitform
        cell_size,

        grid_w,
        grid_h,

        max_offset,

        min_sdf,
        sdf_step,
    };
}

// 返回 u16，从高到低
// num_points: 2-bit
// offset + sdf: 14-bit
// 返回 [encode, sdf_index]
pub fn encode_to_uint16(
    is_interval: bool, // 圆弧和晶格是否相交；
    num_points: f32,   // 只有 0，1，2，3 四个值

    offset: f32,     // 数据 在 数据纹理 的偏移，单位：像素，介于 [0, max_offset] 之间
    max_offset: f32, // 最大的偏移，单位像素

    sdf: f32,     // 浮点数，介于 [min_sdf, max_sdf] 之间
    min_sdf: f32, // sdf 的 最小值, 为负数表示内部
    sdf_step: f32,
) -> [f32; 2] {
    // 以区间的索引作为sdf的编码
    let mut sdf_index = ((sdf - min_sdf) / sdf_step).floor();

    // 比实际的 sdf 范围多出 2
    // 用 0 表示 完全 在内 的 晶格！
    // 用 1 表示 完全 在外 的 晶格！
    if !is_interval {
        sdf_index = if sdf > 0.0 { 1. } else { 0. };
    } else {
        sdf_index += 2.0;
    }

    // 将 sdf_index 和 offset 编码到一个 uint16 中
    // 注：二维坐标 编码成 一维数字的常用做法
    let sdf_and_offset_index = sdf_index * max_offset + offset;

    if (sdf_and_offset_index >= (2i32.pow(14)) as f32) {
        println!(
            "sdf_and_offset_index: {}, 2 ^ 14 : {}",
            sdf_and_offset_index,
            2i32.pow(14)
        );
        panic!(
            "Encode error, out of range !, sdf_and_offset_index = {}",
            sdf_and_offset_index
        );
    }

    let mut r = ((num_points as i32) << 14) | sdf_and_offset_index as i32;
    r = r & 0xffff;
    return [r as f32, sdf_index];
}

pub struct Res {
    is_interval: bool,
    num_points: f32,
    sdf: f32,
    offset: f32,
}
// value: u16，从高到低
// num_points: 2-bit
// offset + sdf: 14-bit
pub fn decode_from_uint16(value: f32, max_offset: f32, min_sdf: f32, sdf_step: f32) -> Res {
    let num_points = (value / 16384.0).floor();
    let sdf_and_offset_index = value % 16384.0;

    let mut sdf_index = (sdf_and_offset_index / max_offset).floor();
    let offset = sdf_and_offset_index % max_offset;

    let mut sdf = 0.0;
    let mut is_interval = true;

    // 比实际的 sdf 范围多出 2
    // 用 0 表示 完全 在内 的 晶格！
    // 用 1 表示 完全 在外 的 晶格！
    if (sdf_index == 0.0) {
        is_interval = false;
        sdf = -GLYPHY_INFINITY;
    } else if (sdf_index == 1.0) {
        is_interval = false;
        sdf = GLYPHY_INFINITY;
    } else {
        sdf_index -= 2.0;
        sdf = sdf_index * sdf_step + min_sdf;
    }

    return Res {
        is_interval,
        num_points,
        sdf,
        offset,
    };
}

pub fn get_line_key(ep0: &ArcEndpoint, ep1: &ArcEndpoint) -> String {
    let mut key = "".to_string();
    key.push_str(&format!("{}_{}_{}_", ep0.p.x, ep0.p.y, ep0.d));
    key.push_str(&format!("{}_{}_{}_", ep1.p.x, ep1.p.y, ep1.d));
    return key;
}

pub fn get_key(unit_arc: &UnitArc) -> String {
    let mut key = "".to_string();
    if (unit_arc.data.len() == 1 && unit_arc.data[0].line_key.is_some()) {
        // 线段
        key.push_str(unit_arc.data[0].line_key.as_ref().unwrap());
    } else {
        for endpoint in &unit_arc.data {
            key.push_str(&format!(
                "{}_{}_{}_",
                endpoint.p.x, endpoint.p.y, endpoint.d
            ));
        }
    }
    return key;
}

// 按数据去重，并编码到纹理
pub fn encode_data_tex(
    data: &mut BlobArc,
    extents: Aabb,
    width_cells: f32,
    height_cells: f32,
) -> (HashMap<String, UnitArc>, Vec<u8>) {
    // println!("data: {:?}, extents: {:?}, width_cells: {}, height_cells:{}", data.data, extents, width_cells,height_cells);
    let mut map = HashMap::new();

    let mut before_size = 0;

    let mut keys :Vec<String> = vec![];
    for row in &data.data {
        for unit_arc in row {
            let key = get_key(&unit_arc);
            before_size += unit_arc.data.len();
            if (key.len() > 0) {
                // let r = unit_arc.clone();
                map.insert(key.clone(), unit_arc.clone());
                if keys.iter().find(|v| **v == key).is_none(){
                    keys.push(key);
                }
                
            }
        }
    }

    // let mut after_size = 0;
    // for value in map.values() {
    //     after_size += value.data.len();
    // }

    let mut r = vec![];

    // console.warn(`map size = ${map.size}, before_size = ${before_size}, after_size = ${after_size}, ratio = ${after_size / before_size}`)

    // for key in &keys {
    for v in map.values_mut() {
        // let unit_arc = map.get_mut(key).unwrap();
        let unit_arc = v;
        // if (unit_arc.is_none()) {
        //     panic!("unit_arc is null");
        // }
        // let unit_arc = unit_arc.unwrap();

        unit_arc.offset = r.len() / 4;

        if (unit_arc.data.len() == 1) {
            assert!(unit_arc.data[0].line_encode.is_some());
            if let Some(data) = &unit_arc.data[0].line_encode {
                r.push(data[0] as u8);
                r.push(data[1] as u8);
                r.push(data[2] as u8);
                r.push(data[3] as u8);
            }
        } else {
            for endpoint in &unit_arc.data {
                let qx = quantize_x(endpoint.p.x, &extents, width_cells);
                let qy = quantize_y(endpoint.p.y, &extents, height_cells);
                let rgba = arc_endpoint_encode(qx, qy, endpoint.d);

                // console.warn(`encode_data_tex ${r.length / 4}, (${endpoint.p.x.toFixed(1)}, ${endpoint.p.y.toFixed(1)}), d = ${endpoint.d.toFixed(2)}`)
                r.push(rgba[0] as u8);
                r.push(rgba[1] as u8);
                r.push(rgba[2] as u8);
                r.push(rgba[3] as u8);
            }
        }

        // 单元的端点个数超过 3 个，补充一个全零像素代表结束；
        if unit_arc.data.len() > 3 {
            r.push(0);
            r.push(0);
            r.push(0);
            r.push(0);
        }
    }

    data.show.push_str(&format!(
        "<br>数据纹理 像素数量: before = ${}, after = ${}<br>",
        before_size,
        r.len() / 4
    ));

    let tex_data = r;

    return (map, tex_data);
}

pub fn quantize_x(x: f32, extents: &Aabb, glyph_width: f32) -> f32 {
    return (MAX_X * ((x - extents.mins.x) / glyph_width)).round();
}

pub fn quantize_y(y: f32, extents: &Aabb, glyph_height: f32) -> f32 {
    return (MAX_Y * ((y - extents.mins.y) / glyph_height)).round();
}

pub fn dequantize_x(x: f32, extents: &Aabb, glyph_width: f32) -> f32 {
    return x / MAX_X * glyph_width + extents.mins.x;
}

pub fn dequantize_y(y: f32, extents: &Aabb, glyph_height: f32) -> f32 {
    return y / MAX_Y * glyph_height + extents.mins.y;
}

pub fn snap(p: &Point<f32>, extents: &Aabb, glyph_width: f32, glyph_height: f32) -> Point<f32> {
    let qx = quantize_x(p.x, extents, glyph_width);
    let x = dequantize_x(qx, extents, glyph_width);

    let qy = quantize_y(p.y, extents, glyph_height);
    let y = dequantize_y(qy, extents, glyph_height);

    return Point::new(x, y);
}

// const upper_bits = (v: f32, bits: f32, total_bits: f32): f32 => {
// 	return v >> (total_bits - bits);
// }

pub fn lower_bits(v: f32, bits: f32, total_bits: f32) -> f32 {
    return (v as i32 & ((1 << bits as i32) - 1)) as f32;
}

// 将 一个圆弧端点 编码为 RGBA, 4个字节
pub fn arc_endpoint_encode(ix: f32, iy: f32, d: f32) -> [f32; 4] {
    if (ix > MAX_X) {
        panic!("ix must be less than or equal to MAX_X");
    }
    if (iy > MAX_Y) {
        panic!("iy must be less than or equal to MAX_Y");
    }
    let id;
    if (is_inf(d)) {
        id = 0.0;
    } else {
        if (d.abs() > GLYPHY_MAX_D) {
            panic!("d must be less than or equal to GLYPHY_MAX_D");
        }

        id = 128. + (d * 127.0 / GLYPHY_MAX_D).round();
    }
    if (id >= 256.0) {
        panic!("id must be less than 256");
    }
    let r = id as i32;
    let g = lower_bits(ix, 8.0, 12.0);
    let b = lower_bits(iy, 8.0, 12.0);
    let a = ((ix as i32 >> 8) << 4) | (iy as i32 >> 8);

    return [r as f32, g, b, a as f32];
}

pub fn travel_data(blob: &BlobArc) -> [f32; 2] {
    let mut min_sdf = f32::INFINITY;
    let mut max_sdf = -f32::INFINITY;

    // 初始化队列
    for i in 0..blob.data.len() {
        let row = &blob.data[i];
        for j in 0..row.len() {
            let unit_arc = &row[j];
            let curr_dist = unit_arc.sdf;

            if (curr_dist < min_sdf) {
                min_sdf = curr_dist;
            }
            if (curr_dist > max_sdf) {
                max_sdf = curr_dist;
            }
        }
    }

    return [min_sdf, max_sdf];
}

// rgba
pub fn line_encode(line: Line) -> [f32; 4] {
    let l = line.normalized();

    let angle = l.n.sdf_angle();
    let ia = (-angle / std::f32::consts::PI * 0x7FFF as f32).round();
    let ua = ia + 0x8000 as f32;
    assert!(0 == (ua as i32 & -(0xFFFF + 1)));

    let d = l.c;
    let id = (d * 0x1FFF as f32).round();
    let ud = id + 0x4000 as f32;
    assert!(0 == (ud as i32 & -(0x7FFF + 1)));
    let ud = ud as i32 | 0x8000;

    return [
        (ud >> 8) as f32,
        (ud & 0xFF) as f32,
        (ua as i32 >> 8) as f32,
        (ua as i32 & 0xFF) as f32,
    ];
}

pub fn line_decode(encoded: [f32; 4], nominal_size: [f32; 2]) -> Line {
    let ua = encoded[2] * 256.0 + encoded[3];
    let ia = ua - 0x8000 as f32;
    let angle = -ia / 0x7FFF as f32 * 3.14159265358979;

    let ud = (encoded[0] - 128.0) * 256.0 + encoded[1];

    let id = ud - 0x4000 as f32;
    let d = id / 0x1FFF as f32;
    let scale = nominal_size[0].max(nominal_size[1]);

    let n = Vector::new(angle.cos(), angle.sin());

    return Line::from_normal_d(n, d * scale);
}
