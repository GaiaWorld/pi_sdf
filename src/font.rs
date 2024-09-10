use std::{char, collections::HashMap};

use crate::glyphy::blob::TexInfo2;
use allsorts::{
    binary::read::ReadScope,
    font::MatchingPresentation,
    font_data::{DynamicFontTableProvider, FontData},
    outline::OutlineBuilder,
    tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider, HeadTable},
    tag, Font,
};
use pi_share::Share;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    glyphy::{
        blob::{recursion_near_arcs_of_cell, travel_data, BlobArc, EncodeError, TexData, TexInfo},
        geometry::{
            aabb::{Aabb, AabbEXT, Direction},
            arc::{Arc, ArcEndpoint},
            arcs::GlyphyArcAccumulator,
        },
        outline::{self, glyphy_outline_winding_from_even_odd},
        util::GLYPHY_INFINITY,
    },
    utils::{
        compute_layout, encode_uint_arc_data, GlyphInfo, GlyphVisitor, OutlineInfo, SCALE,
        TOLERANCE,
    },
    Point,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct FontFace {
    pub(crate) _data: Share<Vec<u8>>,
    pub(crate) font: Font<DynamicFontTableProvider<'static>>,
    pub(crate) glyf: GlyfTable<'static>,
    _glyf_data: Vec<u8>,
    _loca_data: Vec<u8>,
    pub(crate) _loca: LocaTable<'static>,
    pub(crate) max_box: Aabb,
    pub(crate) max_box_normaliz: Aabb,
    pub(crate) units_per_em: u16,
}

impl FontFace {
    pub fn font(&self) -> &Font<DynamicFontTableProvider> {
        &self.font
    }

    pub fn verties(&self, _font_size: f32, _shadow_offsett: &mut [f32]) -> [f32; 16] {
        let extents = self.max_box_normaliz.clone();

        // let offset_x = shadow_offset[0] / font_size;
        // let offset_y = shadow_offset[1] / font_size;
        // shadow_offset[0] = offset_x;
        // shadow_offset[1] = offset_y;

        // let width = extents.width();
        // let height = extents.height();

        let min_uv = [0.0f32, 0.0];
        let max_uv = [1.0f32, 1.0];
        // if offset_x < 0.0 {
        //     extents.mins.x += offset_x;
        //     min_uv[0] += offset_x / width;
        // } else {
        //     extents.maxs.x += offset_x;
        //     max_uv[0] += offset_x / width;
        // }
        // if offset_y < 0.0 {
        //     extents.mins.y += offset_y;
        //     min_uv[1] += offset_y / height;
        // } else {
        //     extents.maxs.y += offset_y;
        //     max_uv[1] += offset_y / height;
        // }

        [
            extents.mins.x,
            extents.mins.y,
            min_uv[0],
            min_uv[1],
            extents.mins.x,
            extents.maxs.y,
            min_uv[0],
            max_uv[1],
            extents.maxs.x,
            extents.mins.y,
            max_uv[0],
            min_uv[1],
            extents.maxs.x,
            extents.maxs.y,
            max_uv[0],
            max_uv[1],
        ]
    }

    fn get_max_box_normaliz(head_table: &HeadTable) -> Aabb {
        let mut extents = Aabb::new(
            Point::new(head_table.x_min as f32, head_table.y_min as f32),
            Point::new(head_table.x_max as f32, head_table.y_max as f32),
        );
        // println!("extents: {:?}", extents);
        // let per_em = TOLERANCE;

        // let upem = head_table.units_per_em as f32;
        // let tolerance = upem * per_em; /* in font design units */
        // let faraway = upem / 32.0; //upem / (MIN_FONT_SIZE * 2.0f32.sqrt());
        // let embolden_max = upem / 32.0;
        // 抗锯齿需要
        // extents.mins.x -= 128.0;
        // extents.mins.y -= 128.0;
        // extents.maxs.x += 128.0;
        // extents.maxs.y += 128.0;

        let glyph_width = extents.maxs.x - extents.mins.x;
        let glyph_height = extents.maxs.y - extents.mins.y;
        if glyph_width > glyph_height {
            extents.maxs.y = extents.mins.y + glyph_width;
        } else {
            extents.maxs.x = extents.mins.x + glyph_height;
        };
        // extents.maxs.x +=  2048.0;
        // extents.maxs.y +=  2048.0;
        extents.scale(
            1.0 / head_table.units_per_em as f32,
            1.0 / head_table.units_per_em as f32,
        );
        extents
    }

