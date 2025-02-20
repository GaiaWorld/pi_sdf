//! 该模块提供用于处理矢量图形签名距离场(SDF)生成和纹理编码的工具函数和结构体。
//!
//! 主要功能包括：
//! - 字体轮廓的解析和处理
//! - SDF纹理的生成和编码
//! - 几何图元（圆弧、线段）的转换和优化
//! - WebAssembly支持
use core::fmt;
use std::collections::HashMap;
// use std::collections::BTreeMap as HashMap;

use ab_glyph_rasterizer::Rasterizer;
use allsorts::{
    outline::OutlineSink,
    pathfinder_geometry::{line_segment::LineSegment2F, vector::Vector2F},
};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
// use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use usvg::{Color, Fill, NonZeroPositiveF64, Paint, Stroke};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    glyphy::{
        blob::{travel_data, BlobArc},
        geometry::{aabb::Aabb, arcs::GlyphyArcAccumulator},
        sdf::glyphy_sdf_from_arc_list3,
        util::float2_equals,
    },
    Point,
};
use parry2d::math::Vector;
use std::hash::Hasher;

use crate::{
    font::FontFace,
    glyphy::{
        blob::{line_encode, snap, Extents, UnitArc},
        geometry::{
            arc::{Arc, ArcEndpoint},
            line::Line,
            point::PointExt,
            vector::VectorEXT,
        },
        util::{is_inf, GLYPHY_INFINITY},
    },
};

pub static MIN_FONT_SIZE: f32 = 10.0;

pub static TOLERANCE: f32 = 10.0 / 1024.;

pub static ENLIGHTEN_MAX: f32 = 0.0001; /* Per EM */

pub static EMBOLDEN_MAX: f32 = 0.0001; /* Per EM */

pub static SCALE: f32 = 2048.0;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TexInfo2 {
    pub sdf_offset_x: usize,
    /// SDF纹理的y轴偏移量
    pub sdf_offset_y: usize,
    /// 字符的水平推进量
    pub advance: f32,
    /// 当前处理的字符
    pub char: char,
    /// 平面X坐标的最小值
    pub plane_min_x: f32,
    /// 平面Y坐标的最小值
    pub plane_min_y: f32,
    /// 平面X坐标的最大值
    pub plane_max_x: f32,
    /// 平面Y坐标的最大值
    pub plane_max_y: f32,
    /// 纹理集X坐标的最小值
    pub atlas_min_x: f32,
    /// 纹理集Y坐标的最小值
    pub atlas_min_y: f32,
    /// 纹理集X坐标的最大值
    pub atlas_max_x: f32,
    /// 纹理集Y坐标的最大值
    pub atlas_max_y: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone)]

/// SdfInfo2结构体用于存储SDF纹理信息以及相关的辅助数据
pub struct SdfInfo2 {
    /// 包含SDF纹理的位置信息
    pub tex_info: TexInfo2,
    /// SDF纹理的具体数据，使用Vec<u8>存储
    pub sdf_tex: Vec<u8>,
    /// 纹理的大小，使用u32表示
    pub tex_size: u32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone, Serialize, Deserialize)]

/// OutlineInfo结构体用于存储字符的轮廓信息
pub struct OutlineInfo {
    /// 当前处理的字符
    pub(crate) char: char,
    pub endpoints: Vec<ArcEndpoint>,
    pub bbox: Vec<f32>,
    pub advance: u16,
    pub units_per_em: u16,
    pub extents: Vec<f32>,
    // #[cfg(feature = "debug")]
    pub svg_paths: Vec<String>,
}

impl OutlineInfo {
    /// 计算字符的附近圆弧，返回CellInfo结构体，其中包含圆弧集合和其他相关数据
    ///
    /// # 参数
    /// * `scale` - 缩放比例因子，用于调整计算过程中的比例
    ///
    /// # 返回
    /// * `CellInfo` - 包含字符轮廓的圆弧集合和相关元数据
    pub fn compute_near_arcs(&self, scale: f32) -> CellInfo {

        let r = FontFace::compute_near_arcs(
            Aabb::new(
                Point::new(self.extents[0], self.extents[1]),
                Point::new(self.extents[2], self.extents[3]),
            ),
            scale,
            &self.endpoints
        );
        // println!("char: {} CellInfo: {:?}", self.char, (&r.arcs.len(), &r.info));
        r
    }

    /// 计算字符的布局信息，包括字符在纹理中的位置、大小等
    ///
    /// # 参数
    /// * `tex_size` - 纹理的大小
    /// * `pxrange` - 像素范围
    /// * `cur_off` - 当前偏移量
    ///
    /// # 返回
    /// * `LayoutInfo` - 包含字符布局的详细信息
    pub fn compute_layout(&self, tex_size: usize, pxrange: u32, cur_off: u32) -> LayoutInfo {
        compute_layout(
            &self.extents,
            tex_size,
            pxrange,
            self.units_per_em,
            cur_off,
            false,
        )
    }

