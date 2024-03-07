
use pi_shape::plane::aabb::Aabb;
use pi_shape::plane::Point;
use wasm_bindgen::prelude::wasm_bindgen;

use super::geometry::aabb::AabbEXT;

#[wasm_bindgen]
#[derive(Debug, Clone)]
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

impl GlyphInfo {
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
        &self,
        font_size: Option<f32>, // = 1.0
        extents: Option<&mut Aabb>,
    ) -> [GlyphyVertex; 4] {
        let font_size = if let Some(v) = font_size { v } else { 1.0 };
        let r = [
            self.encode_corner(0.0, 0.0, font_size),
            self.encode_corner(0.0, 1.0, font_size),
            self.encode_corner(1.0, 0.0, font_size),
            self.encode_corner(1.0, 1.0, font_size),
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

    pub fn add_glyph_uv(&self) -> [GlyphyUV; 4] {
        let r = [
            self.encode_corner2(0.0, 0.0),
            self.encode_corner2(0.0, 1.0),
            self.encode_corner2(1.0, 0.0),
            self.encode_corner2(1.0, 1.0),
        ];

        return r;
    }

    pub fn encode_corner2(&self, cx: f32, cy: f32) -> GlyphyUV {
        return self.glyph_vertex_encode2(cx, cy);
    }

    pub fn encode_corner(&self, cx: f32, cy: f32, font_size: f32) -> GlyphyVertex {
        let vx = font_size * ((1.0 - cx) * self.extents.mins.x + cx * self.extents.maxs.x);

        let vy = font_size * ((1.0 - cy) * self.extents.mins.y + cy * self.extents.maxs.y);

        return self.glyph_vertex_encode(vx, vy, cx, cy);
    }

    /**
     * 顶点 编码
     */
    pub fn glyph_vertex_encode(
        &self,
        x: f32,
        y: f32,
        corner_x: f32,
        corner_y: f32, // 0 代表 左/上，1代表 右/下
    ) -> GlyphyVertex {
        let encoded = glyph_encode(
            self.atlas_x as i32,
            self.atlas_y as i32,
            corner_x as i32,
            corner_y as i32,
            self.nominal_w as i32,
            self.nominal_h as i32,
        );

        return GlyphyVertex {
            x,
            y,
            g16hi: encoded >> 16,
            g16lo: encoded & 0xFFFF,
        };
    }

    /**
     * 顶点 编码
     */
    pub fn glyph_vertex_encode2(
        &self,
        corner_x: f32,
        corner_y: f32, // 0 代表 左/上，1代表 右/下
    ) -> GlyphyUV {
        let encoded = glyph_encode(
            self.atlas_x as i32,
            self.atlas_y as i32,
            corner_x as i32,
            corner_y as i32,
            self.nominal_w as i32,
            self.nominal_h as i32,
        );

        return GlyphyUV {
            g16hi: encoded >> 16,
            g16lo: encoded & 0xFFFF,
        };
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

pub struct GlyphyUV {
    // Glyph 信息，具体包含内容如下：
    //   + 纹理 起始位置
    //   + corner_x / corner_y: 0 代表 左 / 上，1代表 右 / 下
    //   + 格子个数（宽，高）
    pub g16hi: i32,
    pub g16lo: i32,
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
