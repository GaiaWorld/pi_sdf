#![feature(trait_alias)]

// #[macro_use]
// extern crate lazy_static;

use std::io::Read;

mod arc_to;
pub mod blur;
pub mod font;
pub mod glyphy;
pub mod render_path;
pub mod shape;
pub mod svg;
pub mod utils;

pub type Point = parry2d::math::Point<f32>;
pub type Matrix4 = parry2d::na::Matrix4<f32>;
pub type Vector3 = parry2d::na::Vector3<f32>;
pub type Vector2 = parry2d::na::Vector2<f32>;
pub type Orthographic3 = parry2d::na::Orthographic3<f32>;

use font::FontFace;
use pi_share::Share;
use utils::SdfInfo2;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn brotli_decompressor(data: &[u8]) -> Vec<u8> {
    let mut reader = brotli_decompressor::Decompressor::new(data, data.len());
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    buf
}

#[cfg(feature = "debug")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn compute_sdf_debug(char: String, tex_size: usize, pxrange: u32) -> SdfInfo2 {
    let _ = console_log::init_with_level(log::Level::Debug);
    let buffer = include_bytes!("../source/msyh.ttf").to_vec();
    let mut ft_face = FontFace::new(buffer);
    let char = char.chars().next().unwrap();
    let outline_info = ft_face.to_outline(char);
    let cell_info = outline_info.compute_near_arcs(2.0);
    outline_info.compute_sdf_tex(cell_info, tex_size, pxrange, false, pxrange)
}
