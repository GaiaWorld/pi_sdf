use parry2d::math::Point;

use crate::glyphy::geometry::{arc::Arc, vector::VectorEXT};

use super::{
    geometry::{arc::ArcEndpoint, point::PointExt},
    util::{float_equals, is_zero, xor, GLYPHY_EPSILON, GLYPHY_INFINITY},
};
/// 反转glyph轮廓的端点序列，使路径方向相反
/// 该函数修改了传入的`endpoints`切片，使其路径方向反转
/// # 参数
/// * `endpoints` - 要反转的端点序列
pub fn glyphy_outline_reverse(endpoints: &mut [ArcEndpoint]) {
    let num_endpoints = endpoints.len();

    if num_endpoints == 0 {
        return;
    }

    // 第一部分: 处理d的值，将其反转
    let d0 = endpoints[0].d;
    for i in 0..num_endpoints - 1 {
        endpoints[i].d = if endpoints[i + 1].d == GLYPHY_INFINITY {
            GLYPHY_INFINITY
        } else {
            -endpoints[i + 1].d
        };
    }
    endpoints[num_endpoints - 1].d = d0;

    // 第二部分: 反转整个端点序列
    for (i, j) in (0..num_endpoints).zip((0..num_endpoints).rev()) {
        if !i < j {
            break;
        }
        let t = endpoints[i].clone();
        endpoints[i] = endpoints[j].clone();
        endpoints[j] = t;
    }
}

/// 计算端点序列的环绕方向
/// 通过计算路径的有向面积来确定环绕方向，如果面积小于零，则为顺时针方向
/// # 参数
/// * `endpoints` - 组成路径的端点序列
/// # 返回值
/// * `bool` - 如果路径是顺时针方向，则返回true，否则返回false
pub fn winding(endpoints: &mut [ArcEndpoint]) -> bool {
    let num_endpoints = endpoints.len();

    let mut area = 0.0;
    for i in 1..num_endpoints {
        let p0 = endpoints[i - 1].p;
        let p0 = Point::new(p0[0], p0[1]);
        let p1 = endpoints[i].p;
        let p1 = Point::new(p1[0], p1[1]);
        let d = endpoints[i].d;

        assert!(d != GLYPHY_INFINITY);

        // 使用向量叉积计算有向面积的一部分，并减去由d参数引入的修正项
        area +=  p0.into_vector().sdf_cross(&p1.into_vector());
        area -= 0.5 * d * (p1 - p0).norm_squared();
    }
    return area < 0.0;
}

