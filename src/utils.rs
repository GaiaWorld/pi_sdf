use std::char;

use ab_glyph_rasterizer::{point, Rasterizer};
use allsorts::{
    binary::read::ReadScope,
    font::MatchingPresentation,
    font_data::{DynamicFontTableProvider, FontData},
    outline::{OutlineBuilder, OutlineSink},
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
    tables::{glyf::GlyfTable, loca::LocaTable, FontTableProvider},
    tag, Font,
};
// use freetype_sys::FT_Vector;
use image::{ImageBuffer, Rgba};
use wasm_bindgen::prelude::wasm_bindgen;
// use parry2d::math::Point;

use crate::{glyphy::geometry::arcs::GlyphyArcAccumulator, Point};

pub struct User {
    pub accumulate: GlyphyArcAccumulator,
    pub path_str: String,
    pub svg_paths: Vec<String>,
    pub svg_endpoints: Vec<[f32; 2]>,
}

// pub extern "C" fn move_to(to: *const FT_Vector, user: *mut c_void) -> i32 {
//     let to = unsafe { &*to };
//     let user = unsafe { &mut *(user as *mut User) };

//     if !user.accumulate.result.is_empty() {
//         println!("+ Z");
//         user.accumulate.close_path();
//         user.path_str.push_str("Z");
//         user.svg_paths.push(user.path_str.clone());
//         user.path_str.clear();
//     }
//     println!("M {} {} ", to.x, to.y);

//     user.accumulate
//         .move_to(Point::new(to.x as f32, to.y as f32));
//     user.path_str.push_str(&format!("M {} {}", to.x, to.y));
//     user.svg_endpoints.push([to.x as f32, to.y as f32]);

//     return 0;
// }

// pub extern "C" fn line_to(to: *const FT_Vector, user: *mut c_void) -> i32 {
//     let to = unsafe { &*to };
//     println!("+ L {} {} ", to.x, to.y);

//     let user = unsafe { &mut *(user as *mut User) };
//     user.accumulate
//         .line_to(Point::new(to.x as f32, to.y as f32));
//     user.path_str.push_str(&format!("L {} {}", to.x, to.y));
//     user.svg_endpoints.push([to.x as f32, to.y as f32]);

//     return 0;
// }

// pub extern "C" fn conic_to(
//     control: *const FT_Vector,
//     to: *const FT_Vector,
//     user: *mut c_void,
// ) -> i32 {
//     let control = unsafe { &*control };
//     let to = unsafe { &*to };
//     println!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);

//     let user = unsafe { &mut *(user as *mut User) };
//     user.accumulate.conic_to(
//         Point::new(control.x as f32, control.y as f32),
//         Point::new(to.x as f32, to.y as f32),
//     );
//     user.svg_endpoints.push([to.x as f32, to.y as f32]);
//     return 0;
// }

// pub extern "C" fn cubic_to(
//     control1: *const FT_Vector,
//     control2: *const FT_Vector,
//     to: *const FT_Vector,
//     user: *mut c_void,
// ) -> i32 {
//     let control1 = unsafe { &*control1 };
//     let control2 = unsafe { &*control2 };
//     let to = unsafe { &*to };
//     println!(
//         "+ C {} {} {} {} {} {} ",
//         control1.x, control1.y, control2.x, control2.y, to.x, to.y
//     );

//     let _user = unsafe { &mut *(user as *mut User) };

//     return 0;
// }

#[wasm_bindgen]
pub struct FontFace {
    pub(crate) _data: Vec<u8>,
    pub(crate) font: Font<DynamicFontTableProvider<'static>>,
    pub(crate) glyf: GlyfTable<'static>,
    _glyf_data: Vec<u8>,
    _loca_data: Vec<u8>,
    pub(crate) _loca: LocaTable<'static>,
}

impl FontFace {
    pub fn new(_data: Vec<u8>) -> Self {
        let d: &'static Vec<u8> = unsafe { std::mem::transmute(&_data) };
        let scope = ReadScope::new(d);
        let font_file = scope.read::<FontData<'static>>().unwrap();
        // font_file.table_provider(index)

        let provider = font_file.table_provider(0).unwrap();
        let font: Font<DynamicFontTableProvider<'static>> = Font::new(provider).unwrap().unwrap();

        let _loca_data = font
            .font_table_provider
            .read_table_data(tag::LOCA)
            .unwrap()
            .to_vec();
        let l: &'static Vec<u8> = unsafe { std::mem::transmute(&_loca_data) };

        let loca = ReadScope::new(&l)
            .read_dep::<LocaTable<'_>>((
                usize::from(font.maxp_table.num_glyphs),
                font.head_table()
                    .unwrap()
                    .ok_or("missing head table")
                    .unwrap()
                    .index_to_loc_format,
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
        }
    }

    pub fn to_outline(&mut self, ch: char, sink: &mut impl OutlineSink) {
        let (glyph_index, _) =
            self.font
                .lookup_glyph_index(ch, MatchingPresentation::NotRequired, None);
        let _ = self.glyf.visit(glyph_index, sink);
    }
}

