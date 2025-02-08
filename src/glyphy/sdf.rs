use crate::glyphy::geometry::arc::Arc;
use crate::glyphy::geometry::arc::ArcEndpoint;
use crate::glyphy::util::GLYPHY_EPSILON;
use crate::glyphy::util::GLYPHY_INFINITY;
use crate::Point;
/**
 * SDF 算法
 *
 * 点 p 到 所有圆弧的 sdf 的 最小值
 *
 * 返回：[sdf, 影响sdf的圆弧起点在-endpoints-中的索引]
 */
pub fn glyphy_sdf_from_arc_list(endpoints: &Vec<ArcEndpoint>, p: Point) -> (f32, usize) {
    let num_endpoints = endpoints.len();

    let c = p.clone();
    let mut p0 = Point::new(1.0, 1.0);
    let mut closest_arc = Arc::new(p0, p0, 0.0);

    let mut side = 0;
    let mut min_dist = GLYPHY_INFINITY;

    // 影响 min_dist 的 圆弧起点的 索引
    let mut last_index = 0;

    for i in 0..num_endpoints {
        let endpoint = &endpoints[i];

        if endpoint.d == GLYPHY_INFINITY {
            // 无穷代表 Move 语义
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }

        // 当 d = 0 时候，代表线段
        let arc = Arc::new(p0.clone(), Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);

        if arc.wedge_contains_point(&c) {
            // 在 扇形夹角范围内

            /* TODO This distance has the wrong sign.  Fix */
            let sdist = arc.distance_to_point(c);

            let udist = sdist.abs() * (1.0 - GLYPHY_EPSILON);

            if udist <= min_dist {
                min_dist = udist;

                last_index = i - 1;
                // if last_index < 0 {
                //     panic!("1 last_index < 0");
                // }
                side = if sdist >= 0.0 { -1 } else { 1 };
            }
        } else {
            // 在外面

            // 取 距离 点c 最近的 圆弧端点 的 距离
            let la = (arc.p0 - c).norm();
            let lb = (arc.p1 - c).norm();
            let udist = if la < lb { la } else { lb };

            if udist < min_dist {
                // 比 原来的 小，则 更新 此距离
                min_dist = udist;
                last_index = i - 1;

                // 但 此时 符号 未知
                side = 0; /* unsure */
                closest_arc = arc;
            } else if side == 0 && udist == min_dist {
                // 如果 更换了 端点 之后，距离和原来相同，但符号未知
                // 则：拿此次 的 符号 作为 原来的符号。

                // If this new distance is the same as the current minimum,
                // compare extended distances.  Take the sign from the arc
                // with larger extended distance.

                let old_ext_dist = closest_arc.extended_dist(&c);

                // 新的 距离 是 arc 到 c 的 扩展距离
                let new_ext_dist = arc.extended_dist(&c);

                let ext_dist = if new_ext_dist.abs() <= old_ext_dist.abs() {
                    old_ext_dist
                } else {
                    new_ext_dist
                };

                /* For emboldening and stuff: */
                // min_dist = fabs (ext_dist);
                side = if ext_dist >= 0.0 { 1 } else { -1 };
            }
        }

        p0 = Point::new(endpoint.p[0], endpoint.p[1]);
    }

    if side == 0 {
        // Technically speaking this should not happen, but it does.  So try to fix it.
        let ext_dist = closest_arc.extended_dist(&c);
        side = if ext_dist >= 0.0 { 1 } else { -1 };
    }

    return (side as f32 * min_dist, last_index);
}

/**
 * SDF 算法
 *
 * 点 p 到 所有圆弧的 sdf 的 最小值
 *
 * 返回：[sdf, 影响sdf的圆弧起点在-endpoints-中的索引]
 */
pub fn glyphy_sdf_from_arc_list2(arcs: &Vec<Arc>, p: Point) -> (f32, usize) {
    // let num_endpoints = endpoints.len();

    let c = p.clone();
    let p0 = Point::new(1.0, 1.0);
    let mut closest_arc = &Arc::new(p0, p0, 0.0);

    let mut side = 0;
    let mut min_dist = GLYPHY_INFINITY;

    // 影响 min_dist 的 圆弧起点的 索引
    let mut last_index = 0;

    for i in 0..arcs.len() {
        // let endpoint = &endpoints[i];

        // if endpoint.d == GLYPHY_INFINITY {
        //     // 无穷代表 Move 语义
        //     p0 = endpoint.p.clone();
        //     continue;
        // }

        // 当 d = 0 时候，代表线段
        let arc = &arcs[i];

        if arc.wedge_contains_point(&c) {
            // 在 扇形夹角范围内

            /* TODO This distance has the wrong sign.  Fix */
            let sdist = arc.distance_to_point(c);

            let udist = sdist.abs() * (1.0 - GLYPHY_EPSILON);

            if udist <= min_dist {
                min_dist = udist;

                last_index = i;
                // if last_index < 0 {
                //     panic!("1 last_index < 0");
                // }
                side = if sdist >= 0.0 { -1 } else { 1 };
            }
        } else {
            // 在外面

            // 取 距离 点c 最近的 圆弧端点 的 距离
            let la = (arc.p0 - c).norm();
            let lb = (arc.p1 - c).norm();
            let udist = if la < lb { la } else { lb };

            if udist < min_dist {
                // 比 原来的 小，则 更新 此距离
                min_dist = udist;
                last_index = i;
                // if last_index < 0 {
                //     panic!("2 last_index < 0");
                // }

                // 但 此时 符号 未知
                side = 0; /* unsure */
                closest_arc = &arc;
            } else if side == 0 && udist == min_dist {
                // 如果 更换了 端点 之后，距离和原来相同，但符号未知
                // 则：拿此次 的 符号 作为 原来的符号。

                // If this new distance is the same as the current minimum,
                // compare extended distances.  Take the sign from the arc
                // with larger extended distance.

                let old_ext_dist = closest_arc.extended_dist(&c);

                // 新的 距离 是 arc 到 c 的 扩展距离
                let new_ext_dist = arc.extended_dist(&c);

                let ext_dist = if new_ext_dist.abs() <= old_ext_dist.abs() {
                    old_ext_dist
                } else {
                    new_ext_dist
                };

                /* For emboldening and stuff: */
                // min_dist = fabs (ext_dist);
                side = if ext_dist >= 0.0 { 1 } else { -1 };
            }
        }

        // p0 = arc.p.clone();
    }

    if side == 0 {
        // Technically speaking this should not happen, but it does.  So try to fix it.
        let ext_dist = closest_arc.extended_dist(&c);
        side = if ext_dist >= 0.0 { 1 } else { -1 };
    }

    return (side as f32 * min_dist, last_index);
}