pub fn even_odd(
    c_endpoints: &[ArcEndpoint],
    endpoints: &[ArcEndpoint],
    start_index: usize,
) -> bool {
    // 计算一个点的 even-odd 交叉次数
    let num_c_endpoints = c_endpoints.len();
    let num_endpoints = endpoints.len();
    // 将当前点设置为第一个控制点
    let p = Point::new(c_endpoints[0].p[0], c_endpoints[0].p[1]);

    let mut count = 0.0;
    let mut p0 = Point::new(0.0, 0.0);  // 起始点
    for i in 0..num_endpoints {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            // 无穷大的d表示直线段的起点
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        // 创建一个弧对象
        let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);  // 更新起始点为当前终点

        if i >= start_index && i < start_index + num_c_endpoints {
            // 当前索引在忽略的范围内，直接继续
            continue;
        }

        // 判断点p是否在当前弧的y坐标范围内
        let s0 = categorize(arc.p0.y, p.y);
        let s1 = categorize(arc.p1.y, p.y);

        if is_zero(arc.d, None) {  // 直线段
            // 如果弧的两个端点y坐标与p相同，则可能交叉
            if s0 == 0 || s1 == 0 {
                // 计算直线段的切线斜率
                let t = arc.tangents();
                // 计算交叉次数
                if s0 == 0 && arc.p0.x < p.x + GLYPHY_EPSILON {
                    // 起始点在p的左边，且斜率为正
                    count += 0.5 * categorize(t.0.1, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + GLYPHY_EPSILON {
                    // 终点在p的左边，且斜率为正
                    count += 0.5 * categorize(t.1.1, 0.0) as f32;
                }
                continue;
            }

            // 如果弧的两个端点y坐标在p的同一侧，不会交叉
            if s0 == s1 {
                continue;
            }

            // 计算直线段与p所在水平线的交点x坐标
            let x = arc.p0.x + (arc.p1.x - arc.p0.x) * ((p.y - arc.p0.y) / (arc.p1.y - arc.p0.y));

            // 如果交点x在p的右侧，不计入交叉次数
            if x >= p.x - GLYPHY_EPSILON {
                continue;
            }

            // 计入一次交叉
            count += 1.0;

            continue;
        } else {  // 圆弧
            // 如果弧的两个端点y坐标与p相同，则可能交叉
            if s0 == 0 || s1 == 0 {
                let mut t = arc.tangents();
                // 调整切线斜率以确定交叉方向
                if is_zero(t.0.1, None) {
                    t.0.1 = s1 as f32;
                }
                if is_zero(t.1.1, None) {
                    t.1.1 = -s0 as f32;
                }

                // 计算交叉次数
                if s0 == 0 && arc.p0.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.0.1, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.1.1, 0.0) as f32;
                }
            }

            // 计算圆弧的中心和半径，判断是否在p的水平线交汇
            let c = arc.center();
            let r = arc.radius();
            if c.x - r >= p.x {
                continue; // 没有交汇的可能
            }

            // 计算与p所在水平线的交点
            let y = p.y - c.y;
            let x2 = r * r - y * y;
            if x2 <= GLYPHY_EPSILON {
                continue; // 无实数解，没有交点
            }
            let dx = x2.sqrt();

            let pp = [  // 水平线与圆的交点
                Point::new(c.x - dx, p.y),
                Point::new(c.x + dx, p.y)
            ];

            // 遍历交点，判断是否在弧段内部且在左侧
            for i in 0..pp.len() {
                if !pp[i].equals(&arc.p0) && !pp[i].equals(&arc.p1)
                    && pp[i].x < p.x - GLYPHY_EPSILON
                && arc.wedge_contains_point(&pp[i]) {
                    // 每次完全交叉计入一次
                    count += 1.0;
                }
            }
        }
    }

    // 判断交叉次数的奇偶性，决定是否内部
    return (count.floor() as i32 & 1) == 0;
}
/**
 * 计算曲线的winding number
 * @note endpoints 是 all_endpoints 的 切片
 * @param start_index endpoints的起始元素在all_endpoints中的索引
 * @returns 如果修改了轮廓，则返回true
 */
pub fn process_contour(
    endpoints: &mut [ArcEndpoint],
    all_endpoints: &[ArcEndpoint],
    inverse: bool,
    start_index: usize,
) -> bool {
    // 判断端点数组是否为空，如果为空则返回false
    let num_endpoints = endpoints.len();
    if num_endpoints == 0 {
        return false;
    }

    // 如果端点数少于3，输出警告并返回false
    if num_endpoints < 3 {
        log::warn!("Don't expect this");
        return false;
    }

    // 检查首尾两个端点坐标是否一致，不一致则输出警告并返回false
    if !(float_equals(endpoints[0].p[0], endpoints[num_endpoints - 1].p[0], None) && float_equals(endpoints[0].p[1], endpoints[num_endpoints - 1].p[1], None)) {
        log::warn!("Don't expect this");
        return false;
    }

    // 调用winding函数计算正负面积，得到r的初始值
    // 再通过even_odd函数计算奇偶性，结合inverse参数，异或运算得到最终的r
    let mut r = xor(inverse, winding(endpoints));
    r = xor(r, even_odd(endpoints, all_endpoints, start_index));

    // 如果r为true，则反转端点数组并返回true
    if r {
        glyphy_outline_reverse(endpoints);
        return true;
    }

    return false;
}

 /// 用奇偶规则计算轮廓的winding number
 /// 如果修改了轮廓，则返回true
pub fn glyphy_outline_winding_from_even_odd(
    endpoints: &Vec<ArcEndpoint>,
    inverse: bool,
) -> bool {
    let mut start = 0;
    let mut ret = false;
    let num_endpoints = endpoints.len();
    for i in 1..num_endpoints {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            ret = ret
                || process_contour(
                    &mut endpoints[start..i].to_vec(),
                    &endpoints,
                    inverse,
                    start,
                );
            start = i;
        }
    }
    ret = ret || process_contour(&mut endpoints[start..].to_vec(), &endpoints, inverse, start);

    return ret;
}

pub fn categorize(v: f32, r: f32) -> i32 {
    return if v < r - GLYPHY_EPSILON {
        -1
    } else if v > r + GLYPHY_EPSILON {
        1
    } else {
        0
    };
}