#[wasm_bindgen]
pub struct GlyphVisitor {
    rasterizer: Rasterizer,
    pub(crate) accumulate: GlyphyArcAccumulator,
    pub(crate) path_str: String,
    pub(crate) svg_paths: Vec<String>,
    pub(crate) svg_endpoints: Vec<[f32; 2]>,

    scale: f32,
    start: Point,
    previous: Point,
}

#[wasm_bindgen]
impl GlyphVisitor {
    pub fn new(scale: f32) -> Self {
        let accumulate = GlyphyArcAccumulator::new();
        let rasterizer = ab_glyph_rasterizer::Rasterizer::new(512, 512);
        Self {
            rasterizer,
            accumulate,
            path_str: "".to_string(),
            svg_paths: vec![],
            svg_endpoints: vec![],
            scale,
            start: Point::default(),
            previous: Point::default(),
        }
    }
}

impl GlyphVisitor {
    pub fn get_pixmap(&mut self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut img = ImageBuffer::from_fn(512, 512, |_, _| Rgba([255u8, 0, 0, 0]));

        self.rasterizer.for_each_pixel_2d(|x, y, a| {
            let rgba = img.get_pixel_mut(x, 512 - y - 1);
            rgba[3] = (a * 255.0) as u8;
        });

        return img;
    }
}

impl OutlineSink for GlyphVisitor {
    fn move_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::info!("M {} {} ", to.x, to.y);

        if self.scale > 0.02 {
            self.accumulate
                .move_to(Point::new(to.x as f32, to.y as f32));
            self.path_str.push_str(&format!("M {} {}", to.x, to.y));
            self.svg_endpoints.push([to.x as f32, to.y as f32]);
        }

        self.start = to;
        self.previous = to;
    }

    fn line_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y());
        log::info!("+ L {} {} ", to.x, to.y);
        if self.scale > 0.02 {
            self.accumulate.line_to(to);
            self.path_str.push_str(&format!("L {} {}", to.x, to.y));
            self.svg_endpoints.push([to.x as f32, to.y as f32]);
        } else {
            self.rasterizer.draw_line(
                point(
                    self.previous.x * self.scale,
                    (self.previous.y + 500.) * self.scale,
                ),
                point(to.x, to.y),
            );
        }

        self.previous = to;
    }

    fn quadratic_curve_to(&mut self, control: Vector2F, to: Vector2F) {
        let control = Point::new(control.x(), control.y());
        let to = Point::new(to.x(), to.y());

        log::info!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
        if self.scale > 0.02 {
            self.accumulate.conic_to(control, to);
            self.svg_endpoints.push([to.x, to.y]);
        } else {
            self.rasterizer.draw_quad(
                point(
                    self.previous.x * self.scale,
                    (self.previous.y + 500.) * self.scale,
                ),
                point(control.x * self.scale, (control.y + 500.) * self.scale),
                point(to.x * self.scale, (to.y + 500.) * self.scale),
            );
        }
        self.previous = to;
    }

    fn cubic_curve_to(&mut self, _control: LineSegment2F, _to: Vector2F) {
        // 字形数据没有三次贝塞尔曲线
        todo!();
        // println!(
        //     "curve_to({}, {}, {}, {}, {}, {})",
        //     control.from_x(),
        //     control.from_y(),
        //     control.to_x(),
        //     control.to_y(),
        //     to.x(),
        //     to.y()
        // );

        // let control_from = Point::new(control.from_x(), (control.from_y()));
        // let control_to = Point::new(control.to_x(), (control.to_y()));
        // let to = Point::new(to.x(), (to.x()));
        // self.accumulate.cubic_to(control_from, control_to, to);
        // self.previous = to;
    }

    fn close(&mut self) {
        if self.previous != self.start {
            log::info!("+ L {} {} ", self.start.x, self.start.y);
            if self.scale > 0.02 {
                self.accumulate.line_to(self.start);
                self.path_str
                    .push_str(&format!("M {} {}", self.start.x, self.start.y));
                self.svg_endpoints
                    .push([self.start.x as f32, self.start.y as f32]);
            } else {
                self.rasterizer.draw_line(
                    point(
                        self.previous.x * self.scale,
                        (self.previous.y + 500.) * self.scale,
                    ),
                    point(
                        self.start.x * self.scale,
                        (self.start.y + 500.) * self.scale,
                    ),
                )
            }
        }
        log::info!("+ Z");
        if self.scale > 0.02 {
            self.accumulate.close_path();
            self.path_str.push_str("Z");
            self.svg_paths.push(self.path_str.clone());
            self.path_str.clear();
        }
        // println!("close()");
    }
}
