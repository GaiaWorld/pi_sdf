/**
 * 路径动词与图元
 *
 * 设计：design/pi_sdf2/00-index.md
 */

use crate::core::base::error::Result;
use crate::core::outline::contour::Outline;
use crate::core::geom::aabb::Aabb;
use crate::api::path::primitives_inner;

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
pub enum PathVerb {
    /// 绝对移动到
    MoveTo = 1,
    /// 绝对直线到
    LineTo = 2,
    /// 绝对水平线到
    HorizontalLineTo = 3,
    /// 绝对垂直线到
    VerticalLineTo = 4,
    /// 绝对二次曲线到
    QuadraticCurveTo = 5,
    /// 绝对平滑二次曲线到
    SmoothQuadraticCurveTo = 6,
    /// 绝对三次曲线到
    CubicCurveTo = 7,
    /// 绝对平滑三次曲线到
    SmoothCubicCurveTo = 8,
    /// 绝对椭圆弧到
    EllipticalArcTo = 9,
    /// 闭合路径
    ClosePath = 10,
    /// 相对移动到
    RelativeMoveTo = 11,
    /// 相对直线到
    RelativeLineTo = 12,
    /// 相对水平线到
    RelativeHorizontalLineTo = 13,
    /// 相对垂直线到
    RelativeVerticalLineTo = 14,
    /// 相对二次曲线到
    RelativeQuadraticCurveTo = 15,
    /// 相对平滑二次曲线到
    RelativeSmoothQuadraticCurveTo = 16,
    /// 相对三次曲线到
    RelativeCubicCurveTo = 17,
    /// 相对平滑三次曲线到
    RelativeSmoothCubicCurveTo = 18,
    /// 相对椭圆弧到
    RelativeEllipticalArcTo = 19,
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
    type Error = crate::core::base::error::Error;

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
    fn try_from(value: u8) -> Result<Self> {
        primitives_inner::path_verb_try_from(value)
    }
}

/**
 * 取路径动词判别值。
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
    ///
    /// 参数：verb — 路径动词
    fn from(verb: PathVerb) -> u8 {
        primitives_inner::path_verb_to_u8(verb)
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
    Polygon { points: Vec<[f32; 2]> },
    /// 折线
    Polyline { points: Vec<[f32; 2]>, is_close: bool },
}

impl Shape {
    /// 图形元轮廓。
    ///
    /// 契约：API-028
    ///
    /// 约束：
    ///   - requires  半径与宽高为非负值；多边形至少 3 点
    ///   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
    ///   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
    pub fn to_outline(&self) -> Result<Outline> {
        primitives_inner::shape_to_outline(self)
    }

    /// 图形元包围盒。
    ///
    /// 契约：API-028
    ///
    /// 约束：
    ///   - requires  半径与宽高为非负值；多边形至少 3 点
    ///   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
    ///   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
    pub fn extents(&self) -> Aabb {
        primitives_inner::shape_extents(self)
    }

    /// 是否为面积图元。
    ///
    /// 契约：API-028
    ///
    /// 约束：
    ///   - requires  半径与宽高为非负值；多边形至少 3 点
    ///   - ensures   六种图元轮廓与现状等价；非法尺寸返回 Err
    ///   - 错误      InvalidParam —— 半径或宽高为负，或点数不足
    pub fn is_area(&self) -> bool {
        primitives_inner::shape_is_area(self)
    }
}