    pub fn encode_uint_arc(
        extents: Aabb,
        mut endpoints: Vec<ArcEndpoint>,
    ) -> (BlobArc, HashMap<u64, u64>) {
        // println!("result_arcs: {:?}", result_arcs.len());

        // let width_cells = (extents.width() / min_width).floor();
        // let height_cells = (extents.height() / min_height).floor();
        // 根据最小格子大小计算每个格子的圆弧数据
        let (result_arcs, min_width, min_height, near_arcs) =
            Self::compute_near_arcs(extents, &mut endpoints);
        log::trace!("near_arcs: {}", near_arcs.len());
        let (unit_arcs, map) =
            encode_uint_arc_data(result_arcs, &extents, min_width, min_height, None);
        // println!("unit_arcs[14][5]: {:?}", unit_arcs[14][5]);

        let [min_sdf, max_sdf] = travel_data(&unit_arcs);
        let blob_arc = BlobArc {
            min_sdf,
            max_sdf,
            cell_size: min_width,
            #[cfg(feature = "debug")]
            show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
            extents,
            data: unit_arcs,
            avg_fetch_achieved: 0.0,
            endpoints,
        };

        // extents.scale(1.0 / upem, 1.0 / upem);

        // gi.nominal_w = width_cells;
        // gi.nominal_h = height_cells;

        // gi.extents.set(&extents);

        (blob_arc, map)
    }

