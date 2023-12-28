#[macro_use]
extern crate lazy_static;

pub mod glyphy;
pub mod utils;
pub mod svg;
pub mod render_path;
pub mod font;

pub type Point = parry2d::math::Point<f32>;
pub type Matrix4 = parry2d::na::Matrix4<f32>;
pub type Vector3 = parry2d::na::Vector3<f32>;
pub type Orthographic3 = parry2d::na::Orthographic3<f32>;