    /// 生成字符的SDF纹理信息
    ///
    /// # 参数
    /// * `result_arcs` - 包含字符轮廓的圆弧信息
    /// * `tex_size` - 纹理的大小
    /// * `pxrange` - 像素范围
    /// * `is_outer_glow` - 是否为外发光效果
    /// * `cur_off` - 当前偏移量
    ///
    /// # 返回
    /// * `SdfInfo2` - 包含SDF纹理数据和布局信息的结构体
    pub fn compute_sdf_tex(
        &self,
        result_arcs: CellInfo,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
    ) -> SdfInfo2 {
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = self.compute_layout(tex_size, pxrange, cur_off);
        let extents = Aabb::new(
            Point::new(extents[0], extents[1]),
            Point::new(extents[2], extents[3]),
        );
        let CellInfo { arcs, info, .. } = result_arcs;
        let pixmap = encode_sdf(
            &arcs,
            info,
            &extents,
            tex_size as usize,
            distance,
            None,
            is_outer_glow,
            false,
            None,
        );

        SdfInfo2 {
            tex_info: TexInfo2 {
                char: self.char,
                advance: self.advance as f32 / self.units_per_em as f32,
                sdf_offset_x: 0,
                sdf_offset_y: 0,
                plane_min_x: plane_bounds[0],
                plane_min_y: plane_bounds[1],
                plane_max_x: plane_bounds[2],
                plane_max_y: plane_bounds[3],
                atlas_min_x: atlas_bounds[0],
                atlas_min_y: atlas_bounds[1],
                atlas_max_x: atlas_bounds[2],
                atlas_max_y: atlas_bounds[3],
            },
            sdf_tex: pixmap,
            tex_size,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl OutlineInfo {
    /// 通过 wasm 绑定计算字符的附近圆弧信息，并返回序列化后的字节数组
    ///
    /// # 参数
    /// * `outline` - 轮廓信息的字节数组输入
    /// * `scale` - 缩放比例因子
    ///
    /// # 返回
    /// * `Vec<u8>` - 序列化后的字符轮廓圆弧信息字节数组
    pub fn compute_near_arcs_of_wasm(outline: &[u8], scale: f32) -> Vec<u8> {
        let outline: OutlineInfo = bitcode::deserialize(outline).unwrap();
        bitcode::serialize(&outline.compute_near_arcs(scale)).unwrap()
    }

    /// 通过 wasm 绑定计算字符的 SDF 纹理，并返回序列化后的字节数组
    ///
    /// # 参数
    /// * `result_arcs` - 包含字符轮廓的圆弧信息字节数组
    /// * `extents` - 字符的范围数组
    /// * `units_per_em` - 每em单位的数量，用于缩放计算
    /// * `advance` - 字符的水平推进量，表示字符的宽度
    /// * `tex_size` - 纹理的大小，以像素为单位
    /// * `pxrange` - 用于计算距离场的像素范围
    /// * `is_outer_glow` - 是否应用外发光效果
    /// * `cur_off` - 当前的偏移量，用于多字符布局
    ///
    /// # 返回
    /// * `Vec<u8>` - 序列化后的 SDF 纹理信息字节数组
    pub fn compute_sdf_tex_of_wasm(
        result_arcs: &[u8],
        extents: &[f32],
        units_per_em: u16,
        advance: u16,
        tex_size: usize,
        pxrange: u32,
        is_outer_glow: bool,
        cur_off: u32,
    ) -> Vec<u8> {
        let result_arcs: CellInfo = bitcode::deserialize(result_arcs).unwrap();
        let LayoutInfo {
            plane_bounds,
            atlas_bounds,
            distance,
            tex_size,
            extents,
        } = compute_layout(extents, tex_size, pxrange, units_per_em, cur_off, false);
        let extents = Aabb::new(
            Point::new(extents[0], extents[1]),
            Point::new(extents[2], extents[3]),
        );
        let CellInfo { arcs, info, .. } = result_arcs; // 解构CellInfo，获取arcs和info字段
        let pixmap = encode_sdf( // 调用encode_sdf生成SDF纹理数据
            &arcs, // 传递圆弧的引用
            info, // 传递CellInfo的info字段，可能包含其他元数据信息
            &extents, // 传递字符的范围信息，表示字符在平面中的位置和大小
            tex_size as usize, // 纹理的大小，确保类型正确
            distance, // 用于生成距离场的距离值
            None, // 可能是其他参数的默认值，暂时保留为None
            is_outer_glow, // 传递是否应用外发光效果的标志
            false, // 可能的其他参数，默认为false
            None, // 可能的其他参数，默认为None
        ); // 调用结束后， pixmap就包含了生成的SDF纹理数据
        // 使用生成的SDF纹理数据和其他布局信息，构造SdfInfo2结构体实例，并将其序列化为字节数组返回
        bitcode::serialize(&SdfInfo2 {
            tex_info: TexInfo2 { // 构造TexInfo2结构体实例
                char: ' ', // 当前处理的字符，这里示例中使用空格，实际应根据情况设置为对应的字符
                advance: advance as f32 / units_per_em as f32, // 计算字符的推进量，即宽度比例
                sdf_offset_x: 0, // SDF纹理在x轴的偏移量，默认为0，可以根据布局需求调整
                sdf_offset_y: 0, // SDF纹理在y轴的偏移量，默认为0，可以根据布局需求调整
                plane_min_x: plane_bounds[0], // 平面坐标系中x的最小值
                plane_min_y: plane_bounds[1], // 平面坐标系中y的最小值
                plane_max_x: plane_bounds[2], // 平面坐标系中x的最大值
                plane_max_y: plane_bounds[3], // 平面坐标系中y的最大值
                atlas_min_x: atlas_bounds[0], // 纹理集坐标系中x的最小值
                atlas_min_y: atlas_bounds[1], // 纹理集坐标系中y的最小值
                atlas_max_x: atlas_bounds[2], // 纹理集坐标系中x的最大值
                atlas_max_y: atlas_bounds[3], // 纹理集坐标系中y的最大值
            }, // 结束TexInfo2结构体的构造
            sdf_tex: pixmap, // 将pixmap设置为SDF纹理数据字段
            tex_size, // 设置纹理大小，这里直接使用计算出的tex_size值，并确保类型正确可达u32类型，若有需要可以进行调整，例如tex_size as u32
        }) // 结束SdfInfo2结构体的构造，并将其作为参数传递给serialize函数进行序列化处理
        .unwrap() // 使用unwrap处理Result，假设序列化总是成功，或者根据需要替换为error处理机制
    } // 结束compute_sdf_tex_of_wasm函数定义，返回序列化后的字节数组

    /// 通过 wasm 绑定计算字符的布局信息，并返回序列化后的字节数组
    ///
    /// # 参数
    /// * `extents` - 字符的范围数组
    /// * `units_per_em` - 每em单位的数量，用于缩放计算
    /// * `tex_size` - 纹理的大小，以像素为单位
    /// * `pxrange` - 用于计算距离场的像素范围
    /// * `cur_off` - 当前的偏移量，用于多字符布局
    ///
    /// # 返回
    /// * `Vec<f32>` - 序列化后的布局信息数组，包括平面界限、纹理集界限、范围界限、距离和纹理大小
    pub fn compute_layout_of_wasm(
        extents: &[f32],
        units_per_em: u16,
        tex_size: usize,
        pxrange: u32,
        cur_off: u32,
    ) -> Vec<f32> {
        let LayoutInfo {
            mut plane_bounds,
            mut atlas_bounds,
            mut extents,
            distance,
            tex_size,
        } = compute_layout(extents, tex_size, pxrange, units_per_em, cur_off, false);
        let mut res = Vec::with_capacity(14);
        res.append(&mut plane_bounds);
        res.append(&mut atlas_bounds);
        res.append(&mut extents);
        res.push(distance);
        res.push(tex_size as f32);
        res
    }
}
/// 用户相关的数据结构，包含累积信息和路径信息
pub struct User { // 用户自定义数据结构的定义，包含 glyphs累积器，路径字符串，svg路径和路径终点坐标
    pub accumulate: GlyphyArcAccumulator, // glyphs累积器，用于累积弧段信息，可能是多字符布局所需的累积结构
    pub path_str: String, // 路径字符串，用于描述图形路径的字符串表示，可能是SVG路径描述字符串或其他格式
    pub svg_paths: Vec<String>, // 包含多个SVG路径字符串的向量，用于存储多个路径的信息，例如多个字符的耦合路径段
    pub svg_endpoints: Vec<[f32; 2]>, // 包含坐标的向量，每个元素为一个坐标点的数组，用于存储各个路径的终点坐标，便于后续处理和绘制
}

/// GlyphVisitor用于处理字体路径数据的结构体
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct GlyphVisitor {
    /// 用于光栅化的Rasterizer对象
    _rasterizer: Rasterizer,
     /// 用于累积路径数据的GlyphyArcAccumulator对象
    pub(crate) accumulate: GlyphyArcAccumulator,
    /// 字体路径的字符串表示（仅在调试模式下有效）
    #[cfg(feature = "debug")]
    pub(crate) path_str: String,
    /// SVG路径的集合
    pub(crate) svg_paths: Vec<String>,
    /// SVG路径的终点集合
    pub(crate) svg_endpoints: Vec<[f32; 2]>,

    scale: f32,
    // scale2: f32,
     /// 起始点
    pub(crate) start: Point,
    /// 上一个点
    pub(crate) previous: Point,
    pub index: usize,
    /// 边界框
    pub(crate) bbox: Aabb,
     /// 弧的数量
    pub(crate) arcs: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl GlyphVisitor {
    /// GlyphVisitor的构造函数，创建一个新的GlyphVisitor实例
    /// 
    /// # 参数
    /// * `scale` - 标尺因子，用于缩放坐标
    pub fn new(scale: f32) -> Self {
        let accumulate = GlyphyArcAccumulator::new();
        let _rasterizer = ab_glyph_rasterizer::Rasterizer::new(512, 512);
        Self {
            _rasterizer,
            accumulate,
            #[cfg(feature = "debug")]
            path_str: "".to_string(),
            // #[cfg(feature = "debug")]
            svg_paths: vec![],
            svg_endpoints: vec![],
            scale,
            // scale2,
            start: Point::default(),
            previous: Point::default(),
            index: 0,
            bbox: Aabb::new(
                Point::new(core::f32::MAX, core::f32::MAX),
                Point::new(core::f32::MIN, core::f32::MIN),
            ),
            arcs: 0,
        }
    }
}

/// OutlineSinkExt trait的扩展实现
pub trait OutlineSinkExt: OutlineSink {
    fn arc2_to(&mut self, d: f32, to: Vector2F);
}

impl OutlineSinkExt for GlyphVisitor {
    fn arc2_to(&mut self, d: f32, to: Vector2F) {
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::debug!("+ A {} {} ", to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.arc_to(to, d);
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("L {} {} ", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_line(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(to.x, to.y),
        //     );
        // }
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }
}

impl OutlineSink for GlyphVisitor {
     /// 移动到指定点的方法
    /// 
    /// # 参数
    /// * `to` - 目标点
    fn move_to(&mut self, to: Vector2F) {
        self.arcs += 1;
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::debug!("M {} {} ", to.x, to.y);

        // if self.scale > 0.02 {
        self.accumulate.move_to(Point::new(to.x, to.y));
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("M {} {} ", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // }
        self.bbox.extend_by(to.x, to.y);
        self.start = to;
        self.previous = to;
    }

    /// 直线到指定点的方法
    /// 
    /// # 参数
    /// * `to` - 目标点
    fn line_to(&mut self, to: Vector2F) {
        let to = Point::new(to.x(), to.y()) * self.scale;
        log::debug!("+ L {} {} ", to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.line_to(to);
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("L {} {} ", to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_line(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(to.x, to.y),
        //     );
        // }
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }

     /// 二次贝塞尔曲线到指定点的方法
    /// 
    /// # 参数
    /// * `control` - 控制点
    /// * `to` - 目标点
    fn quadratic_curve_to(&mut self, control: Vector2F, to: Vector2F) {
        let control = Point::new(control.x(), control.y()) * self.scale;
        let to = Point::new(to.x(), to.y()) * self.scale;

        log::debug!("+ Q {} {} {} {} ", control.x, control.y, to.x, to.y);
        // if self.scale > 0.02 {
        self.accumulate.conic_to(control, to);
        #[cfg(feature = "debug")]
        self.path_str.push_str(&format!("Q {} {} {} {} ", control.x, control.y, to.x, to.y));
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_quad(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(control.x * self.scale, control.y * self.scale),
        //         point(to.x * self.scale, to.y * self.scale),
        //     );
        // }
        self.bbox.extend_by(control.x, control.y);
        self.bbox.extend_by(to.x, to.y);
        self.previous = to;
    }

     /// 三次贝塞尔曲线到指定点的方法
    /// 
    /// # 参数
    /// * `control` - 控制线段
    /// * `to` - 目标点
    fn cubic_curve_to(&mut self, control: LineSegment2F, to: Vector2F) {
        // 字形数据没有三次贝塞尔曲线
        let control1 = Point::new(control.from_x(), control.from_y()) * self.scale;
        let control2 = Point::new(control.to_x(), control.to_y()) * self.scale;
        let to = Point::new(to.x(), to.y()) * self.scale;

        log::debug!(
            "+ C {}, {}, {}, {}, {}, {}",
            control1.x,
            control1.y,
            control2.x,
            control2.y,
            to.x,
            to.y
        );

        // if self.scale > 0.02 {
        self.accumulate.cubic_to(control1, control2, to);
        self.svg_endpoints.push([to.x, to.y]);
        // } else {
        //     self.rasterizer.draw_cubic(
        //         point(self.previous.x * self.scale, self.previous.y * self.scale),
        //         point(control1.x * self.scale, control1.y * self.scale),
        //         point(control1.x * self.scale, control1.y * self.scale),
        //         point(to.x * self.scale, to.y * self.scale),
        //     );
        // }
        // }
        self.bbox.extend_by(control1.x, control1.y);
        self.bbox.extend_by(control2.x, control2.y);
        self.bbox.extend_by(to.x, to.y);

        self.previous = to;
    }

    /// 关闭路径的方法
    fn close(&mut self) {
        if self.previous != self.start {
            log::debug!("+ L {} {} ", self.start.x, self.start.y);
            // if self.scale > 0.02 {
            self.accumulate.line_to(self.start);
            // #[cfg(feature = "debug")]
            // self.path_str
            //     .push_str(&format!("M {} {}", self.start.x, self.start.y));
            self.svg_endpoints.push([self.start.x, self.start.y]);
            // } else {
            //     let x = self.previous.x * self.scale;
            //     self.rasterizer.draw_line(
            //         point(x, (self.previous.y) * self.scale),
            //         point(self.start.x * self.scale, self.start.y * self.scale),
            //     )
            // }
        }
        log::debug!("+ Z");
        // if self.scale > 0.02 {
        self.accumulate.close_path();
        #[cfg(feature = "debug")]
        {
            self.path_str.push_str("Z");
            self.svg_paths.push(self.path_str.clone());
            self.path_str.clear();
        }
        // }

        // let r = self.compute_direction();
        // let s = if r { "顺时针" } else { "逆时针" };
        // log::debug!("{}", s);
        self.index = self.accumulate.result.len();
        // log::debug!("close()");
    }
}

/// 将输入的矢量数据转换为纹理表示的SDF格式
///
/// # 参数
/// * `global_arcs` - 存储所有弧段的全局列表
/// * `arcs_info` - 每个网格点对应的弧段索引及其在矢量路径中所占据的位置
/// * `extents` - 矢量形状的包围盒
/// * `tex_size` - 生成的纹理宽度和高度（二维纹理，方形纹理，边长为tex_size）
/// * `distance` - 在该距离内，alpha值衰减为0的点（边缘范围的截止值）
/// * `width` - SVG转换中的线宽（未实现，保留为None）
/// * `is_outer_glow` - 是否应用外发光效果（闪烁效果）
/// * `is_svg` - 是否作为SVG路径进行处理（影响坐标变换的方向）
/// * `is_reverse` - 是否反转颜色通道（区分前景与背景）
/// # 返回值
/// 生成的SDF纹理作为一维数组，每个元素代表一个纹理点的值。
pub fn encode_sdf(
    global_arcs: &Vec<Arc>,
    arcs_info: Vec<(Vec<usize>, Aabb)>,
    extents: &Aabb,
    tex_size: usize,
    distance: f32, // sdf在这个值上alpha 衰减为 0
    width: Option<f32>,
    is_outer_glow: bool,
    is_svg: bool,
    is_reverse: Option<bool>,
) -> Vec<u8> {
    // 除非字符的高度不是0，否则方阵将无法正确生成。极端情况下字符宽度为0时可能会导致计算错误，应当进行判断，但根据问题描述，应不会出现这种情况。
    // 假设矢量形状包围盒的width不为零
    let glyph_width = extents.width(); // 计算矢量形状的宽度
    // 假设高度不为零，暂未处理高度为零的情况

    // 计算每单元在包围盒中的尺寸宽度，该值为纹理单位的缩放因子
    let unit_d = glyph_width / tex_size as f32; // 计算每单元宽度

    // 初始化所有纹理点的值为0
    let mut data = vec![0; tex_size * tex_size]; // 创建一个一维数组用于存储最终的纹理数据

    // 遍历每个网格点（cell），每个cell对应一个弧段列表，以及该cell在矢量形状包围盒中的所在区域
    for (near_arcs, cell) in arcs_info {  // 遍历每个预处理好的单元格
        if let Some(ab) = cell.collision(extents) { // 确定该单元格是否在矢量过程中实际占用空间
            // Compute the relative positions for this tile in texture space中将包围盒偏移一个单元尺寸，以适合整数坐标计算
            let begin = ab.mins - extents.mins; // 将包围盒的起点坐标系原点设为中心点坐标系下的张量起点，便于计算
            let end = ab.maxs - extents.mins;   // 围绕盒的终点

            // 将包围盒的起点坐标系转换为纹理坐标系，单位缩放为unit_d，并转换为整数索引
            let mut begin_x = begin.x / unit_d;
            // 使用四舍五入的方式将浮点数转换为整数索引，避免坐标系偏移误差
            begin_x = (begin_x * 10000.0).round() * 0.0001; // 四舍五入
            let begin_x = begin_x.round() as usize;

            // 同样处理其他坐标轴
            let mut begin_y = begin.y / unit_d;
            begin_y = (begin_y * 10000.0).round() * 0.0001;
            let begin_y = begin_y.round() as usize;

            let mut end_x = end.x / unit_d;
            end_x = (end_x * 10000.0).round() * 0.0001;
            let end_x = end_x.round() as usize;

            let mut end_y = end.y / unit_d;
            end_y = (end_y * 10000.0).round() * 0.0001;
            let end_y = end_y.round() as usize;

            // 遍历该单元格在纹理中的对应区域，每个(i,j)点即为纹理中的一个点，对应的2D坐标i,j转换为线性数组索引i + j * tex_size
            for i in begin_x..end_x {  // 纹理x轴方向遍历
                for j in begin_y..end_y {   // 纹理y轴方向遍历
                    // 将纹理点的i,j转回矢量空间的具体点p。由于每个单元的中心点对应于i+0.5的位置，所以坐标转换要考虑scale和offset
                    let p = Point::new(
                        (i as f32 + 0.5) * unit_d + extents.mins.x,  // 计算x坐标
                        (j as f32 + 0.5) * unit_d + extents.mins.y   // 计算y坐标
                    );

                    // 调用计算函数，计算该点p处的SDF值
                    let r = compute_sdf2(
                        global_arcs,
                        p,
                        &near_arcs,
                        distance,
                        width,
                        is_outer_glow,
                        is_reverse,
                    );

                    // 根据是否是_svg模式，调整点p在数据数组中的索引。_svg模式则不需要颠倒y轴，否则颠倒y轴以适应纹理坐标系
                    if is_svg {
                        // 对SVG不存在颠倒，索引i,j直接访问
                        data[j * tex_size + i] = r.0;
                    } else {
                        // 非-svg模式下，颠倒y轴，将纹理的y轴坐标从下往上存储，即(y) -> (tex_size - 1 - y)
                        data[(tex_size - j - 1) * tex_size + i] = r.0;
                    }
                }
            }
        }
    }
    data // 返回生成的纹理数据
}
/// 计算SDF的函数，用于对每个点p进行采样，计算其对应SDF的值。
///
/// 此函数主要用于svg和字体贴图的sdf生成。函数会根据输入的参数计算出点p处的sdf值，并根据一些后续处理参数进行调整。
///
/// # 参数说明
/// * `global_arcs` - 全局的弧段列表，包含整个矢量形状的各个弧线段信息。
/// * `p` - 当前需要计算SDF的点的位置。
/// * `near_arcs` - 当前单元内的近邻弧段列表。用于快速计算该点的SDF。
/// * `distance` - SDF衰减距离，超过这个距离的点，其alpha值为0。
/// * `width` - 轮廓宽度，如果非空，则将宽度中心设定为0.5，否则将取具体的距离。
/// * `is_outer_glow` - 是否应用外发光效果，主要用于闪烁效果处理。
/// * `is_reverse` - 是否反转颜色通道，用于区分前景与背景颜色。
///
/// # 返回值
/// 一个元组，包含：
/// - `u8`：处理后的sdf值，用于颜色显示。
/// - `f32`：原始的sdf值，主要用于后续的计算处理。
/// - `f32`：经过处理的sdf2值，同样用于后续的计算或显示。
pub fn compute_sdf2(
    global_arcs: &Vec<Arc>,         // v作为arc的全局集合输入接口模式不影响内部实现，无需改变变量类型，否则会导致编译错误。但此处是智能指针，应该可以处理。
    p: Point,                       // 点p。能发现它是参数，可以通过显式传递进行计算。
    near_arcs: &Vec<usize>,         // 当前单元格内的近邻arc索引列表，用于快速计算。
    distance: f32,                  // 衰减距离。
    width: Option<f32>,             // 宽度参数，可能影响中心点的处理。
    is_outer_glow: bool,            // 是否外发光效果。
    is_reverse: Option<bool>,       // 是否反转颜色通道。
) -> (u8, f32, f32) {
    let mut sdf = glyphy_sdf_from_arc_list3(near_arcs, p.clone(), global_arcs).0;
    // 去除浮点误差
    sdf = (sdf * 10000.0).round() * 0.0001;

    if let Some(is_reverse) = is_reverse {
        if is_reverse {
            sdf = -sdf;
        }
    }
    if let Some(_) = width {
        sdf = sdf.abs(); // - (width * 0.5);
    }

    if is_outer_glow {
        let sdf2 = (1.0 - (sdf / distance)).powf(1.99);
        return ((sdf2 * 255.0).round() as u8, sdf, sdf2);
        // log::debug!("{:?}", (radius, sdf));
    } else {
        let sdf2 = sdf / distance;
        let a = ((1.0 - sdf2) * 127.0).round() as u8;
        // log::debug!("sdf: {:?}", (sdf, a, distance));
        return (a, sdf, sdf2);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LayoutInfo {
    pub plane_bounds: Vec<f32>,
    pub atlas_bounds: Vec<f32>,
    pub extents: Vec<f32>,
    pub distance: f32,
    pub tex_size: u32,
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub(crate) fn compute_layout(
    extents: &[f32],
    tex_size: usize,
    pxrange: u32,
    units_per_em: u16,
    cur_off: u32,
    is_svg: bool,
) -> LayoutInfo {
    // 创建一个aabb包围盒用于存储矢量图的几何范围。
    // map 无序导致每次计算的数据不一样
    let mut extents2 = Aabb::new(
        Point::new(extents[0], extents[1]),
        Point::new(extents[2], extents[3]),
    );
    // 计算矢量图的宽度和高度。
    let extents_w = extents2.width();
    let extents_h = extents2.height();
    // 计算缩放比例，将矢量图的尺寸转换为适用的单位。
    let scale = 1.0 / units_per_em as f32;
    let plane_bounds = extents2.scaled(&Vector::new(scale, scale));

    // 计算每个像素在矢量图中的距离。
    let px_distance = extents_w.max(extents_h) / tex_size as f32;
    let distance = px_distance * pxrange as f32;
    // 扩展矢量图的包围盒，以适应纹理边缘的处理。
    let expand = px_distance * cur_off as f32;
    extents2.mins.x -= expand;
    extents2.mins.y -= expand;
    extents2.maxs.x += expand;
    extents2.maxs.y += expand;

    // 计算考虑到偏移后的纹理大小。
    let tex_size = tex_size + (cur_off * 2) as usize;
    // 初始化atlas.Bounds，并根据当前偏移设置其边界范围。
    let mut atlas_bounds = Aabb::new_invalid();
    atlas_bounds.mins.x = cur_off as f32;
    atlas_bounds.mins.y = cur_off as f32;
    atlas_bounds.maxs.x = tex_size as f32 - cur_off as f32;
    atlas_bounds.maxs.y = tex_size as f32 - cur_off as f32;

    // 根据矢量图的宽高差异进行调整，确保矢量图在纹理中正确映射。
    let temp = extents_w - extents_h;
    if temp > 0.0 {
        extents2.maxs.y += temp;
        if is_svg {
            // 对于svg格式，调整atlas_bounds的高度以匹配矢量图的高，避免裁剪。
            atlas_bounds.maxs.y -= (temp / extents2.height() * tex_size as f32).trunc();
        } else {
            // 对于非svg情况，调整atlas_bounds的底部以适应矢量图的高，可能反转y轴。
            atlas_bounds.mins.y += (temp / extents2.height() * tex_size as f32).ceil();
        }
    } else {
        extents2.maxs.x -= temp;
        // 调整atlas_bounds的宽度以匹配矢量图的宽。
        atlas_bounds.maxs.x -= (temp.abs() / extents2.width() * tex_size as f32).trunc();
    }

    // 输出调试信息，显示plane.Bounds，atlas.Bounds和tex_size。
    log::debug!(
        "plane_bounds: {:?}, atlas_bounds: {:?}, tex_size: {}",
        plane_bounds, atlas_bounds, tex_size
    );

    // 创建并返回LayoutInfo结构体，包含计算后的各参数。
    LayoutInfo {
        plane_bounds: vec![
            plane_bounds.mins.x,
            plane_bounds.mins.y,
            plane_bounds.maxs.x,
            plane_bounds.maxs.y,
        ],
        atlas_bounds: vec![
            atlas_bounds.mins.x,
            atlas_bounds.mins.y,
            atlas_bounds.maxs.x,
            atlas_bounds.maxs.y,
        ],
        extents: vec![
            extents2.mins.x,
            extents2.mins.y,
            extents2.maxs.x,
            extents2.maxs.y,
        ],
        distance,
        tex_size: tex_size as u32,
    }
}

// 它根据输入的比例参数进行调整，确保包围盒的范围适合显示需求。
pub fn compute_cell_range(mut bbox: Aabb, scale: f32) -> Aabb {
    // 缩放因子设置为输入值的一半，用于调整包围盒的尺寸。
    let scale = scale * 0.5;
    // 计算当前包围盒的宽度和高度。
    let w = bbox.width();
    let h = bbox.height();
    // 确定宽度和高度的差异，用于后续调整。
    let temp = w - h;
    // 根据差异调整包围盒的范围，确保合适的空间分配。
    if temp > 0.0 {
        // 如果宽度大于高度，调整y轴范围。
        bbox.maxs.y += temp;
    } else {
        // 如果高度大于宽度，调整x轴范围。
        bbox.maxs.x -= temp;
    }
    // 计算调整后的包围盒宽度。
    let w = bbox.width();
    // 计算扩展的大小，用于延伸包围盒。
    let extents = scale * w;
    // 扩展包围盒的每个边，确保内容不会被裁剪。
    bbox.mins.x -= extents;
    bbox.mins.y -= extents;
    bbox.maxs.x += extents;
    bbox.maxs.y += extents;

    bbox
}

pub fn to_arc_cmds(endpoints: &Vec<ArcEndpoint>) -> (Vec<Vec<String>>, Vec<[f32; 2]>) {
    // 初始化命令数组和点列表以存储处理结果
    let mut _cmd = vec![];
    let mut cmd_array = vec![];
    let mut current_point = None;
    let mut pts = vec![];
    // 遍历endpoint切片中的每一个元素，处理弧线绘制命令生成SVG路径字符串
    for ep in endpoints {
        pts.push([ep.p[0], ep.p[1]]);
        pts.push([ep.p[0], ep.p[1]]); // 将当前弧的终点坐标加入pts列表

        // 处理GLYPHY_INFINITY类型的端点，即一条新的路径起点
        if ep.d == GLYPHY_INFINITY {
            if current_point.is_none() || !float2_equals(&ep.p, current_point.as_ref().unwrap()) {
                if _cmd.len() > 0 {
                    cmd_array.push(_cmd);
                    _cmd = vec![];
                }
                _cmd.push(format!(" M ${}, ${}", ep.p[0], ep.p[1]));
                current_point = Some(ep.p);
            }
        } else if ep.d == 0.0 {
            assert!(current_point.is_some());
            if current_point.is_some() && !float2_equals(&ep.p, current_point.as_ref().unwrap()) {
                _cmd.push(format!(" L {}, {}", ep.p[0], ep.p[1]));
                current_point = Some(ep.p);
            }
        } else {
            assert!(current_point.is_some());
            let mut _current_point = current_point.as_ref().unwrap();
            if !float2_equals(&ep.p, _current_point) {
                let arc = Arc::new(
                    Point::new(_current_point[0], _current_point[1]),
                    Point::new(ep.p[0], ep.p[1]),
                    ep.d,
                );
                let center = arc.center();
                let radius = arc.radius();
                let start_v =
                    Vector::new(_current_point[0] - center[0], _current_point[1] - center[1]);
                let start_angle = start_v.sdf_angle();

                let end_v = Vector::new(ep.p[0] - center[0], ep.p[1] - center[1]);
                let end_angle = end_v.sdf_angle();

                // 大于0，顺时针绘制
                let cross = start_v.sdf_cross(&end_v);

                _cmd.push(arc_to_svg_a(
                    center.x,
                    center.y,
                    radius,
                    start_angle,
                    end_angle,
                    cross < 0.0,
                ));

                _current_point = &ep.p;
            }
        }
    }
    if _cmd.len() > 0 {
        cmd_array.push(_cmd);
        _cmd = vec![]
    }

    return (cmd_array, pts);
}

pub fn arc_to_svg_a(
    x: f32,
    y: f32,
    radius: f32,
    _start_angle: f32,
    end_angle: f32,
    anticlockwise: bool,
) -> String {
    // 计算圆弧结束点坐标
    let end_x = x + radius * end_angle.cos();
    let end_y = y + radius * end_angle.sin();

    // large-arc-flag 的值为 0 或 1，决定了弧线是大于还是小于或等于 180 度
    let large_arc_flag = '0'; // endAngle - startAngle <= Math.PI ? '0' : '1';

    // sweep-flag 的值为 0 或 1，决定了弧线是顺时针还是逆时针方向
    let sweep_flag = if anticlockwise { '0' } else { '1' };

    // 返回 SVG "A" 命令参数
    return format!(
        "A {} {} 0 {} {} {} {}",
        radius, radius, large_arc_flag, sweep_flag, end_x, end_y
    );
}
pub fn create_indices() -> [u16; 6] {
    [0, 1, 2, 1, 2, 3]
}

#[derive(Debug, Default, Clone)]
pub struct Attribute {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub is_close: bool,
    pub start: Point,
}

unsafe impl Send for Attribute {}
unsafe impl Sync for Attribute {}
impl Attribute {
    pub fn set_fill_color(&mut self, r: u8, g: u8, b: u8) {
        let fill = Fill::from_paint(Paint::Color(Color::new_rgb(r, g, b)));
        self.fill = Some(fill);
    }

    pub fn set_stroke_color(&mut self, r: u8, g: u8, b: u8) {
        if let Some(stroke) = &mut self.stroke {
            stroke.paint = Paint::Color(Color::new_rgb(r, g, b));
        } else {
            let mut stroke = Stroke::default();
            stroke.paint = Paint::Color(Color::new_rgb(r, g, b));
            self.stroke = Some(stroke);
        }
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        if let Some(stroke) = &mut self.stroke {
            stroke.width = NonZeroPositiveF64::new(width as f64).unwrap();
        } else {
            let mut stroke = Stroke::default();
            stroke.width = NonZeroPositiveF64::new(width as f64).unwrap();
            self.stroke = Some(stroke);
        }
    }

    pub fn set_stroke_dasharray(&mut self, dasharray: Vec<f64>) {
        if let Some(stroke) = &mut self.stroke {
            stroke.dasharray = Some(dasharray)
        } else {
            let mut stroke = Stroke::default();
            stroke.dasharray = Some(dasharray);
            self.stroke = Some(stroke);
        }
    }
}

pub fn arc_to_point(arcs: Vec<Arc>) -> Vec<ArcEndpoint> {
    let mut arc_endpoints = Vec::with_capacity(arcs.len());
    let mut _p1 = Point::new(0.0, 0.0);

    for i in 0..arcs.len() {
        let arc = &arcs[i];

        if i == 0 || !_p1.equals(&arc.p0) {
            let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
            arc_endpoints.push(endpoint);
            _p1 = arc.p0;
        }

        let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
        arc_endpoints.push(endpoint);
        _p1 = arc.p1;
    }
    arc_endpoints
}

pub fn point_to_arc(endpoints: Vec<ArcEndpoint>) -> Vec<Arc> {
    let mut p0 = Point::new(0., 0.);
    let mut arcs = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        // let endpoint = &result[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);

        arcs.push(arc);
    }
    arcs
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
#[derive(Debug, Clone)]
pub struct CellInfo {
    pub extents: Aabb,
    pub(crate) arcs: Vec<Arc>,
    pub(crate) info: Vec<(Vec<usize>, Aabb)>,
    pub(crate) min_width: f32,
    pub(crate) min_height: f32,
    pub is_area: bool,
}

impl CellInfo {
    pub fn encode_blob_arc(&self) -> BlobArc {
        let extents = &self.extents;

        let result_arcs = &self.info;
        let global_arcs = &self.arcs;
        let glyph_width = extents.width();
        let glyph_height = extents.height();
        // // 格子列的数量;
        // // todo 为了兼容阴影minimip先强制索引纹理为32 * 32
        let width_cells = (glyph_width / self.min_width).round() as usize;
        // 格子行的数量
        let height_cells = (glyph_height / self.min_height).round() as usize;

        // 格子列的数量
        let min_width = glyph_width / width_cells as f32;
        // 格子行的数量
        let min_height = glyph_height / height_cells as f32;

        let mut data = vec![
            vec![
                UnitArc {
                    parent_cell: Extents {
                        min_x: 0.,
                        min_y: 0.,
                        max_x: 0.,
                        max_y: 0.
                    },
                    offset: 0,
                    sdf: 0.0,
                    #[cfg(feature = "debug")]
                    show: "".to_owned(),
                    data: Vec::with_capacity(8),
                    origin_data: vec![],
                    key: u64::MAX,
                    s_dist: 0,
                    s_dist_1: 0,
                    s_dist_2: 0,
                    s_dist_3: 0
                };
                width_cells
            ];
            height_cells
        ];

        // let glyph_width = extents.width();
        // let glyph_height = extents.height();
        let c = extents.center();
        let unit = glyph_width.max(glyph_height);

        let mut map = HashMap::new();
        // 二分计算时，个格子的大小会不一样
        // 统一以最小格子细分
        for (near_arcs, cell) in result_arcs {
            let mut near_endpoints = Vec::with_capacity(8);
            let mut _p1 = Point::new(0.0, 0.0);

            for i in 0..near_arcs.len() {
                let arc = &global_arcs[near_arcs[i]];

                if i == 0 || !_p1.equals(&arc.p0) {
                    let endpoint = ArcEndpoint::new(arc.p0.x, arc.p0.y, GLYPHY_INFINITY);
                    near_endpoints.push(endpoint);
                    _p1 = arc.p0;
                }

                let endpoint = ArcEndpoint::new(arc.p1.x, arc.p1.y, arc.d);
                near_endpoints.push(endpoint);
                _p1 = arc.p1;
            }
            // log::debug!("near_endpoints: {:?}", near_endpoints.len());

            let begin = cell.mins - extents.mins;
            let end = cell.maxs - extents.mins;
            let begin_x = (begin.x / min_width).round() as usize;
            let begin_y = (begin.y / min_height).round() as usize;

            let end_x = (end.x / min_width).round() as usize;
            let end_y = (end.y / min_height).round() as usize;
            let parent_cell = Extents {
                min_x: cell.mins.x,
                min_y: cell.mins.y,
                max_x: cell.maxs.x,
                max_y: cell.maxs.y,
            };

            let mut line_result = None;
            let mut arc_result = None;
            // 如果是线段段都编码
            if near_endpoints.len() == 2 && near_endpoints[1].d == 0.0 {
                let start = &near_endpoints[0];
                let end = &near_endpoints[1];

                let mut line = Line::from_points(
                    snap(
                        &Point::new(start.p[0], start.p[1]),
                        &extents,
                        glyph_width,
                        glyph_height,
                    ),
                    snap(
                        &Point::new(end.p[0], end.p[1]),
                        &extents,
                        glyph_width,
                        glyph_height,
                    ),
                );
                // Shader的最后 要加回去
                line.c -= line.n.dot(&c.into_vector());
                // shader 的 decode 要 乘回去
                line.c /= unit;

                let line_key = near_endpoints[0].get_line_key(&near_endpoints[1]);
                let le = line_encode(line);

                let mut line_data = ArcEndpoint::new(0.0, 0.0, 0.0);
                line_data.line_key = Some(line_key);
                line_data.line_encode = Some(le);

                line_result = Some((line_data, start.clone(), end.clone()));
            } else {
                if near_endpoints.len() == 4
                    && is_inf(near_endpoints[2].d)
                    && near_endpoints[0].p[0] == near_endpoints[3].p[0]
                    && near_endpoints[0].p[1] == near_endpoints[3].p[1]
                {
                    let e0 = near_endpoints[2].clone();
                    let e1 = near_endpoints[3].clone();
                    let e2 = near_endpoints[1].clone();

                    near_endpoints.clear();
                    near_endpoints.push(e0);
                    near_endpoints.push(e1);
                    near_endpoints.push(e2);
                }

                // 编码到纹理：该格子 对应 的 圆弧数据
                let mut hasher = pi_hash::DefaultHasher::default();
                let mut key = Vec::with_capacity(20);
                for endpoint in &near_endpoints {
                    key.push(endpoint.p[0]);
                    key.push(endpoint.p[1]);
                    key.push(endpoint.d);
                }
                hasher.write(bytemuck::cast_slice(&key));
                let result = hasher.finish();

                arc_result = Some(result);
            }

            // If the arclist is two arcs that can be combined in encoding if reordered, do that.
            for i in begin_x..end_x {
                for j in begin_y..end_y {
                    let unit_arc = &mut data[j][i];
                    if let Some((line_data, start, end)) = line_result.as_ref() {
                        unit_arc.data.push(line_data.clone());
                        // log::debug!("1row: {}, col: {} line_data: {:?}n \n", row, col, unit_arc.data.len());
                        unit_arc.origin_data.push(start.clone());
                        unit_arc.origin_data.push(end.clone());
                        unit_arc.parent_cell = parent_cell;
                    } else {
                        let key = arc_result.as_ref().unwrap();
                        unit_arc.data.extend_from_slice(&near_endpoints);
                        unit_arc.parent_cell = parent_cell;
                        unit_arc.key = key.clone();
                    }
                    // log::debug!("i: {}, j: {}, unit_arc: {:?}", i, j, unit_arc);
                }
            }
            let key = data[begin_y][begin_x].get_key();
            let ptr: *const UnitArc = &data[begin_y][begin_x];
            // 使用map 去重每个格子的数据纹理
            map.insert(key, ptr as u64);
        }
        let [min_sdf, max_sdf] = travel_data(&data);

        BlobArc {
            min_sdf,
            max_sdf,
            cell_size: min_width,
            #[cfg(feature = "debug")]
            show: format!("<br> 格子数：宽 = {}, 高 = {} <br>", min_width, min_height),
            extents: *extents,
            data: data,
            avg_fetch_achieved: 0.0,
            endpoints: vec![],
            data_tex_map: map,
        }
    }
}

impl Serialize for CellInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let unit_size = self.extents.width() / 32.0;
        let start_point = self.extents.mins;

        let mut s = serializer.serialize_struct("CellInfo", 9)?;
        s.serialize_field("mins_x", &self.extents.mins.x)?;
        s.serialize_field("mins_y", &self.extents.mins.y)?;
        s.serialize_field("maxs_x", &self.extents.maxs.x)?;
        s.serialize_field("maxs_y", &self.extents.maxs.y)?;
        s.serialize_field("arcs", &self.arcs)?;

        let mut info = Vec::with_capacity(self.info.len());
        for (arcs, ab) in &self.info {
            let offset = (ab.mins - start_point) / unit_size;
            let w = ab.width() / unit_size;
            let h = ab.height() / unit_size;
            let mut temp = Vec::with_capacity(arcs.len());
            for arc in arcs {
                temp.push(*arc as u16);
            }

            info.push((
                temp,
                offset.x.round() as u8,
                offset.y.round() as u8,
                w.round() as u8,
                h.round() as u8,
            ));
        }
        // log::debug!("CellInfo: {}", info.len() )
        s.serialize_field("info", &info)?;
        s.serialize_field("min_width", &self.min_width)?;
        s.serialize_field("min_height", &self.min_height)?;
        s.serialize_field("is_area", &self.is_area)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for CellInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CellInfoVisitor;

        impl<'de> Visitor<'de> for CellInfoVisitor {
            type Value = CellInfo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CellInfo")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<CellInfo, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mins_x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mins_y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let maxs_x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let maxs_y1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let arcs = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let src_info: Vec<(Vec<u16>, u8, u8, u8, u8)> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                let extents = Aabb::new(Point::new(mins_x, mins_y), Point::new(maxs_x, maxs_y1));
                let unit_size = extents.width() / 32.0;

                let mut info = Vec::with_capacity(src_info.len());
                for (indexs, offset_x, offset_y, w, h) in src_info {
                    let min = Point::new(
                        extents.mins.x + unit_size * offset_x as f32,
                        extents.mins.y + unit_size * offset_y as f32,
                    );
                    let max =
                        Point::new(min.x + unit_size * w as f32, min.y + unit_size * h as f32);
                    let mut arc_index = Vec::with_capacity(indexs.len());
                    for i in indexs {
                        arc_index.push(i as usize);
                    }
                    info.push((arc_index, Aabb::new(min, max)));
                }
                let min_width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let min_height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let is_area = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                Ok(CellInfo {
                    extents,
                    arcs,
                    info,
                    min_width,
                    min_height,
                    is_area,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "mins_x",
            "mins_y",
            "maxs_x",
            "maxs_y",
            "arcs",
            "info",
            "min_width",
            "min_height",
            "is_area",
        ];
        deserializer.deserialize_struct("Point", FIELDS, CellInfoVisitor)
    }
}

