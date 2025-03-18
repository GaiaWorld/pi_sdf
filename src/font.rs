use crate::{
    glyphy::geometry::segment::{PPoint, PSegment},
    utils::{compute_cell_range, CellInfo},
};
use allsorts::{
    binary::read::ReadScope, font::MatchingPresentation, font_data::{DynamicFontTableProvider, FontData}, gsub::{FeatureMask, Features}, outline::OutlineBuilder, tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider, HeadTable}, tag, Font
};
use pi_share::Share;
use unicode_segmentation::UnicodeSegmentation;

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

fn is_arabic_char(c: char) -> bool {
    match c as u32 {
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => true,
        _ => false,
    }
}

/// FontFace 结构体，表示一个字体的面，包含字体数据和相关信息。
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct FontFace {
    /// 字体的二进制数据，使用 Share 进行内存共享。
    pub(crate) _data: Share<Vec<u8>>,
    /// Font 实例，使用 DynamicFontTableProvider 提供表数据。
    pub(crate) font: Font<DynamicFontTableProvider<'static>>,
    /// GLYF 表，存储字体轮廓信息。
    pub(crate) glyf: GlyfTable<'static>,
    _glyf_data: Vec<u8>,
    _loca_data: Vec<u8>,
    pub(crate) _loca: LocaTable<'static>,
    /// 字体的最大包围盒，用于布局和绘制。
    pub(crate) max_box: Aabb,
    pub(crate) max_box_normaliz: Aabb,
    pub(crate) units_per_em: u16,
}

