/**
 * 路径动词、路径、图形元
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::base::error::Result;
use crate::model::outline::contour::Outline;
use crate::model::geom::aabb::Aabb;
use crate::model::raster::sdf::SdfTexture;
use crate::model::path::path_inner;

/**
 * 路径动词，判别值与 JS 侧 u8 契约一致（1..=19）。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum PathVerb {
    /// 移动到绝对位置
    MoveTo = 1,
    /// 相对当前位置移动
    MoveToRelative = 2,
    /// 直线到绝对位置
    LineTo = 3,
    /// 相对直线
    LineToRelative = 4,
    /// 二次贝塞尔到绝对位置
    QuadTo = 5,
    /// 相对二次贝塞尔
    QuadToRelative = 6,
    /// 平滑二次贝塞尔到绝对位置
    SmoothQuadTo = 7,
    /// 相对平滑二次贝塞尔
    SmoothQuadToRelative = 8,
    /// 三次贝塞尔到绝对位置
    CubicTo = 9,
    /// 相对三次贝塞尔
    CubicToRelative = 10,
    /// 平滑三次贝塞尔到绝对位置
    SmoothCubicTo = 11,
    /// 相对平滑三次贝塞尔
    SmoothCubicToRelative = 12,
    /// 水平线到绝对位置
    HorizontalLineTo = 13,
    /// 相对水平线
    HorizontalLineToRelative = 14,
    /// 垂直线到绝对位置
    VerticalLineTo = 15,
    /// 相对垂直线
    VerticalLineToRelative = 16,
    /// 椭圆弧到绝对位置
    EllipticalArcTo = 17,
    /// 相对椭圆弧
    EllipticalArcToRelative = 18,
    /// 关闭路径
    Close = 19,
}

/**
 * 路径动词的 JS 友好构造与取值（trait impl 无法导出，故另加固有方法）。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 */
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PathVerb {
    /// 由判别值还原路径动词。
    ///
    /// 契约：API-026
    ///
    /// 约束：
    ///   - requires  判别值落在 1..=19
    ///   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
    ///   - 错误      InvalidPathVerb —— 判别值不在 1..=19
    ///
    /// 参数：value — 路径动词判别值
    pub fn try_from_u8(value: u8) -> Result<Self> {
        path_inner::path_verb_try_from(value)
    }

    /// 取路径动词判别值。
    ///
    /// 契约：API-026
    ///
    /// 约束：
    ///   - requires  判别值落在 1..=19
    ///   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
    ///   - 错误      InvalidPathVerb —— 判别值不在 1..=19
    pub fn to_u8(self) -> u8 {
        path_inner::path_verb_to_u8(self)
    }
}

/**
 * 由判别值还原路径动词。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 */
impl TryFrom<u8> for PathVerb {
    type Error = crate::model::base::error::Error;

    /// 由判别值还原路径动词。
    ///
    /// 契约：API-026
    ///
    /// 约束：
    ///   - requires  判别值落在 1..=19
    ///   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
    ///   - 错误      InvalidPathVerb —— 判别值不在 1..=19
    fn try_from(value: u8) -> Result<Self> {
        path_inner::path_verb_try_from(value)
    }
}

/**
 * 路径动词，判别值与 JS 侧 u8 契约一致（1..=19）。
 *
 * 契约：API-026
 *
 * 约束：
 *   - requires  判别值落在 1..=19
 *   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
 *   - 错误      InvalidPathVerb —— 判别值不在 1..=19
 */
impl From<PathVerb> for u8 {
    /// 取路径动词判别值。
    ///
    /// 契约：API-026
    ///
    /// 约束：
    ///   - requires  判别值落在 1..=19
    ///   - ensures   非法判别值返回 Err；判别值与 JS u8 契约一致
    ///   - 错误      InvalidPathVerb —— 判别值不在 1..=19
    fn from(verb: PathVerb) -> u8 {
        path_inner::path_verb_to_u8(verb)
    }
}

/**
 * 矢量路径：动词序列与坐标序列。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 */
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Path {
    /// 路径动词序列
    pub verbs: Vec<PathVerb>,
    /// 路径坐标序列，拍平存放（每点 2 个分量）
    pub points: Vec<f32>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Path {
/**
 * 由动词字节与扁平坐标构造路径。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 *
 * 参数：verbs — 路径动词判别值序列
 *
 * 参数：points — 扁平坐标序列，每点 2 个分量
 */
    pub fn new(verbs: Vec<u8>, points: Vec<f32>) -> Result<Self> {
        path_inner::path_new(verbs, points)
    }

/**
 * 由路径动词与扁平坐标构造路径。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 *
 * 参数：verbs — 路径动词序列
 *
 * 参数：points — 扁平坐标序列，每点 2 个分量
 */
    pub fn from_verbs(verbs: Vec<PathVerb>, points: Vec<f32>) -> Result<Self> {
        path_inner::path_from_verbs(verbs, points)
    }

/**
 * 转为轮廓。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 */
    pub fn to_outline(&self) -> Result<Outline> {
        path_inner::path_to_outline(self)
    }

/**
 * 生成 SDF 纹理。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   非法动词或坐标不匹配返回 Err；sdf_texture 结果与现状等价
 *   - 错误      InvalidPathVerb —— 动词字节非法；InvalidParam —— 坐标数量不匹配或参数非正
 *
 * 参数：tex_size — 纹理边长，正整数
 *
 * 参数：pxrange — 距离像素范围，正整数
 */
    pub fn sdf_texture(&self, tex_size: u32, pxrange: u32) -> Result<SdfTexture> {
        path_inner::path_sdf_texture(self, tex_size, pxrange)
    }

/**
 * 路径包围盒。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   结果与现状一致
 *   - 错误      无
 */
    pub fn extents(&self) -> Aabb {
        path_inner::path_extents(self)
    }

/**
 * 反转路径方向。
 *
 * 契约：API-027
 *
 * 约束：
 *   - requires  points 长度与 verbs 所需的坐标数一致
 *   - ensures   结果原地作用于自身
 *   - 错误      无
 */
    pub fn reverse(&mut self) {
        path_inner::path_reverse(self)
    }
}

/**
 * 图形元：圆、矩形、线段、椭圆、多边形、折线。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum Shape {
    /// 圆
    Circle { cx: f32, cy: f32, r: f32 },
    /// 矩形
    Rect { x: f32, y: f32, w: f32, h: f32 },
    /// 线段
    Line { ax: f32, ay: f32, bx: f32, by: f32, step: Option<f32> },
    /// 椭圆
    Ellipse { cx: f32, cy: f32, rx: f32, ry: f32 },
    /// 多边形
    Polygon { points: Vec<f32> },
    /// 折线
    Polyline { points: Vec<f32>, is_close: bool },
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Shape {
/**
 * 图形元轮廓。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
 *   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
 */
    pub fn to_outline(&self) -> Result<Outline> {
        path_inner::shape_to_outline(self)
    }

/**
 * 图形元包围盒。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   结果与现状一致
 *   - 错误      无
 */
    pub fn extents(&self) -> Aabb {
        path_inner::shape_extents(self)
    }

/**
 * 是否为面积图元。
 *
 * 契约：API-028
 *
 * 约束：
 *   - requires  半径与宽高为非负值；多边形至少 3 点
 *   - ensures   线段类返回假，其余返回真
 *   - 错误      无
 */
    pub fn is_area(&self) -> bool {
        path_inner::shape_is_area(self)
    }
}
