use crate::{glyphy::geometry::{arc::ID, segment::{PPoint, PSegment}}, utils::{compute_cell_range, CellInfo}};
use allsorts::{
    binary::read::ReadScope,
    font::MatchingPresentation,
    font_data::{DynamicFontTableProvider, FontData},
    outline::OutlineBuilder,
    tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider, HeadTable},
    tag, Font,
};
use pi_share::Share;

use crate::utils::SdfInfo2;
use crate::{
    glyphy::{
        blob::recursion_near_arcs_of_cell,
        geometry::{
            aabb::{Aabb, Direction},
            arc::{Arc, ArcEndpoint},
            arcs::GlyphyArcAccumulator,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::GLYPHY_INFINITY,
    },
    utils::{GlyphVisitor, OutlineInfo, SCALE, TOLERANCE},
    Point,
};
use std::char;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

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
        log::debug!("units_per_em: {}", head_table.units_per_em);
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

    pub fn font(&self) -> &Font<DynamicFontTableProvider> {
        &self.font
    }

    pub fn verties(&self, _font_size: f32, _shadow_offsett: &mut [f32]) -> [f32; 16] {
        let extents = self.max_box_normaliz.clone();

        let min_uv = [0.0f32, 0.0];
        let max_uv = [1.0f32, 1.0];

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

    // pub fn encode_uint_arc(
    //     extents: Aabb,
    //     mut endpoints: Vec<ArcEndpoint>,
    // ) -> (BlobArc, HashMap<u64, u64>) {
    //     // log::debug!("result_arcs: {:?}", result_arcs.len());

    //     // let width_cells = (extents.width() / min_width).floor();
    //     // let height_cells = (extents.height() / min_height).floor();
    //     // 根据最小格子大小计算每个格子的圆弧数据
    //     let CellInfo{info, min_width, min_height, ..} =
    //         Self::compute_near_arcs(extents, 0.0, &mut endpoints);
    //     // log::trace!("near_arcs: {}", near_arcs.len());
    //     let (unit_arcs, map) =
    //         encode_uint_arc_data(info, &extents, min_width, min_height, None);
    //     // log::debug!("unit_arcs[14][5]: {:?}", unit_arcs[14][5]);

    //     let [min_sdf, max_sdf] = travel_data(&unit_arcs);
    //     let blob_arc = BlobArc {
    //         min_sdf,
    //         max_sdf,
    //         cell_size: min_width,
    //         #[cfg(feature = "debug")]
    //         show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
    //         extents,
    //         data: unit_arcs,
    //         avg_fetch_achieved: 0.0,
    //         endpoints,
    //     };

    //     // extents.scale(1.0 / upem, 1.0 / upem);

    //     // gi.nominal_w = width_cells;
    //     // gi.nominal_h = height_cells;

    //     // gi.extents.set(&extents);

    //     (blob_arc, map)
    // }

    pub fn compute_near_arcs<'a>(
        extents: Aabb,
        scale: f32,
        endpoints: &Vec<ArcEndpoint>,
    ) -> CellInfo {
        let extents = compute_cell_range(extents, scale);
        log::debug!("extents: {:?}", extents);

        if endpoints.len() > 0 {
            // 用奇偶规则，计算 每个圆弧的 环绕数
            glyphy_outline_winding_from_even_odd(endpoints, false);
        }

        let mut min_width = f32::INFINITY;
        let mut min_height = f32::INFINITY;

        let mut p0 = Point::new(0., 0.);

        let startid = ID.load(std::sync::atomic::Ordering::SeqCst);
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

        // let mut tempsegment = parry2d::shape::Segment::new(Point::new(0., 0.), Point::new(0., 0.));
        let mut tempsegment = PSegment::new(PPoint::new(0., 0.), PPoint::new(0., 0.));
        let mut result_arcs = vec![];
        let mut temp = Vec::with_capacity(arcs.len());
        let mut tempidx = vec![];
        let (ab1, ab2) = extents.half(Direction::Col);
        // 二分法递归细分格子，知道格子周围的圆弧数量小于二或者小于32/1停止
        recursion_near_arcs_of_cell(
            &near_arcs,
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
            &mut tempsegment,
            startid,
            &mut tempidx
        );
        recursion_near_arcs_of_cell(
            &near_arcs,
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
            &mut tempsegment,
            startid,
            &mut tempidx
        );
        CellInfo {
            extents,
            arcs: near_arcs,
            info: result_arcs,
            min_width,
            min_height,
            is_area: true,
        }
    }
}

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

    pub fn max_box(&self) -> Vec<f32> {
        vec![
            self.max_box.mins.x,
            self.max_box.mins.y,
            self.max_box.maxs.x,
            self.max_box.maxs.y,
        ]
    }

    pub fn max_box_normaliz(&self) -> Vec<f32> {
        vec![
            self.max_box_normaliz.mins.x,
            self.max_box_normaliz.mins.y,
            self.max_box_normaliz.maxs.x,
            self.max_box_normaliz.maxs.y,
        ]
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

    pub fn to_outline(&mut self, ch: char) -> OutlineInfo {
        let mut sink = GlyphVisitor::new(SCALE / self.units_per_em as f32);
        sink.accumulate.tolerance = self.units_per_em as f32 * TOLERANCE;

        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        assert_ne!(glyph_index, 0);
        let advance = self.font.horizontal_advance(glyph_index).unwrap();

        let _ = self.glyf.visit(glyph_index, &mut sink);

        let mut bbox2 = Aabb::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
        if let Ok(r) = self.glyf.get_parsed_glyph(glyph_index) {
            if let Some(g) = r {
                // log::debug!("g.bounding_box:{:?}", g.bounding_box);
                bbox2.mins.x = g.bounding_box.x_min as f32;
                bbox2.mins.y = g.bounding_box.y_min as f32;
                bbox2.maxs.x = g.bounding_box.x_max as f32;
                bbox2.maxs.y = g.bounding_box.y_max as f32;
            }
        }

        let GlyphVisitor {
            accumulate: GlyphyArcAccumulator { result, .. },
            bbox,
            ..
        } = sink;

        OutlineInfo {
            endpoints: result,
            bbox: vec![bbox2.mins.x, bbox2.mins.y, bbox2.maxs.x, bbox2.maxs.y],
            advance,
            units_per_em: self.units_per_em,
            char: ch,
            extents: vec![bbox.mins.x, bbox.mins.y, bbox.maxs.x, bbox.maxs.y],
        }
    }

    pub fn to_outline_of_wasm(&mut self, ch: char) -> WasmOutlineInfo {
        let outline = self.to_outline(ch);
        let buf = bitcode::serialize(&outline).unwrap();
        WasmOutlineInfo {
            buf,
            units_per_em: outline.units_per_em,
            advance: outline.advance,
            bbox: outline.bbox,
            extents: outline.extents
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct WasmOutlineInfo {
    pub buf: Vec<u8>,
    pub units_per_em: u16,
    pub advance: u16,
    pub bbox: Vec<f32>,
    pub extents: Vec<f32>
}
