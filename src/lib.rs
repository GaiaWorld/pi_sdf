#![feature(trait_alias)]

// #[macro_use]
// extern crate lazy_static;

use std::io::Read;

pub mod font;
pub mod glyphy;
pub mod render_path;
pub mod shape;
pub mod svg;
pub mod utils;
pub mod blur;

pub type Point = parry2d::math::Point<f32>;
pub type Matrix4 = parry2d::na::Matrix4<f32>;
pub type Vector3 = parry2d::na::Vector3<f32>;
pub type Vector2 = parry2d::na::Vector2<f32>;
pub type Orthographic3 = parry2d::na::Orthographic3<f32>;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn brotli_decompressor(data: &[u8]) -> Vec<u8> {
    let mut reader = brotli_decompressor::Decompressor::new(data, data.len());
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    buf
}