    pub fn compute_near_arcs<'a>(
        extents: Aabb,
        endpoints: &mut Vec<ArcEndpoint>,
    ) -> (Vec<(Vec<&'a Arc>, Aabb)>, f32, f32, Vec<Arc>) {
        // log::error!("get_char_arc: {:?}", char);
        // let extents = self.max_box.clone();
        // let endpoints = &mut endpoints;
        // let r = endpoints.len();
        // println!("endpoints: {}",  endpoints.len());
        if endpoints.len() > 0 {
            // 用奇偶规则，计算 每个圆弧的 环绕数
            glyphy_outline_winding_from_even_odd(endpoints, false);
        }
        // println!("extents: {:?}", extents);

        let mut min_width = f32::INFINITY;
        let mut min_height = f32::INFINITY;

        let mut p0 = Point::new(0., 0.);

        // 将圆弧控制点变成圆弧
        let mut near_arcs = Vec::with_capacity(endpoints.len());
        let mut arcs = Vec::with_capacity(endpoints.len());
        for i in 0..endpoints.len() {
            let endpoint = &endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = Point::new(endpoint.p[0], endpoint.p[1]);
                continue;
            }
            let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);

            near_arcs.push(arc);
            arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
        }

        let mut result_arcs = vec![];
        let mut temp = Vec::with_capacity(arcs.len());
        let (ab1, ab2) = extents.half(Direction::Col);
        // 二分法递归细分格子，知道格子周围的圆弧数量小于二或者小于32/1停止
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
        (result_arcs, min_width, min_height, near_arcs)
    }

    pub fn out_tex_data(
        &mut self,
        text: &str,
        tex_data: &mut TexData,
    ) -> Result<Vec<TexInfo>, EncodeError> {
        let mut infos = Vec::with_capacity(text.len());
        let text = text.chars();

        let data_tex = &mut tex_data.data_tex;
        let width0 = tex_data.data_tex_width;
        let offset_x0 = &mut tex_data.data_offset_x;
        let offset_y0 = &mut tex_data.data_offset_y;

        let index_tex = &mut tex_data.index_tex;
        let width1 = tex_data.index_tex_width;
        let offset_x1 = &mut tex_data.index_offset_x;
        let offset_y1 = &mut tex_data.index_offset_y;
        let mut last_offset1 = (*offset_x1, *offset_x1);

        let sdf_tex = &mut tex_data.sdf_tex;
        let sdf_tex1 = &mut tex_data.sdf_tex1;
        let sdf_tex2 = &mut tex_data.sdf_tex2;
        let sdf_tex3 = &mut tex_data.sdf_tex3;

        for char in text {
            // println!("char: {}", char);
            let result = self.to_outline(char);
            let (mut blod_arc, map) = Self::encode_uint_arc(self.max_box.clone(), result);
            let size = blod_arc.encode_data_tex(&map, data_tex, width0, offset_x0, offset_y0)?;
            // println!("data_map: {}", map.len());
            let mut info = blod_arc.encode_index_tex(
                index_tex, width1, offset_x1, offset_y1, map, size, sdf_tex, sdf_tex1, sdf_tex2,
                sdf_tex3,
            )?;

            info.index_offset_x = last_offset1.0;
            info.index_offset_y = last_offset1.1;
            info.data_offset_x = *offset_x0;
            info.data_offset_y = *offset_y0;

            *offset_x0 += size / 8;
            if size % 8 != 0 {
                *offset_x0 += 1;
            }
            // println!("info.index_offset: {:?}", info.index_offset);
            last_offset1 = (*offset_x1, *offset_y1);

            infos.push(info);
        }

        Ok(infos)
    }

    pub fn compute_sdf(max_box: Aabb, endpoints: Vec<ArcEndpoint>) -> SdfInfo {
        // log::error!("endpoints.len(): {}", endpoints.len());
        // map 无序导致每次计算的数据不一样
        let (mut blod_arc, map) = Self::encode_uint_arc(max_box, endpoints);
        // println!("data_map: {}", map.len());
        let data_tex = blod_arc.encode_data_tex1(&map);
        let (tex_info, index_tex, sdf_tex1, sdf_tex2, sdf_tex3, sdf_tex4) =
            blod_arc.encode_index_tex1(map, data_tex.len() / 4);
        let grid_size = blod_arc.grid_size();

        SdfInfo {
            tex_info,
            data_tex,
            index_tex,
            sdf_tex1,
            sdf_tex2,
            sdf_tex3,
            sdf_tex4,
            grid_size: vec![grid_size.0, grid_size.1],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct SdfInfo {
    pub tex_info: TexInfo,
    pub data_tex: Vec<u8>,
    pub index_tex: Vec<u8>,
    pub sdf_tex1: Vec<u8>,
    pub sdf_tex2: Vec<u8>,
    pub sdf_tex3: Vec<u8>,
    pub sdf_tex4: Vec<u8>,
    pub grid_size: Vec<f32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SdfInfo2 {
    pub tex_info: TexInfo2,
    pub sdf_tex: Vec<u8>,
    pub tex_size: usize,
}

impl FontFace {
    pub fn new_inner(_data: Share<Vec<u8>>) -> Self {
        let _ = console_log::init_with_level(log::Level::Info);
        // log::info!("=========== 1, : {}", _data.len());
        let d: &'static Vec<u8> = unsafe { std::mem::transmute(_data.as_ref()) };
        let scope = ReadScope::new(d);
        let font_file = scope.read::<FontData<'static>>().unwrap();
        // font_file.table_provider(index)
        // log::info!("=========== 2");
        let provider = font_file.table_provider(0).unwrap();
        let font: Font<DynamicFontTableProvider<'static>> = Font::new(provider).unwrap().unwrap();
        // log::info!("=========== 3");
        let head_table = font
            .head_table()
            .unwrap()
            .ok_or("missing head table")
            .unwrap();

        // log::info!("=========== 4");
        let max_box_normaliz = Self::get_max_box_normaliz(&head_table);
        let _loca_data = font
            .font_table_provider
            .read_table_data(tag::LOCA)
            .unwrap()
            .to_vec();
        // log::info!("=========== 5");
        let l: &'static Vec<u8> = unsafe { std::mem::transmute(&_loca_data) };
        // log::info!("=========== 6");
        let loca = ReadScope::new(&l)
            .read_dep::<LocaTable<'_>>((
                usize::from(font.maxp_table.num_glyphs),
                head_table.index_to_loc_format,
            ))
            .unwrap();
        let _loca: LocaTable<'static> = unsafe { std::mem::transmute(loca) };
        let loca_ref = unsafe { std::mem::transmute(&_loca) };
        // log::info!("=========== 7");
        let _glyf_data = font
            .font_table_provider
            .read_table_data(tag::GLYF)
            .unwrap()
            .to_vec();
        let g: &'static Vec<u8> = unsafe { std::mem::transmute(&_glyf_data) };
        // log::info!("=========== 8");
        let glyf = ReadScope::new(g)
            .read_dep::<GlyfTable<'_>>(loca_ref)
            .unwrap();
        // log::info!("=========== 9");
        let mut extents = max_box_normaliz.clone();
        extents.scale(SCALE, SCALE);

        // 抗锯齿需要
        // extents.mins.x -= 128.0;
        // extents.mins.y -= 128.0;
        // extents.maxs.x += 128.0;
        // extents.maxs.y += 128.0;

        // log::info!("=========== 10");
        // todo!()
        println!("units_per_em: {}", head_table.units_per_em);
        Self {
            _data,
            font,
            glyf,
            _glyf_data,
            _loca,
            _loca_data,
            max_box_normaliz,
            max_box: extents,
            units_per_em: head_table.units_per_em,
        }
    }
}

// pub struct SdfInfos

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl FontFace {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_data: Share<Vec<u8>>) -> Self {
        Self::new_inner(_data)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(_data: Vec<u8>) -> Self {
        let data = Share::new(_data);
        Self::new_inner(data)
    }

    pub fn compute_text_sdf(&mut self, text: &str) -> Vec<SdfInfo> {
        let mut info = Vec::with_capacity(text.len());
        for char in text.chars() {
            let result = self.to_outline(char);
            let mut v = Self::compute_sdf(self.max_box.clone(), result);
            v.tex_info.char = char;
            info.push(v);
        }
        info
    }

    pub fn compute_sdf2(max_box: Vec<f32>, endpoints: Vec<u8>) -> Vec<u8> {
        let max_box = Aabb::new(
            Point::new(max_box[0], max_box[1]),
            Point::new(max_box[2], max_box[3]),
        );
        let endpoints: Vec<ArcEndpoint> = bincode::deserialize(&endpoints).unwrap();
        bincode::serialize(&Self::compute_sdf(max_box, endpoints)).unwrap()
    }

    /// 水平宽度
    pub fn horizontal_advance(&mut self, char: char) -> f32 {
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(char, MatchingPresentation::NotRequired, None);
        if glyph_index != 0 {
            // self.font
            match self.font.horizontal_advance(glyph_index) {
                Some(r) => return r as f32 / self.units_per_em as f32,
                None => return 0.0,
            }
        } else {
            return 0.0;
        }
    }

    pub fn ascender(&self) -> f32 {
        self.font.hhea_table.ascender as f32 / self.units_per_em as f32
    }

    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    pub fn descender(&self) -> f32 {
        self.font.hhea_table.descender as f32 / self.units_per_em as f32
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn max_box(&self) -> Aabb {
        self.max_box.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn max_box(&self) -> Vec<f32> {
        vec![
            self.max_box.mins.x,
            self.max_box.mins.y,
            self.max_box.maxs.x,
            self.max_box.maxs.y,
        ]
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn max_box_normaliz(&self) -> Aabb {
        self.max_box_normaliz.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn max_box_normaliz(&self) -> Vec<f32> {
        vec![
            self.max_box_normaliz.mins.x,
            self.max_box_normaliz.mins.y,
            self.max_box_normaliz.maxs.x,
            self.max_box_normaliz.maxs.y,
        ]
    }

    pub fn to_outline(&mut self, ch: char) -> Vec<ArcEndpoint> {
        let OutlineInfo { endpoints, .. } = self.to_outline3(ch);
        endpoints
    }

    pub fn to_outline2(&mut self, ch: char) -> Vec<u8> {
        let OutlineInfo { endpoints, .. } = self.to_outline3(ch);
        bincode::serialize(&endpoints).unwrap()
    }

    pub fn glyph_index(&mut self, ch: char) -> u16 {
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        glyph_index
    }

    pub fn debug_size(&self) -> usize {
        self._data.len()
    }

    pub fn to_outline3(&mut self, ch: char) -> OutlineInfo {
        let mut sink = GlyphVisitor::new(SCALE / self.units_per_em as f32);
        sink.accumulate.tolerance = self.units_per_em as f32 * TOLERANCE;

        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        assert_ne!(glyph_index, 0);
        let advance = self.font.horizontal_advance(glyph_index).unwrap();

        let _ = self.glyf.visit(glyph_index, &mut sink);
        let GlyphVisitor {
            accumulate, bbox, ..
        } = sink;

        let GlyphyArcAccumulator { result, .. } = accumulate;

        OutlineInfo {
            endpoints: result,
            bbox,
            advance,
            units_per_em: self.units_per_em,
            char: ch,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn compute_sdf_tex(
        outline_info: OutlineInfo,
        tex_size: usize, // 需要计算纹理的宽高，默认正方形，像素为单位
        pxrange: u32,
        is_outer_glow: bool,
    ) -> SdfInfo2 {
        let OutlineInfo {
            char,
            mut endpoints,
            bbox,
            advance,
            units_per_em,
        } = outline_info;
        let mut extents = bbox;

        let (plane_bounds, atlas_bounds, distance, tex_size) =
            compute_layout(&mut extents, tex_size, pxrange, units_per_em);
        // println!("pxrange: {}, tex_size: {}", pxrange, tex_size);
        let (result_arcs, _, _, near_arcs) = Self::compute_near_arcs(extents, &mut endpoints);
        log::trace!("near_arcs: {}", near_arcs.len());

        let pixmap =
            crate::utils::encode_sdf(result_arcs, &extents, tex_size, tex_size, distance, None, is_outer_glow, false);
        SdfInfo2 {
            tex_info: TexInfo2 {
                char,
                advance: advance as f32 / units_per_em as f32,
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

    #[cfg(target_arch = "wasm32")]
    pub fn compute_sdf_tex(
        outline_info: OutlineInfo,
        tex_size: usize, // 需要计算纹理的宽高，默认正方形，像素为单位
        pxrange: u32,
    ) -> Vec<u8> {
        let OutlineInfo {
            char,
            mut endpoints,
            bbox,
            advance,
            units_per_em,
        } = outline_info;
        let mut extents = bbox;

        let (plane_bounds, atlas_bounds, distance, tex_size) =
            compute_layout(&mut extents, tex_size, pxrange, units_per_em);
        // println!("pxrange: {}, tex_size: {}", pxrange, tex_size);
        let (result_arcs, _, _, near_arcs) = Self::compute_near_arcs(extents, &mut endpoints);
        log::trace!("near_arcs: {}", near_arcs.len());

        let pixmap =
            crate::utils::encode_sdf(result_arcs, &extents, tex_size, tex_size, distance, None);
        let info = GlyphInfo {
            char,
            advance: advance as f32 / units_per_em as f32,
            plane_bounds: [
                plane_bounds.mins.x,
                plane_bounds.mins.y,
                plane_bounds.maxs.x,
                plane_bounds.maxs.y,
            ],
            atlas_bounds: [
                atlas_bounds.mins.x,
                atlas_bounds.mins.y,
                atlas_bounds.maxs.x,
                atlas_bounds.maxs.y,
            ],
            sdf_tex: pixmap,
            tex_size: tex_size as u32,
        };
        bincode::serialize(&info).unwrap()
    }
}
