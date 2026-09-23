//! 数据层：数据结构与它们自身的内蕴运算（native 与 wasm 共用同一份定义）
//!
//! 对外入口：`pi_sdf2::model::{geom, outline, grid, raster, path, base}`

pub mod base;
pub mod geom;
pub mod outline;
pub mod grid;
pub mod raster;
pub mod path;

pub use base::Error;
pub use geom::{Aabb, Arc, ArcEndpoint, Bezier, Direction, Line, Point, Segment, SignedVector, Vector};
pub use outline::{Contour, Outline};
pub use grid::{Cell, CellGrid};
pub use raster::{DataTexture, IndexTexture, RasterOptions, SdfTexture, TextureInfo, TextureLayout, UnitArc};
pub use path::{Path, PathVerb, Shape};
