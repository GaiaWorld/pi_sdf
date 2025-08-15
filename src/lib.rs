#![feature(trait_alias)]

#[macro_use]
extern crate lazy_static;

use std::io::Read;

pub mod blur;
pub mod font;
pub mod glyphy;
pub mod shape;
pub mod svg;
pub mod utils;
mod system_font;

pub type Point = parry2d::math::Point<f32>;
pub type Matrix4 = parry2d::na::Matrix4<f32>;
pub type Vector3 = parry2d::na::Vector3<f32>;
pub type Vector2 = parry2d::na::Vector2<f32>;
pub type Orthographic3 = parry2d::na::Orthographic3<f32>;
// #[cfg(target_arch = "wasm32")]
// use lol_alloc::{FreeListAllocator, LockedAllocator};
// #[cfg(target_arch = "wasm32")]
// #[global_allocator]
// static ALLOCATOR: LockedAllocator<FreeListAllocator> = LockedAllocator::new(FreeListAllocator::new(/* 64*1024*1024 */));

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: talc::Talck<talc::locking::AssumeUnlockable, talc::ClaimOnOom> = unsafe {
    static mut MEMORY: [u8; 128 * 1024 * 1024] = [0; 128 * 1024 * 1024];
    let span = talc::Span::from_const_array(std::ptr::addr_of!(MEMORY));
    talc::Talc::new(talc::ClaimOnOom::new(span)).lock()
};
// use font::FontFace;
use glyphy::geometry::{aabb::Aabb, arc::Arc};
// use shape::SvgInfo;
// use unicode_segmentation::UnicodeSegmentation;
// use pi_share::Share;
// use serde_json::value::Index;
use utils::{CellInfo, OutlineInfo, SdfInfo2};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn brotli_decompressor(data: &[u8]) -> Vec<u8> {
    let mut reader = brotli_decompressor::Decompressor::new(data, data.len());
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    buf
}

// // #[cfg(target_arch = "wasm32")]
// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
// pub fn get_outline(char: String, tex_size: usize, pxrange: u32) -> OutlineInfo {
//     let _ = console_log::init_with_level(log::Level::Debug);
//     let buffer = include_bytes!("../source/msyh.ttf").to_vec();
//     let mut ft_face = FontFace::new(buffer);
//     let char = char.chars().next().unwrap();
//     ft_face.to_outline(char)
// }

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct DebugAabb {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct DebugCellInfo {
    pub extents: DebugAabb,
    pub arcs: Vec<Arc>,
    infos: Vec<Vec<f32>>,
    pub min_width: f32,
    pub min_height: f32,
    pub is_area: bool,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl DebugCellInfo {
    pub fn get_info(&self, index: usize) -> Option<Vec<f32>> {
        self.infos.get(index).map(|v| v.to_vec())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_near_arcs(outline_info: &OutlineInfo, scale: f32) -> DebugCellInfo {
    let CellInfo {
        extents,
        arcs,
        info,
        min_width,
        min_height,
        is_area,
    } = outline_info.compute_near_arcs(scale);

    DebugCellInfo {
        extents: DebugAabb {
            min_x: extents.mins.x,
            min_y: extents.mins.y,
            max_x: extents.maxs.x,
            max_y: extents.maxs.y,
        },
        arcs,
        infos: info
            .iter()
            .map(|(indexs, bbox)| {
                let mut res = vec![bbox.mins.x, bbox.mins.y, bbox.maxs.x, bbox.maxs.y];
                indexs.iter().for_each(|v| res.push((*v) as f32));
                res
            })
            .collect(),
        min_width,
        min_height,
        is_area,
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_sdf_tex(
    outline_info: &OutlineInfo,
    cell_info: &DebugCellInfo,
    tex_size: usize,
    pxrange: u32,
) -> SdfInfo2 {
    let DebugCellInfo {
        extents,
        arcs,
        infos,
        min_width,
        min_height,
        is_area,
    } = cell_info;
    let mut res = Vec::new();
    for v in infos {
        let bbox = Aabb::new(Point::new(v[0], v[1]), Point::new(v[2], v[3]));
        let indexs = v[4..v.len()]
            .iter()
            .map(|v| *v as usize)
            .collect::<Vec<usize>>();
        res.push((indexs, bbox));
    }

    let info = CellInfo {
        extents: Aabb::new(
            Point::new(extents.min_x, extents.min_y),
            Point::new(extents.max_x, extents.max_y),
        ),
        arcs: arcs.clone(),
        info: res,
        min_width: *min_width,
        min_height: *min_height,
        is_area: *is_area,
    };

    outline_info.compute_sdf_tex(info, tex_size, pxrange, false, pxrange)
}

static mut AAR: Vec<Vec<u8>> = Vec::new();
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn test(i: i32) {
    log::error!("================ i: {}", i);
    // let data = include_bytes!("../source/kt.woff2").to_vec();
    // let r = crate::font::FontFace::new(data);
    // let r = vec![0; 1024];
    // let mut r2 = Vec::new();
    // for j in 0..i {
    //     r2.push((j % 255) as u8);
    // }
    // unsafe { AAR.push(r2) };
    // let text = "، فهو يتحدّث بلغة يونيكود.يد (Unicode Conference)، الذي سيعقد في 10-12 آذار 1997 بمدينة مَايِنْتْس، ألمانيا. حيث ستتم، تصميم النصوص والحوسبة متعددة اللغات\r\n".to_string();
    // let g = text.split_word_bounds().collect::<Vec<&str>>();
    // log::error!("========== text: {:?}", g);
}
