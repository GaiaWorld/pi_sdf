use std::{collections::HashMap, ops::Range};

// use freetype_sys::FT_New_Face;
// use parry2d::na::ComplexField;
// use hashlink::LinkedHashMap;
use parry2d::{ math::Vector};

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use super::{
    geometry::{
        aabb::{Aabb, Direction},
        arc::{Arc, ArcEndpoint},
        line::Line,
        segment::SegmentEXT,
        vector::VectorEXT,
    },
    sdf::glyphy_sdf_from_arc_list,
    util::{is_inf, GLYPHY_INFINITY},
};

use crate::Point;

pub const MAX_GRID_SIZE: f32 = 63.0;

const GLYPHY_MAX_D: f32 = 0.5;

const MAX_X: f32 = 4095.;
const MAX_Y: f32 = 4095.;

#[derive(Debug)]
pub enum EncodeError {
    MemoryOverflow,
    NewLine,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Clone, Debug)]
pub struct UnitArc {
    pub parent_cell: Extents,
    pub offset: usize, // 此单元（去重后）在数据纹理中的 像素偏移（不是字节偏移）；

    pub sdf: f32, // 方格中心对应的sdf
    #[cfg(feature = "debug")]
    pub(crate) show: String, // 用于Canvas显示的字符串

    pub data: Vec<ArcEndpoint>,

    pub origin_data: Vec<ArcEndpoint>, // 原始数据, 用于显示 点 (因为data 对 1, 0 做了优化)

    pub key: u64,
    pub s_dist: u8,
    pub s_dist_1: u64,
    pub s_dist_2: u64,
    pub s_dist_3: u64,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
impl UnitArc {
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    pub fn get_key(&self) -> u64 {
        if self.data.len() == 1 && self.data[0].line_key.is_some() {
            // 线段
            return self.data[0].line_key.unwrap();
        } else {
            return self.key;
        }
    }

