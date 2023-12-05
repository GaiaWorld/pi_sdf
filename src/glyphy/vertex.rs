use parry2d::{bounding_volume::Aabb, math::Point};
use wasm_bindgen::prelude::wasm_bindgen;

use super::geometry::aabb::AabbEXT;

#[wasm_bindgen]
pub struct GlyphInfo {
    pub(crate) extents: Aabb,

    // 晶格 的 宽-高
    pub(crate) nominal_w: f32,
    pub(crate) nominal_h: f32,

    // 数据 在 纹理 的 起始地址
    // 单个字符，永远为0
    atlas_x: f32,
    atlas_y: f32,
}

#[wasm_bindgen]
impl GlyphInfo {
    pub fn new() -> Self {
        Self {
            extents: Aabb::new(
                Point::new(f32::INFINITY, f32::INFINITY),
                Point::new(f32::INFINITY, f32::INFINITY),
            ),
            nominal_w: 0.0,
            nominal_h: 0.0,
            atlas_x: 0.0,
            atlas_y: 0.0,
        }
    }
}

pub struct GlyphyVertex {
    // 位置信息
    // 就是 该字符 包围盒 对应 的 位置
    pub x: f32,
    pub y: f32,

    // Glyph 信息，具体包含内容如下：
    //   + 纹理 起始位置
    //   + corner_x / corner_y: 0 代表 左 / 上，1代表 右 / 下
    //   + 格子个数（宽，高）
    pub g16hi: i32,
    pub g16lo: i32,
}

/**
 * 顶点数据，每字符 一个 四边形，2个三角形，6个顶点
 *
 * 拐点: 0, 0, 1, 1
 *
 * 通过 glyph_vertex_encode 函数 编码，数据如下：
 *    - 位置信息: x, y
 *	  - corner_x/corner_y: 0 代表 左/上，1代表 右/下
 *	  - 纹理信息: 纹理 起始位置, corner_x/corner_y, 格子个数（宽，高）
 */
pub fn add_glyph_vertices(
    gi: &GlyphInfo,
    font_size: Option<f32>, // = 1.0
    extents: Option<&mut Aabb>,
) -> [GlyphyVertex; 4] {
    let font_size = if let Some(v) = font_size { v } else { 1.0 };
    let r = [
        encode_corner(0.0, 0.0, gi, font_size),
        encode_corner(0.0, 1.0, gi, font_size),
        encode_corner(1.0, 0.0, gi, font_size),
        encode_corner(1.0, 1.0, gi, font_size),
    ];

    if let Some(extents) = extents {
        extents.clear();
        for i in 0..4 {
            let p = Point::new(r[i].x, r[i].y);
            extents.add(p);
        }
    }

    return r;
}

pub fn encode_corner(cx: f32, cy: f32, gi: &GlyphInfo, font_size: f32) -> GlyphyVertex {
    let vx = font_size * ((1.0 - cx) * gi.extents.mins.x + cx * gi.extents.maxs.x);

    let vy = font_size * ((1.0 - cy) * gi.extents.mins.y + cy * gi.extents.maxs.y);

    return glyph_vertex_encode(vx, vy, cx, cy, gi);
}

/**
 * 顶点 编码
 */
pub fn glyph_vertex_encode(
    x: f32,
    y: f32,
    corner_x: f32,
    corner_y: f32, // 0 代表 左/上，1代表 右/下
    gi: &GlyphInfo,
) -> GlyphyVertex {
    let encoded = glyph_encode(
        gi.atlas_x as i32,
        gi.atlas_y as i32,
        corner_x as i32,
        corner_y as i32,
        gi.nominal_w as i32,
        gi.nominal_h as i32,
    );

    return GlyphyVertex {
        x,
        y,
        g16hi: encoded >> 16,
        g16lo: encoded & 0xFFFF,
    };
}

pub fn glyph_encode(
    atlas_x: i32, /* 7 bits */
    atlas_y: i32, /* 7 bits */

    corner_x: i32, /* 1 bit */
    corner_y: i32, /* 1 bit */

    nominal_w: i32, /* 6 bits */
    nominal_h: i32, /* 6 bits */
) -> i32 {
    assert!(0 == (atlas_x & -(0x7F + 1)));
    assert!(0 == (atlas_y & -(0x7F + 1)));

    assert!(0 == (corner_x & -2));
    assert!(0 == (corner_y & -2));

    assert!(0 == (nominal_w & -(0x3F + 1)));
    assert!(0 == (nominal_h & -(0x3F + 1)));

    // 共  16 位
    // 最高 2 位 --> 00
    //      7 位 --> 纹理偏移
    //      6 位 --> 网格宽高
    //   低 1 位 --> 是否 右下角
    let x = (((atlas_x << 6) | nominal_w) << 1) | corner_x;

    // 共 16位
    let y = (((atlas_y << 6) | nominal_h) << 1) | corner_y;

    return (x << 16) | y;
}
