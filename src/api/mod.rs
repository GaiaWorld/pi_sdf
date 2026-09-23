//! 门面层：对外的类型与入口（native 与 wasm 共用）

pub mod font;
pub mod path;
pub mod entry;
mod entry_inner;
pub mod codec;
mod codec_inner;

pub use entry::{brotli_decompressor, compute_cell_grid, compute_layout, compute_near_arcs, compute_sdf_tex, compute_svg_debug, get_char_arc_debug, rasterize};
pub use font::{FontFace, GlyphMetrics};
pub use path::SvgScene;
