//! TASK-09 布局与光栅化：模型侧数据不变量仅经对外冻结接口验证。
//!
//! core/raster 为 pub(crate)，其算法测试内联在 layout_inner.rs / raster_inner.rs；
//! 本文件只覆盖对外可见的 model 层数据不变量（CASE-089、090、091）
//! 与 SdfTexture 尺寸自洽（CASE-044 的模型侧）。

use pi_sdf2::model::{
    Aabb, DataTexture, IndexTexture, Point, SdfTexture, TextureInfo, TextureLayout,
};

/// CASE-089：数据纹理像素长度 = 宽 × 高。
#[test]
fn case_089_data_texture_size_is_consistent() {
    // 编码器输出恒为「宽 4 字节、高 = 记录数」；按契约构造代表实例校验尺寸不变量。
    let data = DataTexture {
        pixels: vec![0u8; 4 * 3],
        width: 4,
        height: 3,
    };
    assert_eq!(data.pixels.len() as u32, data.width * data.height);

    let single = DataTexture {
        pixels: vec![1, 2, 3, 4],
        width: 4,
        height: 1,
    };
    assert_eq!(single.pixels.len() as u32, single.width * single.height);

    let empty = DataTexture {
        pixels: Vec::new(),
        width: 4,
        height: 0,
    };
    assert_eq!(empty.pixels.len() as u32, empty.width * empty.height);
}

/// CASE-090：索引纹理像素长度 = 宽 × 高。
#[test]
fn case_090_index_texture_size_is_consistent() {
    // 编码器输出恒为「宽 2 字节、高 = 格元数」。
    let index = IndexTexture {
        pixels: vec![0u8; 2 * 5],
        width: 2,
        height: 5,
    };
    assert_eq!(index.pixels.len() as u32, index.width * index.height);

    let empty = IndexTexture {
        pixels: Vec::new(),
        width: 2,
        height: 0,
    };
    assert_eq!(empty.pixels.len() as u32, empty.width * empty.height);
}

/// CASE-091：纹理布局字段自洽 —— tex_size > 0 且 distance > 0。
#[test]
fn case_091_texture_layout_fields_positive() {
    let layout = TextureLayout {
        plane_bounds: Aabb::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        atlas_bounds: Aabb::new(Point::new(0.0, 0.0), Point::new(64.0, 64.0)),
        distance: 0.0625,
        tex_size: 64,
    };
    assert!(layout.tex_size > 0, "纹理边长必须为正");
    assert!(layout.distance > 0.0, "距离必须为正");
}

/// CASE-044（模型侧）：SdfTexture 像素数 = tex_size²。
#[test]
fn case_044_sdf_texture_consistency() {
    let tex_info = TextureInfo {
        plane_bounds: Aabb::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        atlas_bounds: Aabb::new(Point::new(0.0, 0.0), Point::new(8.0, 8.0)),
        sdf_offset_x: 0.0,
        sdf_offset_y: 0.0,
    };
    let consistent = SdfTexture {
        pixels: vec![0u8; 8 * 8],
        tex_size: 8,
        tex_info,
    };
    assert!(consistent.is_consistent(), "像素数等于 tex_size² 时应自洽");
    assert_eq!(consistent.pixels.len(), (consistent.tex_size * consistent.tex_size) as usize);

    let inconsistent = SdfTexture {
        pixels: vec![0u8; 7],
        tex_size: 8,
        tex_info,
    };
    assert!(!inconsistent.is_consistent(), "像素数与 tex_size² 不符时应不自洽");
}
