use std::sync::Arc;

use allsorts::{binary::read::ReadScope, font::MatchingPresentation, font_data::FontData, gsub::{FeatureMask, Features}, tag, Font};
use image::ColorType;

// use nalgebra::Vector3;
use pi_sdf::font::FontFace;


fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let buffer = std::fs::read("./source/wdyk-Reg.ttf").unwrap();
    let mut ft_face = { FontFace::new(Arc::new(buffer)) };
    ft_face.to_outline('一');
    // let g = ft_face.glyph_index(' ');
    // println!("=============g: {}", g);
}
