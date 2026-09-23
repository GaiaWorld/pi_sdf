/**
 * 轮廓与子轮廓、绕向判定
 *
 * 设计：design/pi_sdf2/00-index.md
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::geom::ArcEndpoint;
use crate::model::geom::primitive::Point;
use crate::model::geom::curve::Arc;
use crate::model::geom::aabb::Aabb;
use crate::model::outline::contour_inner;

/**
 * 子轮廓：一条闭合或开放的弧序列。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Contour {
    /// 构成子轮廓的弧序列
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub arcs: Vec<Arc>,
    /// 是否闭合
    pub is_closed: bool,
}

/**
 * 轮廓：子轮廓的集合。
 *
 * 契约：API-010
 *
 * 约束：
 *   - requires  Contour 闭合时首尾弧端点相接
 *   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
 *   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
 */
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Outline {
    /// 子轮廓集合
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub contours: Vec<Contour>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Contour {
    /// 由弧序列与闭合标志构造子轮廓。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    ///
    /// 参数：arcs — 弧序列
    /// 参数：is_closed — 是否闭合
    pub fn new(arcs: Vec<Arc>, is_closed: bool) -> Self {
        contour_inner::contour_new(arcs, is_closed)
    }

    /// 由弧端点序列构造子轮廓。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    ///
    /// 参数：eps — 弧端点序列
    pub fn from_endpoints(eps: Vec<ArcEndpoint>) -> Self {
        contour_inner::contour_from_endpoints(eps)
    }

    /// 导出弧端点序列。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn endpoints(&self) -> Vec<ArcEndpoint> {
        contour_inner::contour_endpoints(self)
    }

    /// 是否为顺时针绕向。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn is_clockwise(&self) -> bool {
        contour_inner::contour_is_clockwise(self)
    }

    /// 反转绕向（原地）。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn reverse(&mut self) {
        contour_inner::contour_reverse(self)
    }

    /// 子轮廓的包围盒。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn extents(&self) -> Aabb {
        contour_inner::contour_extents(self)
    }

    /// 弧序列（wasm 投影：字段被 skip，改由 getter 暴露）。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  无
    ///   - ensures   返回内部数据的副本，与字段内容一致
    ///   - 错误      无
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(getter)]
    pub fn arcs(&self) -> Vec<Arc> {
        self.arcs.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Outline {
    /// 由子轮廓集合构造轮廓。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    ///
    /// 参数：contours — 子轮廓集合
    pub fn new(contours: Vec<Contour>) -> Self {
        contour_inner::outline_new(contours)
    }

    /// 轮廓的包围盒。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn extents(&self) -> Aabb {
        contour_inner::outline_extents(self)
    }

    /// 反转全部子轮廓的绕向（原地）。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn reverse(&mut self) {
        contour_inner::outline_reverse(self)
    }

    /// 整体是否为顺时针绕向。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  Contour 闭合时首尾弧端点相接
    ///   - ensures   reverse 结果原地作用于自身；from_endpoints 与 endpoints 往返一致
    ///   - 错误      Geometry —— 端点序列首尾不接时返回未闭合轮廓
    pub fn is_clockwise(&self) -> bool {
        contour_inner::outline_is_clockwise(self)
    }

    /// 子轮廓集合（wasm 投影：字段被 skip，改由 getter 暴露）。
    ///
    /// 契约：API-010
    ///
    /// 约束：
    ///   - requires  无
    ///   - ensures   返回内部数据的副本，与字段内容一致
    ///   - 错误      无
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(getter)]
    pub fn contours(&self) -> Vec<Contour> {
        self.contours.clone()
    }
}

/**
 * 按需求决定是否反转子轮廓绕向。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  contour 至少含一条弧
 *   - ensures   需要反转时真正改写传入对象
 *   - 错误      无
 *
 * 参数：contour — 待判定的子轮廓，原地改写
 * 参数：inverse — 是否需要反转
 */
pub fn contour_winding(contour: &mut Contour, inverse: bool) {
    contour_inner::contour_winding(contour, inverse)
}

/**
 * 按需求决定是否反转整个轮廓的绕向。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  outline 至少含一条子轮廓
 *   - ensures   需要反转时真正改写传入对象
 *   - 错误      无
 *
 * 参数：outline — 待判定的轮廓，原地改写
 * 参数：inverse — 是否需要反转
 */
pub fn outline_winding(outline: &mut Outline, inverse: bool) {
    contour_inner::outline_winding(outline, inverse)
}

/**
 * 奇偶规则内外判定。
 *
 * 契约：API-011
 *
 * 约束：
 *   - requires  contour 至少含一条弧
 *   - ensures   对闭合子轮廓给出内外判定
 *   - 错误      无
 *
 * 参数：contour — 子轮廓
 * 参数：p — 目标点
 */
pub fn even_odd(contour: &Contour, p: Point) -> bool {
    contour_inner::even_odd(contour, p)
}