    #[cfg(feature = "debug")]
    pub fn get_show(&self) -> String {
        self.show
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct BlobArc {
    pub min_sdf: f32,
    pub max_sdf: f32,

    pub cell_size: f32,
    // 显示
    #[cfg(feature = "debug")]
    pub(crate) show: String,

    pub(crate) extents: Aabb,

    pub(crate) data: Vec<Vec<UnitArc>>,
    pub avg_fetch_achieved: f32,
    pub(crate) endpoints: Vec<ArcEndpoint>,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct Extents {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
impl BlobArc {
    pub fn get_unit_arc(&self, i: usize, j: usize) -> UnitArc {
        // log::debug!("i: {}, j: {}", i, j);
        self.data[j][i].clone()
    }

    pub fn get_extents(&self) -> Extents {
        Extents {
            max_x: self.extents.maxs.x,
            max_y: self.extents.maxs.y,
            min_x: self.extents.mins.x,
            min_y: self.extents.mins.y,
        }
    }

    pub fn get_endpoints_len(&self) -> usize {
        self.endpoints.len()
    }

    pub fn get_endpoint(&self, index: usize) -> ArcEndpoint {
        self.endpoints[index].clone()
    }
}

impl BlobArc {
    // 按数据去重，并编码到纹理
    pub fn encode_data_tex(
        &self,
        map: &HashMap<u64, u64>,
        data_tex: &mut Vec<u8>,
        data_tex_width: usize,
        offset_x: &mut usize,
        offset_y: &mut usize,
    ) -> Result<usize, EncodeError> {
        match self.encode_data_tex_impl(map, data_tex, data_tex_width, *offset_x, *offset_y) {
            Ok(len) => return Ok(len),
            Err(err) => {
                if let EncodeError::NewLine = err {
                    *offset_x = 0;
                    *offset_y += 8;
                    let len = self.encode_data_tex_impl(
                        &map,
                        data_tex,
                        data_tex_width,
                        *offset_x,
                        *offset_y,
                    )?;
                    return Ok(len);
                } else {
                    return Err(err);
                }
            }
        }
    }

    pub fn encode_data_tex1(&self, map: &HashMap<u64, u64>) -> Vec<u8> {
        // 返回索引数据和宽高
        let mut len = 0usize;
        let glyph_width = self.extents.width();
        let glyph_height = self.extents.height();
        let mut data_tex: Vec<u8> = Vec::with_capacity(map.len());

        for v in map.values() {
            let unit_arc = unsafe { &mut *(*v as *mut UnitArc) };
            unit_arc.offset = len;
            // println!("unit_arc.data.len(): {}", unit_arc.data.len());
            if unit_arc.data.len() == 1 {
                assert!(unit_arc.data[0].line_encode.is_some());
                if let Some(data) = &unit_arc.data[0].line_encode {
                    write_data_tex_by_width(&mut data_tex, data, &mut len);
                }
            } else {
                for endpoint in &unit_arc.data {
                    let qx = quantize_x(endpoint.p[0], &self.extents, glyph_width);
                    let qy = quantize_y(endpoint.p[1], &self.extents, glyph_height);
                    let rgba = arc_endpoint_encode(qx, qy, endpoint.d);

                    write_data_tex_by_width(&mut data_tex, &rgba, &mut len);
                }
            }

            // 单元的端点个数超过 3 个，补充一个全零像素代表结束；
            if unit_arc.data.len() > 3 {
                write_data_tex_by_width(&mut data_tex, &[0., 0., 0., 0.], &mut len);
            }
        }

        data_tex
    }

    fn encode_data_tex_impl(
        &self,
        map: &HashMap<u64, u64>,
        data_tex: &mut Vec<u8>,
        data_tex_width: usize,
        offset_x: usize,
        offset_y: usize,
    ) -> Result<usize, EncodeError> {
        let mut len = 0usize;
        let glyph_width = self.extents.width();
        let glyph_height = self.extents.height();

        for v in map.values() {
            let unit_arc = unsafe { &mut *(*v as *mut UnitArc) };
            unit_arc.offset = len;

            if unit_arc.data.len() == 1 {
                assert!(unit_arc.data[0].line_encode.is_some());
                if let Some(data) = &unit_arc.data[0].line_encode {
                    write_data_tex(data_tex, data, &mut len, data_tex_width, offset_x, offset_y)?;
                }
            } else {
                for endpoint in &unit_arc.data {
                    let qx = quantize_x(endpoint.p[0], &self.extents, glyph_width);
                    let qy = quantize_y(endpoint.p[1], &self.extents, glyph_height);
                    let rgba = arc_endpoint_encode(qx, qy, endpoint.d);

                    write_data_tex(
                        data_tex,
                        &rgba,
                        &mut len,
                        data_tex_width,
                        offset_x,
                        offset_y,
                    )?;
                }
            }

            // 单元的端点个数超过 3 个，补充一个全零像素代表结束；
            if unit_arc.data.len() > 3 {
                write_data_tex(
                    data_tex,
                    &[0., 0., 0., 0.],
                    &mut len,
                    data_tex_width,
                    offset_x,
                    offset_y,
                )?;
            }
        }

        Ok(len)
    }

    pub fn encode_index_tex(
        &mut self,
        index_tex: &mut Vec<u8>,
        index_tex_width: usize,
        offset_x: &mut usize,
        offset_y: &mut usize,
        data_tex_map: HashMap<u64, u64>,
        data_tex_len: usize,
        sdf_tex: &mut Vec<u8>, // 字节数 = 4 * 像素个数
        sdf_tex1: &mut Vec<u8>,
        sdf_tex2: &mut Vec<u8>,
        sdf_tex3: &mut Vec<u8>,
    ) -> Result<TexInfo, EncodeError> {
        let max_offset = data_tex_len;
        // 计算sdf的 梯度等级
        let mut level = (2usize.pow(14) / max_offset) - 1;
        if level < 1 {
            level = 1;
        }
        let sdf_range = self.max_sdf - self.min_sdf + 0.1;
        // 量化：将 sdf_range 分成 level 个区间，看 sdf 落在哪个区间
        let sdf_step = sdf_range / level as f32;

        // 2 * grid_w * grid_h 个 Uint8
        for i in 0..self.data.len() {
            let len = self.data[i].len();
            for j in 0..len {
                // let unit_arc = &mut row[j];
                let key = self.data[i][j].get_key();
                if key != u64::MAX {
                    let map_arc_data = data_tex_map.get(&key);
                    if map_arc_data.is_none() {
                        panic!("unit_arc not found");
                    }
                    let map_arc_data = unsafe { &*((*map_arc_data.unwrap()) as *const UnitArc) };

                    let mut num_points = map_arc_data.data.len();

                    let _num_points2 = num_points;
                    if num_points > 3 {
                        num_points = 0;
                    }

                    let offset = map_arc_data.offset;
                    let sdf = self.data[i][j].sdf;

                    let cell_size = self.cell_size;
                    let is_interval = sdf.abs() <= cell_size * 0.5f32.sqrt();
                    let [encode, _] = encode_to_uint16(
                        is_interval,
                        num_points as f32,
                        offset as f32,
                        max_offset as f32,
                        sdf,
                        self.min_sdf,
                        sdf_step,
                    );
                    let offset_x = *offset_x;
                    let offset_y = *offset_y;

                    let sdf_index = (offset_x + j) + (offset_y + i) * index_tex_width;
                    let index = sdf_index * 2;
                    index_tex[index] = (encode as i32 & 0xff) as u8;
                    index_tex[index + 1] = (encode as i32 >> 8) as u8;

                    sdf_tex[sdf_index] = self.data[i][j].s_dist;

                    if i % 2 == 1 && j % 2 == 1 {
                        self.data[i][j].s_dist_1 = (self.data[i][j].s_dist as u64
                            + self.data[i - 1][j].s_dist as u64
                            + self.data[i][j - 1].s_dist as u64
                            + self.data[i - 1][j - 1].s_dist as u64)
                            / 4;

                        let index1 = (offset_x + j) / 2 + (offset_y + i) / 2 * index_tex_width / 2;
                        sdf_tex1[index1] = self.data[i][j].s_dist_1 as u8;

                        if i % 4 == 3 && j % 4 == 3 {
                            self.data[i][j].s_dist_2 = (self.data[i][j].s_dist_1 as u64
                                + self.data[i - 2][j].s_dist_1 as u64
                                + self.data[i][j - 2].s_dist_1 as u64
                                + self.data[i - 2][j - 2].s_dist_1 as u64)
                                / 4;

                            let index2 =
                                (offset_x + j) / 4 + (offset_y + i) / 4 * index_tex_width / 4;
                            sdf_tex2[index2] = self.data[i][j].s_dist_2 as u8;

                            if i % 8 == 7 && j % 8 == 7 {
                                self.data[i][j].s_dist_3 = (self.data[i][j].s_dist_2 as u64
                                    + self.data[i - 4][j].s_dist_2 as u64
                                    + self.data[i][j - 4].s_dist_2 as u64
                                    + self.data[i - 4][j - 4].s_dist_2 as u64)
                                    / 4;

                                let index3 =
                                    (offset_x + j) / 8 + (offset_y + i) / 8 * index_tex_width / 8;
                                sdf_tex3[index3] = self.data[i][j].s_dist_3 as u8;
                            }
                        }
                    }

                    // println!(
                    //     "i: {}, j: {}, sdf: {}, sdf1:{}",
                    //     i, j, self.data[i][j].s_dist, self.data[i][j].s_dist_1
                    // );
                    #[cfg(feature = "debug")]
                    {
                        self.data[i][j].show = format!("{}", self.data[i][j].s_dist);
                    }
                }
            }
        }

        let (grid_w, grid_h) = self.grid_size();

        *offset_x += grid_w as usize;
        if *offset_x >= index_tex_width {
            *offset_x = 0;
            *offset_y += grid_h as usize;
        }

        let cell_size = self.cell_size;
        #[cfg(feature = "debug")]
        self.show.push_str(&format!("<br> var max_offset = {:.2}, min_sdf = {:.2}, max_sdf = {:.2}, sdf_step = {:.2}, cell_size = {:.2} <br>", max_offset, self.min_sdf, self.max_sdf, sdf_step, cell_size));

        return Ok(TexInfo {
            // unitform
            cell_size,

            grid_w,
            grid_h,

            max_offset,

            min_sdf: self.min_sdf,
            sdf_step,
            char: char::default(),
            index_offset_x: 0,
            index_offset_y:0,
            data_offset_x: 0,
            data_offset_y: 0,
            extents_min_x: Default::default(),
            extents_min_y: Default::default(),
            extents_max_x: Default::default(),
            extents_max_y: Default::default(),
            binding_box_min_x: Default::default(),
            binding_box_min_y: Default::default(),
            binding_box_max_x: Default::default(),
            binding_box_max_y: Default::default(),
        });
    }

    pub fn encode_index_tex1(
        &mut self,
        data_tex_map: HashMap<u64, u64>,
        data_tex_len: usize,
    ) -> (TexInfo, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
        let max_offset = data_tex_len;
        // 计算sdf的 梯度等级
        let mut level = (2usize.pow(14) / max_offset) - 1;
        if level < 1 {
            level = 1;
        }
        let sdf_range = self.max_sdf - self.min_sdf + 0.1;
        // 量化：将 sdf_range 分成 level 个区间，看 sdf 落在哪个区间
        let sdf_step = sdf_range / level as f32;
        let (grid_w, grid_h) = self.grid_size();
        let (grid_w, grid_h) = (grid_w as usize, grid_h as usize);

        let mut index_tex: Vec<u8> = Vec::with_capacity(grid_w * grid_h * 2);

        let mut sdf_tex: Vec<u8> = Vec::with_capacity(grid_w * grid_h); //阴影用的minimip
        let mut sdf_tex1: Vec<u8> = Vec::with_capacity((grid_w >> 1) * (grid_h >> 1));
        let mut sdf_tex2: Vec<u8> = Vec::with_capacity((grid_w >> 2) * (grid_h >> 2));
        let mut sdf_tex3: Vec<u8> = Vec::with_capacity((grid_w >> 3) * (grid_h >> 3));

        // 2 * grid_w * grid_h 个 Uint8
        for i in 0..self.data.len() {
            let len = self.data[i].len();
            for j in 0..len {
                // let unit_arc = &mut row[j];
                let key = self.data[i][j].get_key();
                if key != u64::MAX {
                    let map_arc_data = data_tex_map.get(&key);
                    if map_arc_data.is_none() {
                        panic!("unit_arc not found");
                    }
                    let map_arc_data = unsafe { &*((*map_arc_data.unwrap()) as *const UnitArc) };

                    let mut num_points = map_arc_data.data.len();

                    let _num_points2 = num_points;
                    if num_points > 3 {
                        num_points = 0;
                    }

                    let offset = map_arc_data.offset;
                    let sdf = self.data[i][j].sdf;

                    let cell_size = self.cell_size;
                    let is_interval = sdf.abs() <= cell_size * 0.5f32.sqrt();
                    let [encode, _] = encode_to_uint16(
                        is_interval,
                        num_points as f32,
                        offset as f32,
                        max_offset as f32,
                        sdf,
                        self.min_sdf,
                        sdf_step,
                    );
                    let mut index = (j + i * grid_w) * 2;
                    while index > index_tex.len() {
                        index_tex.push(0);
                        index -= 0;
                    }
                    index_tex.push((encode as i32 & 0xff) as u8);
                    index_tex.push((encode as i32 >> 8) as u8);
                    // println!("index_tex[{}][{}]: {} {}", j, i, encode as i32 & 0xff, encode as i32 >> 8);
                    sdf_tex.push(self.data[i][j].s_dist);

                    if i % 2 == 1 && j % 2 == 1 {
                        self.data[i][j].s_dist_1 = (self.data[i][j].s_dist as u64
                            + self.data[i - 1][j].s_dist as u64
                            + self.data[i][j - 1].s_dist as u64
                            + self.data[i - 1][j - 1].s_dist as u64)
                            / 4;

                        sdf_tex1.push(self.data[i][j].s_dist_1 as u8);

                        if i % 4 == 3 && j % 4 == 3 {
                            self.data[i][j].s_dist_2 = (self.data[i][j].s_dist_1 as u64
                                + self.data[i - 2][j].s_dist_1 as u64
                                + self.data[i][j - 2].s_dist_1 as u64
                                + self.data[i - 2][j - 2].s_dist_1 as u64)
                                / 4;

                            sdf_tex2.push(self.data[i][j].s_dist_2 as u8);

                            if i % 8 == 7 && j % 8 == 7 {
                                self.data[i][j].s_dist_3 = (self.data[i][j].s_dist_2 as u64
                                    + self.data[i - 4][j].s_dist_2 as u64
                                    + self.data[i][j - 4].s_dist_2 as u64
                                    + self.data[i - 4][j - 4].s_dist_2 as u64)
                                    / 4;

                                sdf_tex3.push(self.data[i][j].s_dist_3 as u8);
                            }
                        }
                    }

                    #[cfg(feature = "debug")]
                    {
                        unit_arc.show = format!("{}", _num_points2);
                    }
                }
            }
        }

        let cell_size = self.cell_size;
        #[cfg(feature = "debug")]
        self.show.push_str(&format!("<br> var max_offset = {:.2}, min_sdf = {:.2}, max_sdf = {:.2}, sdf_step = {:.2}, cell_size = {:.2} <br>", max_offset, self.min_sdf, self.max_sdf, sdf_step, cell_size));

        return (
            TexInfo {
                // unitform
                cell_size,

                grid_w: grid_w as f32,
                grid_h: grid_h as f32,

                max_offset,

                min_sdf: self.min_sdf,
                sdf_step,
                char: char::default(),
                index_offset_x: 0,
                index_offset_y:0,
                data_offset_x: 0,
                data_offset_y: 0,
                extents_min_x: Default::default(),
                extents_min_y: Default::default(),
                extents_max_x: Default::default(),
                extents_max_y: Default::default(),
                binding_box_min_x: Default::default(),
                binding_box_min_y: Default::default(),
                binding_box_max_x: Default::default(),
                binding_box_max_y: Default::default(),
           
            },
            index_tex,
            sdf_tex,
            sdf_tex1,
            sdf_tex2,
            sdf_tex3,
            
        );
    }

    pub fn grid_size(&self) -> (f32, f32) {
        (
            (self.extents.width() / self.cell_size).round(),
            (self.extents.height() / self.cell_size).round(),
        )
    }
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

    if index >= loop_start_indies[loop_start_indies.len() - 1] {
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
    cp0: Point,
    cp1: Point,
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
    // let msg = format!("choose_best_arcs error: start_index = {}, sdf_sign = {}, cp0 = ({}, {}), cp1 = ({}, {}), arcs = ", start_index, sdf_sign, cp0.x, cp0.y, cp1.x, cp1.y) ;

    // log::warn!("{} {:?}, all endpoints = {:?}", msg, new_arcs, endpoints);
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
    cp0: Point,
    cp1: Point,
) -> (usize, Vec<ArcEndpoint>) {
    let mut r = vec![];

    for _i in 0..num {
        let endpoint = &endpoints[index];
        r.push(ArcEndpoint::new(endpoint.p[0], endpoint.p[1], endpoint.d));
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

/// 正方形的四个角落是否 全部 在 给定圆弧的 外面/里面
/// 返回有几个点的 符号 和 sdf_sign 相同
pub fn is_quad_same_sign(
    cp0: Point,
    cp1: Point,
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
pub fn is_point_same_sign(point: Point, endpoints: &Vec<ArcEndpoint>, sdf_sign: i32) -> bool {
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

#[cfg_attr(target_arch="wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone)]
pub struct TexData {
    pub index_tex: Vec<u8>, // 字节数 = 2 * 像素个数
    pub index_offset_x: usize,
    pub index_offset_y: usize,
    pub index_tex_width: usize,
    pub data_tex: Vec<u8>, // 字节数 = 4 * 像素个数
    pub data_offset_x: usize,
    pub data_offset_y: usize,
    pub data_tex_width: usize,

    pub sdf_tex: Vec<u8>, // 字节数 = 4 * 像素个数
    pub sdf_tex1: Vec<u8>,
    pub sdf_tex2: Vec<u8>,
    pub sdf_tex3: Vec<u8>,
}

// impl TexData{
//     pub fn new(index_tex: )
// }

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexInfo {
    pub grid_w: f32,
    pub grid_h: f32,

    pub cell_size: f32,

    pub max_offset: usize,
    pub min_sdf: f32,
    pub sdf_step: f32,
    
    pub index_offset_x: usize,
    pub index_offset_y: usize,
    pub data_offset_x: usize,
    pub data_offset_y: usize,
    pub char: char,
    pub extents_min_x: f32,
    pub extents_min_y: f32,
    pub extents_max_x: f32,
    pub extents_max_y: f32, 
    pub binding_box_min_x: f32,
    pub binding_box_min_y: f32,
    pub binding_box_max_x: f32,
    pub binding_box_max_y: f32,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
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

impl Default for TexInfo {
    fn default() -> Self {
        Self {
            grid_w: Default::default(),
            grid_h: Default::default(),
            cell_size: Default::default(),
            max_offset: Default::default(),
            min_sdf: Default::default(),
            sdf_step: Default::default(),
            char: char::default(),
            index_offset_x: Default::default(),
            index_offset_y: Default::default(),
            data_offset_x: Default::default(),
            data_offset_y: Default::default(),
            extents_min_x: Default::default(),
            extents_min_y: Default::default(),
            extents_max_x: Default::default(),
            extents_max_y: Default::default(),
            binding_box_min_x: Default::default(),
            binding_box_min_y: Default::default(),
            binding_box_max_x: Default::default(),
            binding_box_max_y: Default::default(),
            // ..
        }
    }
}

// 两张纹理，索引纹理 和 数据纹理
//
// 数据纹理：
//     32bit: [p.x, p.y, d]
//     按 数据 去重
//素，每像素 2B
// uniform: [max_offset, min_sdf,  索引纹理：共 grid_w * grid_h 个像sdf_step]

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

    if sdf_and_offset_index >= (2i32.pow(14)) as f32 {
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
    pub is_interval: bool,
    pub num_points: f32,
    _sdf: f32,
    pub offset: f32,
}
// value: u16，从高到低
// num_points: 2-bit
// offset + sdf: 14-bit
pub fn decode_from_uint16(value: f32, max_offset: f32, min_sdf: f32, sdf_step: f32) -> Res {
    let num_points = (value / 16384.0).floor();
    let sdf_and_offset_index = value % 16384.0;

    let mut sdf_index = (sdf_and_offset_index / max_offset).floor();
    let offset = sdf_and_offset_index % max_offset;

    let mut _sdf = 0.0;
    let mut is_interval = true;

    // 比实际的 sdf 范围多出 2
    // 用 0 表示 完全 在内 的 晶格！
    // 用 1 表示 完全 在外 的 晶格！
    if sdf_index == 0.0 {
        is_interval = false;
        _sdf = -GLYPHY_INFINITY;
    } else if sdf_index == 1.0 {
        is_interval = false;
        _sdf = GLYPHY_INFINITY;
    } else {
        sdf_index -= 2.0;
        _sdf = sdf_index * sdf_step + min_sdf;
    }

    return Res {
        is_interval,
        num_points,
        _sdf,
        offset,
    };
}

fn get_offset(
    len: usize,
    tex_width: usize,
    offset_x: usize,
    offset_y: usize,
) -> Option<(usize, usize)> {
    let x: usize = len / 8 + offset_x;
    if x >= tex_width {
        return None;
    }

    let y = (len % 8) + offset_y;
    // println!("x: {}, y: {}", x, y);
    Some((x, y))
}

fn write_data_tex(
    data_tex: &mut Vec<u8>,
    src_data: &[f32; 4],
    len: &mut usize,
    data_tex_width: usize,
    offset_x: usize,
    offset_y: usize,
) -> Result<(), EncodeError> {
    if let Some((x, y)) = get_offset(*len, data_tex_width, offset_x, offset_y) {
        // println!("x: {}, y: {}", x, y);
        let offset = (x + y * data_tex_width) * 4;
        *len = *len + 1;

        if data_tex.get(offset).is_none() {
            return Err(EncodeError::MemoryOverflow);
        }

        data_tex[offset] = src_data[0] as u8;
        data_tex[offset + 1] = src_data[1] as u8;
        data_tex[offset + 2] = src_data[2] as u8;
        data_tex[offset + 3] = src_data[3] as u8;
    } else {
        return Err(EncodeError::NewLine);
    }
    Ok(())
}

// 定宽（宽度为8）
fn write_data_tex_by_width(
    data_tex: &mut Vec<u8>,
    src_data: &[f32; 4],
    len: &mut usize,
    // data_tex_height: usize,
) {
    // let (x, y) = get_offset_by_width(*len);
    // let offset = (y + x * 8) * 4;
    // *len = *len + 1;

    // if data_tex.get(offset).is_none() {
    // 	return Err(EncodeError::MemoryOverflow);
    // }

    // data_tex[offset] = src_data[0] as u8;
    // data_tex[offset + 1] = src_data[1] as u8;
    // data_tex[offset + 2] = src_data[2] as u8;
    // data_tex[offset + 3] = src_data[3] as u8;

    data_tex.push(src_data[0] as u8);
    data_tex.push(src_data[1] as u8);
    data_tex.push(src_data[2] as u8);
    data_tex.push(src_data[3] as u8);
    *len = *len + 1;
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

pub fn snap(p: &Point, extents: &Aabb, glyph_width: f32, glyph_height: f32) -> Point {
    let qx = quantize_x(p.x, extents, glyph_width);
    let x = dequantize_x(qx, extents, glyph_width);

    let qy = quantize_y(p.y, extents, glyph_height);
    let y = dequantize_y(qy, extents, glyph_height);

    return Point::new(x, y);
}

// const upper_bits = (v: f32, bits: f32, total_bits: f32): f32 => {
// 	return v >> (total_bits - bits);
// }

pub fn lower_bits(v: f32, bits: f32, _total_bits: f32) -> f32 {
    return (v as i32 & ((1 << bits as i32) - 1)) as f32;
}

// 将 一个圆弧端点 编码为 RGBA, 4个字节
pub fn arc_endpoint_encode(ix: f32, iy: f32, d: f32) -> [f32; 4] {
    if ix > MAX_X {
        panic!("ix must be less than or equal to MAX_X");
    }
    if iy > MAX_Y {
        panic!("iy must be less than or equal to MAX_Y");
    }
    let id;
    if is_inf(d) {
        id = 0.0;
    } else {
        if d.abs() > GLYPHY_MAX_D {
            panic!(
                "d must be less than or equal to GLYPHY_MAX_D, d: {}, GLYPHY_MAX_D: {}",
                d.abs(),
                GLYPHY_MAX_D
            );
        }

        id = 128. + (d * 127.0 / GLYPHY_MAX_D).round();
    }
    if id >= 256.0 {
        panic!("id must be less than 256");
    }
    let r = id as i32;
    let g = lower_bits(ix, 8.0, 12.0);
    let b = lower_bits(iy, 8.0, 12.0);
    let a = ((ix as i32 >> 8) << 4) | (iy as i32 >> 8);

    return [r as f32, g, b, a as f32];
}

pub fn travel_data(data: &Vec<Vec<UnitArc>>) -> [f32; 2] {
    let mut min_sdf = f32::INFINITY;
    let mut max_sdf = -f32::INFINITY;

    // 初始化队列
    for i in 0..data.len() {
        let row = &data[i];
        for j in 0..row.len() {
            let unit_arc = &row[j];
            let curr_dist = unit_arc.sdf;

            if curr_dist < min_sdf {
                min_sdf = curr_dist;
            }
            if curr_dist > max_sdf {
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

// 判断 每个 格子 最近的 圆弧
pub fn recursion_near_arcs_of_cell<'a>(
    extents: &Aabb,
    cell: &Aabb,
    arcs: &Vec<&'static Arc>,
    min_width: &mut f32,
    min_height: &mut f32,
    top_near: Option<(Vec<&'static Arc>, bool)>,
    bottom_near: Option<(Vec<&'static Arc>, bool)>,
    left_near: Option<(Vec<&'static Arc>, bool)>,
    right_near: Option<(Vec<&'static Arc>, bool)>,
    result_arcs: &mut Vec<(Vec<Arc>, Aabb)>,
    temps: &mut Vec<(Point, f32, Vec<Range<f32>>)>,
) {
    // let time = std::time::Instant::now();
    let cell_width = cell.width();
    let cell_height = cell.height();
    if *min_width > cell_width {
        *min_width = cell_width;
    }

    if *min_height > cell_height {
        *min_height = cell_height;
    }

    let (near_arcs, top_near, bottom_near, left_near, right_near) = compute_near_arc(
        cell,
        arcs,
        top_near,
        bottom_near,
        left_near,
        right_near,
        temps,
    );
    let mut arcs: Vec<&Arc> =
        Vec::with_capacity(near_arcs.len() + top_near.len() + bottom_near.len() + right_near.len());
    arcs.extend(&near_arcs);
    arcs.extend(&top_near);
    arcs.extend(&bottom_near);
    arcs.extend(&left_near);
    arcs.extend(&right_near);
    arcs.sort_by(|a, b| a.id.cmp(&b.id));

    arcs.dedup_by(|a, b| a.id == b.id);

    let glyph_width = extents.width();
    let glyph_height = extents.height();
    if (
        arcs.len() <= 2
        // && float_equals(cell_width, cell_height, Some(0.01))
    ) || (cell_width * 32.0 - glyph_width).abs() < 0.1
        && (cell_height * 32.0 - glyph_height).abs() < 0.1
    {
        let arcs = arcs.iter().map(|item|(*item).clone()).collect();
        result_arcs.push((arcs, cell.clone()));
    } else {
        let (
            (cell1, cell2),
            (top_near1, bottom_near1, left_near1, right_near1),
            (top_near2, bottom_near2, left_near2, right_near2),
        ) = if cell_width > cell_height && cell_width * 32.0 > glyph_width {
            let (ab1, ab2) = cell.half(Direction::Col);

            let col_area = cell.near_area(Direction::Col);

            let mut near_arcs = Vec::with_capacity(arcs.len());
            let right_segment = ab1.bound(Direction::Right);
            col_area.near_arcs(&arcs, &right_segment, &mut near_arcs, temps);

            (
                (ab1, ab2),
                (
                    Some((top_near.clone(), false)),
                    Some((bottom_near.clone(), false)),
                    Some((left_near, true)),
                    Some((near_arcs.clone(), true)),
                ),
                (
                    Some((top_near, false)),
                    Some((bottom_near, false)),
                    Some((near_arcs, true)),
                    Some((right_near, true)),
                ),
            )
        } else {
            let (ab1, ab2) = cell.half(Direction::Row);

            let col_area = ab1.near_area(Direction::Row);

            let mut near_arcs = Vec::with_capacity(arcs.len());
            let bottom_segment = ab1.bound(Direction::Bottom);
            col_area.near_arcs(&arcs, &bottom_segment, &mut near_arcs, temps);

            (
                (ab1, ab2),
                (
                    Some((top_near, true)),
                    Some((near_arcs.clone(), true)),
                    Some((left_near.clone(), false)),
                    Some((right_near.clone(), false)),
                ),
                (
                    Some((near_arcs, true)),
                    Some((bottom_near, true)),
                    Some((left_near, false)),
                    Some((right_near, false)),
                ),
            )
        };
        // println!("cell1: {:?}, cell2: {:?}, cell: {:?}", cell1, cell2, cell);
        recursion_near_arcs_of_cell(
            extents,
            &cell1,
            &near_arcs,
            min_width,
            min_height,
            top_near1,
            bottom_near1,
            left_near1,
            right_near1,
            result_arcs,
            temps,
        );
        recursion_near_arcs_of_cell(
            extents,
            &cell2,
            &near_arcs,
            min_width,
            min_height,
            top_near2,
            bottom_near2,
            left_near2,
            right_near2,
            result_arcs,
            temps,
        );
    }
}

fn compute_near_arc(
    cell: &Aabb,
    arcs: &Vec<&'static Arc>,
    mut top_near: Option<(Vec<&'static Arc>, bool)>,
    mut bottom_near: Option<(Vec<&'static Arc>, bool)>,
    mut left_near: Option<(Vec<&'static Arc>, bool)>,
    mut right_near: Option<(Vec<&'static Arc>, bool)>,
    temps: &mut Vec<(Point, f32, Vec<Range<f32>>)>,
) -> (
    Vec<&'static Arc>,
    Vec<&'static Arc>,
    Vec<&'static Arc>,
    Vec<&'static Arc>,
    Vec<&'static Arc>,
) {
    let c = cell.center();
    // 最近的意思：某个半径的 圆内
    let radius_squared = cell.half_extents().norm_squared();

    let mut near_arcs: Vec<&'static Arc> = Vec::with_capacity(arcs.len());
    for arc in arcs {
        if arc.squared_distance_to_point2(&c).norm_squared() <= radius_squared {
            near_arcs.push(*arc);
        }
    }
    // println!("near_arcs: {:?}", near_arcs);
    let row_area = cell.near_area(Direction::Row);

    let top_near = if let Some((near, is_use)) = top_near.take() {
        if is_use {
            near
        } else {
            let mut near_arcs = Vec::with_capacity(near.len());
            let top_segment = cell.bound(Direction::Top);
            row_area.near_arcs(&near, &top_segment, &mut near_arcs, temps);
            near_arcs
        }
    } else {
        let mut top_near = Vec::with_capacity(arcs.len());
        let top_segment = cell.bound(Direction::Top);
        row_area.near_arcs(arcs, &top_segment, &mut top_near, temps);
        top_near
    };

    let bottom_near = if let Some((near, is_use)) = bottom_near.take() {
        if is_use {
            near
        } else {
            let mut near_arcs = Vec::with_capacity(near.len());
            let bottom_segment = cell.bound(Direction::Bottom);
            row_area.near_arcs(&near, &bottom_segment, &mut near_arcs, temps);
            near_arcs
        }
    } else {
        let mut near_arcs = Vec::with_capacity(arcs.len());
        let bottom_segment = cell.bound(Direction::Bottom);
        row_area.near_arcs(arcs, &bottom_segment, &mut near_arcs, temps);
        near_arcs
    };

    let col_area = cell.near_area(Direction::Col);

    let left_near = if let Some((near, is_use)) = left_near.take() {
        if is_use {
            near
        } else {
            let mut near_arcs = Vec::with_capacity(near.len());
            let left_segment = cell.bound(Direction::Left);
            col_area.near_arcs(&near, &left_segment, &mut near_arcs, temps);
            near_arcs
        }
    } else {
        let mut near_arcs = Vec::with_capacity(arcs.len());
        let left_segment = cell.bound(Direction::Left);
        col_area.near_arcs(arcs, &left_segment, &mut near_arcs, temps);
        near_arcs
    };

    let right_near = if let Some((near, is_use)) = right_near.take() {
        if is_use {
            near
        } else {
            let mut near_arcs = Vec::with_capacity(near.len());
            let right_segment = cell.bound(Direction::Right);
            col_area.near_arcs(&near, &right_segment, &mut near_arcs, temps);
            near_arcs
        }
    } else {
        let mut near_arcs = Vec::with_capacity(arcs.len());
        let right_segment = cell.bound(Direction::Right);
        col_area.near_arcs(arcs, &right_segment, &mut near_arcs, temps);
        near_arcs
    };

    (near_arcs, top_near, bottom_near, left_near, right_near)
}