pub fn glyphy_sdf_from_arc_list3(arcs: &Vec<usize>, p: Point, global_arcs: &Vec<Arc>) -> (f32, usize) {
    // let num_endpoints = endpoints.len();

    let c = p.clone();
    let p0 = Point::new(1.0, 1.0);
    let mut closest_arc = &Arc::new(p0, p0, 0.0);

    let mut side = 0;
    let mut min_dist = GLYPHY_INFINITY;

    // 影响 min_dist 的 圆弧起点的 索引
    let mut last_index = 0;

    for i in 0..arcs.len() {
        // let endpoint = &endpoints[i];

        // if endpoint.d == GLYPHY_INFINITY {
        //     // 无穷代表 Move 语义
        //     p0 = endpoint.p.clone();
        //     continue;
        // }

        // 当 d = 0 时候，代表线段
        let arc = &global_arcs[arcs[i]];

        if arc.wedge_contains_point(&c) {
            // 在 扇形夹角范围内

            /* TODO This distance has the wrong sign.  Fix */
            let sdist = arc.distance_to_point(c);
            // println!("sdist: {}", sdist);

            let udist = sdist.abs() * (1.0 - GLYPHY_EPSILON);

            if udist <= min_dist {
                min_dist = udist;

                last_index = i;
                // if last_index < 0 {
                //     panic!("1 last_index < 0");
                // }
                side = if sdist >= 0.0 { -1 } else { 1 };
            }
        } else {
            // 在外面

            // 取 距离 点c 最近的 圆弧端点 的 距离
            let la = (arc.p0 - c).norm();
            let lb = (arc.p1 - c).norm();
            let udist = if la < lb { la } else { lb };
            // if (p - Point::new(293.82404, 216.00003)).norm_squared() < 0.01 ||(p - Point::new(341.82404, 216.00003)).norm_squared() < 0.01{
            //     log::debug!("udist: {}, min_dist: {:?}", udist, min_dist);
            // }

            if udist < min_dist {
                // 比 原来的 小，则 更新 此距离
                min_dist = udist;
                last_index = i;
                // if last_index < 0 {
                //     panic!("2 last_index < 0");
                // }

                // 但 此时 符号 未知
                side = 0; /* unsure */
                closest_arc = &arc;
            } else if side == 0 && udist == min_dist {
                // 如果 更换了 端点 之后，距离和原来相同，但符号未知
                // 则：拿此次 的 符号 作为 原来的符号。

                // If this new distance is the same as the current minimum,
                // compare extended distances.  Take the sign from the arc
                // with larger extended distance.

                let old_ext_dist = closest_arc.extended_dist(&c);

                // 新的 距离 是 arc 到 c 的 扩展距离
                let new_ext_dist = arc.extended_dist(&c);

                let ext_dist = if new_ext_dist.abs() <= old_ext_dist.abs() {
                    old_ext_dist
                } else {
                    new_ext_dist
                };

                /* For emboldening and stuff: */
                // min_dist = fabs (ext_dist);
                side = if ext_dist >= 0.0 { 1 } else { -1 };
            }
        }

        // p0 = arc.p.clone();
    }

    // if (p - Point::new(293.82404, 216.00003)).norm_squared() < 0.01 ||(p - Point::new(341.82404, 216.00003)).norm_squared() < 0.01{
    //     log::debug!("side: {}, closest_arc: {:?}", side, closest_arc);
    //     for i in 0..arcs.len() {
    //         let arc = &global_arcs[arcs[i]];

    //       log::debug!("arc.wedge_contains_point(&c): {}", arc.wedge_contains_point(&c)); 
    //     }
    // }
    if side == 0 {
        // Technically speaking this should not happen, but it does.  So try to fix it.
        let ext_dist = closest_arc.extended_dist(&c);
        side = if ext_dist >= 0.0 { 1 } else { -1 };
    }

    return (side as f32 * min_dist, last_index);
}

#[test]
fn test(){
    let arcs = vec![
        Arc::new(Point::new(87.00962, 75.5), Point::new(85.0, 83.00001), -0.13385826),
        Arc::new(Point::new(85.0, 83.00001), Point::new(85.0, 85.0), 0.0),
        
    ];
    let sdf = glyphy_sdf_from_arc_list3(&vec![0,1], Point::new(84.999985, 82.0), &arcs);
    log::debug!("sdf: {}", sdf.0);
}