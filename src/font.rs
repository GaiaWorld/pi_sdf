use std::{char, collections::HashMap};

use allsorts::{
    binary::read::ReadScope,
    font::MatchingPresentation,
    font_data::{DynamicFontTableProvider, FontData},
    outline::OutlineBuilder,
    tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider, HeadTable},
    tag, Font,
};
// use freetype_sys::FT_Vector;

use parry2d::bounding_volume::Aabb;
use wasm_bindgen::prelude::wasm_bindgen;
// use parry2d::math::Point;

use crate::{
    glyphy::{
        blob::{recursion_near_arcs_of_cell, travel_data, BlobArc, EncodeError, TexData, TexInfo},
        geometry::{
            aabb::{AabbEXT, Direction},
            arc::Arc,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::GLYPHY_INFINITY,
    },
    utils::{encode_uint_arc_data, GlyphVisitor, EMBOLDEN_MAX, MIN_FONT_SIZE, TOLERANCE},
    Point,
};

#[wasm_bindgen]
pub struct FontFace {
    pub(crate) _data: Vec<u8>,
    pub(crate) font: Font<DynamicFontTableProvider<'static>>,
    pub(crate) glyf: GlyfTable<'static>,
    _glyf_data: Vec<u8>,
    _loca_data: Vec<u8>,
    pub(crate) _loca: LocaTable<'static>,
    pub(crate) max_box: Aabb,
    pub(crate) units_per_em: u16,
}

impl FontFace {
    pub fn new(_data: Vec<u8>) -> Self {
        let d: &'static Vec<u8> = unsafe { std::mem::transmute(&_data) };
        let scope = ReadScope::new(d);
        let font_file = scope.read::<FontData<'static>>().unwrap();
        // font_file.table_provider(index)

        let provider = font_file.table_provider(0).unwrap();
        let font: Font<DynamicFontTableProvider<'static>> = Font::new(provider).unwrap().unwrap();

        let head_table = font
            .head_table()
            .unwrap()
            .ok_or("missing head table")
            .unwrap();

        let extents = Self::get_max_box(&head_table);
        let _loca_data = font
            .font_table_provider
            .read_table_data(tag::LOCA)
            .unwrap()
            .to_vec();
        let l: &'static Vec<u8> = unsafe { std::mem::transmute(&_loca_data) };

        let loca = ReadScope::new(&l)
            .read_dep::<LocaTable<'_>>((
                usize::from(font.maxp_table.num_glyphs),
                head_table.index_to_loc_format,
            ))
            .unwrap();
        let _loca: LocaTable<'static> = unsafe { std::mem::transmute(loca) };
        let loca_ref = unsafe { std::mem::transmute(&_loca) };

        let _glyf_data = font
            .font_table_provider
            .read_table_data(tag::GLYF)
            .unwrap()
            .to_vec();
        let g: &'static Vec<u8> = unsafe { std::mem::transmute(&_glyf_data) };

        let glyf = ReadScope::new(g)
            .read_dep::<GlyfTable<'_>>(loca_ref)
            .unwrap();
        // todo!()
        Self {
            _data,
            font,
            glyf,
            _glyf_data,
            _loca,
            _loca_data,
            max_box: extents,
            units_per_em: head_table.units_per_em,
        }
    }

	pub fn font(&self) -> &Font<DynamicFontTableProvider> {
		&self.font
	}

	/// 水平宽度
	pub fn horizontal_advance(&mut self, char: char) -> f32 {
		let (glyph_index, _) =
		self.font
			.lookup_glyph_index(char, MatchingPresentation::NotRequired, None);
		match self.font.horizontal_advance(glyph_index) {
			Some(r) => r as f32 / self.units_per_em as f32,
			None => 0.0,
		}
	}

	pub fn ascender(&self) -> f32 {
		self.font.hhea_table.ascender as f32 / self.units_per_em as f32
	}

	pub fn descender(&self) -> f32 {
		self.font.hhea_table.descender as f32 / self.units_per_em as f32
	}

	pub fn max_box(&self) -> &Aabb {
		&self.max_box
	}

    pub fn verties(&self) -> [f32; 16] {
        let upem = self.units_per_em as f32;
        let mut extents = self.max_box;
        extents.scale(1.0 / upem, 1.0 / upem);

        [
            extents.mins.x,
            extents.mins.y,
            0.0,
            0.0,
            extents.mins.x,
            extents.maxs.y,
            0.0,
            1.0,
            extents.maxs.x,
            extents.mins.y,
            1.0,
            0.0,
            extents.maxs.x,
            extents.maxs.y,
            1.0,
            1.0,
        ]
    }

    fn get_max_box(head_table: &HeadTable) -> Aabb {
        let mut extents = Aabb::new(
            Point::new(head_table.x_min as f32, head_table.y_min as f32),
            Point::new(head_table.x_max as f32, head_table.y_max as f32),
        );

        // let per_em = TOLERANCE;

        let upem = head_table.units_per_em as f32;
        // let tolerance = upem * per_em; /* in font design units */
        let faraway = upem / (MIN_FONT_SIZE * 2.0f32.sqrt());
        let embolden_max = upem * EMBOLDEN_MAX;

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
        extents
    }

	pub fn to_outline(&mut self, ch: char) -> GlyphVisitor {
		let mut sink = GlyphVisitor::new(1.0);
        sink.accumulate.tolerance = self.units_per_em as f32 * TOLERANCE;
		
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
		// let r = self.font.horizontal_advance(glyph_index);
		// let r1 = self.font.vertical_advance(glyph_index);
		// println!("horizontal_advance, char: {}: horizontal_advance:{:?}, vertical_advance: {:?}", ch, r, r1);
		let _ = self.glyf.visit(glyph_index, &mut sink);
		sink
    }

    pub fn get_char_arc(&self, mut sink: GlyphVisitor) -> (BlobArc, HashMap<String, u64>) {
        // log::error!("get_char_arc: {:?}", char);

        let endpoints = &mut sink.accumulate.result;
        if endpoints.len() > 0 {
            // 用奇偶规则，计算 每个圆弧的 环绕数
            glyphy_outline_winding_from_even_odd(endpoints, false);
        }

        let extents = self.max_box;
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

        // println!("result_arcs: {:?}", result_arcs.len());

        // let width_cells = (extents.width() / min_width).floor();
        // let height_cells = (extents.height() / min_height).floor();
        // 根据最小格子大小计算每个格子的圆弧数据
        let (unit_arcs, map) = encode_uint_arc_data(result_arcs, &extents, min_width, min_height);
        // println!("unit_arcs[14][5]: {:?}", unit_arcs[14][5]);

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

        // extents.scale(1.0 / upem, 1.0 / upem);

        // gi.nominal_w = width_cells;
        // gi.nominal_h = height_cells;

        // gi.extents.set(&extents);

        (blob_arc, map)
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

        for char in text {
            println!("char: {}", char);
			let outline = self.to_outline(char);
            let (mut blod_arc, map) = self.get_char_arc(outline);
            let size = blod_arc.encode_data_tex(&map, data_tex, width0, offset_x0, offset_y0)?;
            // println!("data_map: {}", map.len());
            let mut info =
                blod_arc.encode_index_tex(index_tex, width1, offset_x1, offset_y1, map, size)?;

            info.index_offset = last_offset1;
            info.data_offset = (*offset_x0, *offset_y0);

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

}