impl FontFace {
    /// 创建一个新的 FontFace 实例，使用提供的字体二进制数据进行初始化。
    ///
    /// # 参数
    /// * `_data`: 字体的二进制数据，使用 Share 进行内存共享。
    ///
    /// # 返回值
    /// * `Self`: 新的 FontFace 实例。
    pub fn new_inner(_data: Share<Vec<u8>>) -> Self {
        // 初始化日志模块，设置日志级别为 Info。
        log::error!("=============1");
        let _ = console_log::init_with_level(log::Level::Info);
        let d: &'static Vec<u8> = unsafe { std::mem::transmute(_data.as_ref()) };
        let scope = ReadScope::new(d);
        let font_file = scope.read::<FontData<'static>>().unwrap();
        let provider = font_file.table_provider(0).unwrap();
        let font: Font<DynamicFontTableProvider<'static>> = Font::new(provider).unwrap().unwrap();

        let head_table = font
            .head_table()
            .unwrap()
            .ok_or("missing head table")
            .unwrap();

        let max_box_normaliz = Self::get_max_box_normaliz(&head_table);
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

        log::error!("=========== 10!! _glyf_data: {}, : _loca_data: {}", _glyf_data.len(), _loca_data.len());
        // todo!()
        log::debug!("units_per_em: {}", head_table.units_per_em);
        Self {
            _data: pi_share::Share::new(vec![]),
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

    /// 获取 Font 实例。
    ///
    /// # 返回值
    /// * `&Font<DynamicFontTableProvider>`: Font 实例的引用。
    pub fn font(&self) -> &Font<DynamicFontTableProvider> {
        &self.font
    }

    /// 计算顶点数据，返回一个用于图形渲染的顶点数组。
    ///
    /// # 参数
    /// * `_font_size`: 字体大小，用于缩放字体轮廓。
    /// * `_shadow_offsett`: 阴影偏移数据，用于绘制阴影效果。
    ///
    /// # 返回值
    /// * `[f32; 16]`: 顶点数组，用于传递给图形API进行渲染。
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

    /// 计算标准化的最大包围盒。
    /// 根据Head表中的数据，计算字体的最大包围盒并规范化。
    ///
    /// # 参数
    /// * `head_table: &HeadTable` - Head表的引用，包含字体的基本信息。
    ///
    /// # 返回值
    /// * `Aabb` -标准化后的最大包围盒。
    pub fn get_max_box_normaliz(head_table: &HeadTable) -> Aabb {
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

    /// 计算近段弧的信息。
    /// 根据给定的包围盒和比例因子，计算出与视网格最近的弧的信息，用于后续的绘制。
    ///
    /// # 参数
    /// * `extents: Aabb` - 当前细胞的活动范围。
    /// * `scale: f32` - 缩放比例。
    /// * `endpoints: &Vec<ArcEndpoint>` - 圆弧端点的集合。
    ///
    /// # 返回值
    /// * `CellInfo` - 包含包围盒、近段弧、最小宽度和高度等信息。
    pub fn compute_near_arcs<'a>(
        extents: Aabb,
        scale: f32,
        endpoints: &Vec<ArcEndpoint>,
    ) -> CellInfo {
        let extents = compute_cell_range(extents, scale);
        log::debug!("extents: {:?}", extents);

        if endpoints.len() > 0 {
            // 用奇偶规则，计算每个圆弧的环绕数。
            glyphy_outline_winding_from_even_odd(endpoints, false);
        }

        let mut min_width = f32::INFINITY;
        let mut min_height = f32::INFINITY;

        let mut p0 = Point::new(0., 0.);

        // let startid = ID.load(std::sync::atomic::Ordering::SeqCst);
        // 将圆弧控制点变成圆弧。
        let mut near_arcs = Vec::with_capacity(endpoints.len());
        let mut arcs = Vec::with_capacity(endpoints.len());
        let mut id = 0;
        for i in 0..endpoints.len() {
            let endpoint = &endpoints[i];
            if endpoint.d == GLYPHY_INFINITY {
                p0 = Point::new(endpoint.p[0], endpoint.p[1]);
                continue;
            }
            let mut arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
            arc.id = id;
            id += 1;
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);

            near_arcs.push(arc);
            arcs.push(unsafe { std::mem::transmute(near_arcs.last().unwrap()) });
        }

        let mut tempsegment = PSegment::new(PPoint::new(0., 0.), PPoint::new(0., 0.));
        let mut result_arcs = vec![];
        let mut temp = Vec::with_capacity(arcs.len());
        let mut tempidx = vec![];
        let (ab1, ab2) = extents.half(Direction::Col);
        // 二分法递归细分格子，直到格子周围的圆弧数量少于一定数目或达到停止条件。
        recursion_near_arcs_of_cell(
            // &near_arcs,
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
            // startid,
            &mut tempidx
        );
        recursion_near_arcs_of_cell(
            // &near_arcs,
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
            // startid,
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
        self.horizontal_advance_of_glyph_index(glyph_index)
    }

    /// 水平宽度
    pub fn horizontal_advance_of_glyph_index(&mut self, glyph_index: u16) -> f32 {
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

    /// 获取字体的上升高度（ascender）。
    ///
    /// # 返回值
    /// 上升高度（f32）
    pub fn ascender(&self) -> f32 {
        self.font.hhea_table.ascender as f32 / self.units_per_em as f32
    }

    /// 获取字体的单位每英尺数（units_per_em）。
    ///
    /// # 返回值
    /// 单位每英尺数（u16）
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// 获取字体的下降高度（descender）。
    ///
    /// # 返回值
    /// 下降高度（f32）
    pub fn descender(&self) -> f32 {
        self.font.hhea_table.descender as f32 / self.units_per_em as f32
    }

    /// 获取字形的最大边界框（max_box）。
    ///
    /// # 返回值
    /// 最大边界框的坐标（Vec<f32>）
    pub fn max_box(&self) -> Vec<f32> {
        vec![
            self.max_box.mins.x,
            self.max_box.mins.y,
            self.max_box.maxs.x,
            self.max_box.maxs.y,
        ]
    }

    /// 获取归一化的字形最大边界框（max_box_normaliz）。
    ///
    /// # 返回值
    /// 归一化后的最大边界框的坐标（Vec<f32>）_vals
    pub fn max_box_normaliz(&self) -> Vec<f32> {
        vec![
            self.max_box_normaliz.mins.x,
            self.max_box_normaliz.mins.y,
            self.max_box_normaliz.maxs.x,
            self.max_box_normaliz.maxs.y,
        ]
    }

    /// 获取字符的字形索引。
    ///
    /// # 参数
    /// - `ch`: 查询的字符
    /// # 返回值
    /// 字形索引（u16）
    pub fn glyph_index(&mut self, ch: char) -> u16 {
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        glyph_index
    }

    pub fn glyph_indexs(&mut self, text: &str, script: u32) -> Vec<u16> {
        let g = text.split_word_bounds().collect::<Vec<&str>>();
        let mut glyphs = Vec::new();
        let mut str = "".to_string();
        for s in g {
            let char  = s.chars().next().unwrap();
            println!("===========char: {}, unicode: {}, is_arabic_char: {}", s, char as u32, is_arabic_char(char));
            if !is_arabic_char(char){
                str.push_str(s);
            } else {
                if !str.is_empty() {
                    // println!("========== 普通字符：{}, 颠倒：{}", str, str.chars().rev().collect::<String>());
                    // let r = str.chars().rev().collect::<String>();
                    let t = if script != 0{
                        str.chars().rev().collect::<String>()
                    }else{
                        str.chars().collect::<String>()
                    };

                    glyphs.append(&mut self.glyph_indexs_impl(&t));
                    str.clear();
                }
                glyphs.append(&mut self.glyph_indexs_impl(s));
            }
        }

        if !str.is_empty() {
            // println!("========== 普通字符：{}, 颠倒：{}", str, str.chars().rev().collect::<String>());
            // let r = str.chars().rev().collect::<String>();
            let t = if script != 0{
                str.chars().rev().collect::<String>()
            }else{
                str.chars().collect::<String>()
            };

            glyphs.append(&mut self.glyph_indexs_impl(&t));
            str.clear();
        }
        
        glyphs
    }

    fn glyph_indexs_impl(&mut self, text: &str)-> Vec<u16> {
        let script = tag::ARAB;
        let lang = tag!(b"URD ");
        let glyphs = self.font.map_glyphs(text, script, MatchingPresentation::NotRequired);
        let glyphs = self.font
            .shape(
                glyphs,
                script,
                Some(lang),
                &Features::Mask(FeatureMask::default()),
                true,
            )
            .expect("error shaping text");
        glyphs.iter().map(|item| item.glyph.glyph_index).collect::<Vec<u16>>()
    }

    /// 获取字体数据的大小。
    ///
    /// # 返回值
    /// 字体数据的大小（usize）
    pub fn debug_size(&self) -> usize {
        // self._data.len()
        0
    }

    /// 将字符转换为轮廓信息。
    ///
    /// # 参数
    /// - `ch`: 查询的字符
    /// # 返回值
    /// 轮廓信息（OutlineInfo）
    pub fn to_outline(&mut self, ch: char) -> OutlineInfo {
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        let mut o = self.to_outline_of_glyph_index(glyph_index);
        o.char = ch;
        o
    }

    /// 将字符转换为轮廓信息。
    ///
    /// # 参数
    /// - `ch`: 查询的字符
    /// # 返回值
    /// 轮廓信息（OutlineInfo）
    pub fn to_outline_of_glyph_index(&mut self, glyph_index: u16) -> OutlineInfo {
        let mut sink = GlyphVisitor::new(SCALE / self.units_per_em as f32);
        sink.accumulate.tolerance = self.units_per_em as f32 * TOLERANCE;
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
            // #[cfg(feature = "debug")]
            svg_paths,
            ..
        } = sink;

        OutlineInfo {
            endpoints: result,
            bbox: vec![bbox2.mins.x, bbox2.mins.y, bbox2.maxs.x, bbox2.maxs.y],
            advance,
            units_per_em: self.units_per_em,
            char: ' ',
            extents: vec![bbox.mins.x, bbox.mins.y, bbox.maxs.x, bbox.maxs.y],
            // #[cfg(feature = "debug")]
            svg_paths,
        }
    }

    /// 将字符转换为轮廓信息（WebAssembly专用）。
    ///
    /// # 参数
    /// - `ch`: 查询的字符
    /// # 返回值
    /// 轮廓信息（WasmOutlineInfo）
    pub fn to_outline_of_wasm(&mut self, ch: char) -> WasmOutlineInfo {
        let outline = self.to_outline(ch);
        let buf = bitcode::serialize(&outline).unwrap();
        WasmOutlineInfo {
            buf,
            units_per_em: outline.units_per_em,
            advance: outline.advance,
            bbox: outline.bbox,
            extents: outline.extents,
        }
    }

    /// 将字符转换为轮廓信息（WebAssembly专用）。
    ///
    /// # 参数
    /// - `ch`: 查询的字符
    /// # 返回值
    /// 轮廓信息（WasmOutlineInfo）
    pub fn to_outline_of_wasm_glyph_index(&mut self, glyph_index: u16) -> WasmOutlineInfo {
        let outline = self.to_outline_of_glyph_index(glyph_index);
        let buf = bitcode::serialize(&outline).unwrap();
        WasmOutlineInfo {
            buf,
            units_per_em: outline.units_per_em,
            advance: outline.advance,
            bbox: outline.bbox,
            extents: outline.extents,
        }
    }
}

/// WebAssembly环境下的轮廓信息结构体。
///
/// # 属性
/// - `buf`: 序列化后的轮廓数据
/// - `units_per_em`: 字体的单位每英尺数
/// - `advance`: 字符的水平进度
/// - `bbox`: 字形的边界框
/// - `extents`: 字形的扩展信息
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct WasmOutlineInfo {
    pub buf: Vec<u8>,
    pub units_per_em: u16,
    pub advance: u16,
    pub bbox: Vec<f32>,
    pub extents: Vec<f32>,
}
