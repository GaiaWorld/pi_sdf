use parry2d::math::Point;

use crate::glyphy::geometry::{arc::Arc, vector::VectorEXT};

use super::{
    geometry::{arc::ArcEndpoint, point::PointExt},
    util::{float_equals, is_zero, xor, GLYPHY_EPSILON, GLYPHY_INFINITY},
};

pub fn glyphy_outline_reverse(endpoints: &mut [ArcEndpoint]) {
    let num_endpoints = endpoints.len();

    if num_endpoints == 0 {
        return;
    }

    // Shift the d's first
    let d0 = endpoints[0].d;
    for i in 0..num_endpoints - 1 {
        endpoints[i].d = if endpoints[i + 1].d == GLYPHY_INFINITY {
            GLYPHY_INFINITY
        } else {
            -endpoints[i + 1].d
        };
    }
    endpoints[num_endpoints - 1].d = d0;

    // // Reverse
    // for (let i = 0, j = num_endpoints - 1; i < j; i++, j--) {
    //     let t = endpoints[i];
    //     endpoints[i] = endpoints[j];
    //     endpoints[j] = t;
    // }
    for (i, j) in (0..num_endpoints).zip((0..num_endpoints).rev()) {
        if !i < j {
            break;
        }
        let t = endpoints[i].clone();
        endpoints[i] = endpoints[j].clone();
        endpoints[j] = t;
    }
}

pub fn winding(endpoints: &mut [ArcEndpoint]) -> bool {
    let num_endpoints = endpoints.len();

    /*
     * Algorithm:
     *
     * - Approximate arcs with triangles passing through the mid- and end-points,
     * - Calculate the area of the contour,
     * - Return sign.
     */

    let mut area = 0.0;
    for i in 1..num_endpoints {
        let p0 = endpoints[i - 1].p;
        let p0 = Point::new(p0[0], p0[1]);
        let p1 = endpoints[i].p;
        let p1 = Point::new(p1[0], p1[1]);
        let d = endpoints[i].d;

        assert!(d != GLYPHY_INFINITY);

        area +=  p0.into_vector().sdf_cross(&p1.into_vector());
        area -= 0.5 * d * (p1 - p0).norm_squared();
    }
    return area < 0.0;
}

/*
 * Algorithm:
 *
 * - For a point on the contour, draw a halfline in a direction
 *   (eg. decreasing x) to infinity,
 * - Count how many times it crosses all other contours,
 * - Pay special attention to points falling exactly on the halfline,
 *   specifically, they count as +.5 or -.5, depending the direction
 *   of crossing.
 *
 * All this counting is extremely tricky:
 *
 * - Floating point equality cannot be relied on here,
 * - Lots of arc analysis needed,
 * - Without having a point that we know falls /inside/ the contour,
 *   there are legitimate cases that we simply cannot handle using
 *   this algorithm.  For example, imagine the following glyph shape:
 *
 *         +---------+
 *         | +-----+ |
 *         |  \   /  |
 *         |   \ /   |
 *         +----o----+
 *
 *   If the glyph is defined as two outlines, and when analysing the
 *   inner outline we happen to pick the point denoted by 'o' for
 *   analysis, there simply is no way to differentiate this case from
 *   the following case:
 *
 *         +---------+
 *         |         |
 *         |         |
 *         |         |
 *         +----o----+
 *             / \
 *            /   \
 *           +-----+
 *
 *   However, in one, the triangle should be filled in, and in the other
 *   filled out.
 *
 *   One way to work around this may be to do the analysis for all endpoints
 *   on the outline and take majority.  But even that can fail in more
 *   extreme yet legitimate cases, such as this one:
 *
 *           +--+--+
 *           | / \ |
 *           |/   \|
 *           +     +
 *           |\   /|
 *           | \ / |
 *           +--o--+
 *
 *   The only correct algorithm I can think of requires a point that falls
 *   fully inside the outline.  While we can try finding such a point (not
 *   dissimilar to the winding algorithm), it's beyond what I'm willing to
 *   implement right now.
 */
pub fn even_odd(
    c_endpoints: &[ArcEndpoint],
    endpoints: &[ArcEndpoint],
    start_index: usize,
) -> bool {
    let num_c_endpoints = c_endpoints.len();
    let num_endpoints = endpoints.len();
    let p = Point::new(c_endpoints[0].p[0], c_endpoints[0].p[1]);

    let mut count = 0.0;
    let mut p0 = Point::new(0.0, 0.0);
    for i in 0..num_endpoints {
        let endpoint = &endpoints[i];
        if endpoint.d == GLYPHY_INFINITY {
            p0 = Point::new(endpoint.p[0], endpoint.p[1]);
            continue;
        }
        let arc = Arc::new(p0, Point::new(endpoint.p[0], endpoint.p[1]), endpoint.d);
        p0 = Point::new(endpoint.p[0], endpoint.p[1]);

        /*
         * Skip our own contour
         * c_endpoints 是 endpoints 的 切片，而 start_index 是 c_endpoints 起始元素 在 endpoints 中的索引
         */
        if i >= start_index && i < start_index + num_c_endpoints {
            continue;
        }

        /* End-point y's compared to the ref point; lt, eq, or gt */
        let s0 = categorize(arc.p0.y, p.y);
        let s1 = categorize(arc.p1.y, p.y);

        if is_zero(arc.d, None) {
            /* Line */

            if s0 == 0 || s1 == 0 {
                /*
                 * Add +.5 / -.5 for each endpoint on the halfline, depending on
                 * crossing direction.
                 */
                let t = arc.tangents();
                if s0 == 0 && arc.p0.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.0.y, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.1.y, 0.0) as f32;
                }
                continue;
            }

            if s0 == s1 {
                continue; // Segment fully above or below the halfline
            }

            // Find x pos that the line segment would intersect the half-line.
            let x = arc.p0.x + (arc.p1.x - arc.p0.x) * ((p.y - arc.p0.y) / (arc.p1.y - arc.p0.y));

            if x >= p.x - GLYPHY_EPSILON {
                continue; // Does not intersect halfline
            }

            count += 1.0; // Add one for full crossing
            continue;
        } else {
            /* Arc */

            if s0 == 0 || s1 == 0 {
                /*
                 * Add +.5 / -.5 for each endpoint on the halfline, depending on
                 * crossing direction.
                 */
                let mut t = arc.tangents();

                /* Arc-specific logic:
                 * If the tangent has y==0, use the other endpoint's
                 * y value to decide which way the arc will be heading.
                 */
                if is_zero(t.0.y, None) {
                    t.0.y = categorize(arc.p1.y, p.y) as f32;
                }
                if is_zero(t.1.y, None) {
                    t.1.y = -categorize(arc.p0.y, p.y) as f32;
                }

                if s0 == 0 && arc.p0.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.0.y, 0.0) as f32;
                }
                if s1 == 0 && arc.p1.x < p.x + GLYPHY_EPSILON {
                    count += 0.5 * categorize(t.1.y, 0.0) as f32;
                }
            }

            let c = arc.center();
            let r = arc.radius();
            if c.x - r >= p.x {
                continue; // No chance
            }
            /* Solve for arc crossing line with y = p.y */
            let y = p.y - c.y;
            let x2 = r * r - y * y;
            if x2 <= GLYPHY_EPSILON {
                continue; // Negative delta, no crossing
            }
            let dx = x2.sqrt();
            /* There's two candidate points on the arc with the same y as the
             * ref point. */
            let pp = [Point::new(c.x - dx, p.y), Point::new(c.x + dx, p.y)];

            for i in 0..pp.len() {
                /* Make sure we don't double-count endpoints that fall on the
                 * halfline as we already accounted for those above */
                if !pp[i].equals(&arc.p0)
                    && !pp[i].equals(&arc.p1)
                    && pp[i].x < p.x - GLYPHY_EPSILON
                    && arc.wedge_contains_point(&pp[i])
                {
                    count += 1.0; // Add one for full crossing
                }
            }
        }
    }

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
    /*
     * Algorithm:
     *
     * - Find the winding direction and even-odd number,
     * - If the two disagree, reverse the contour, inplace.
     */

    let num_endpoints = endpoints.len();
    if num_endpoints == 0 {
        return false;
    }

    if num_endpoints < 3 {
        log::warn!("Don't expect this");
        return false; // Need at least two arcs
    }
    
    if !(float_equals(endpoints[0].p[0], endpoints[num_endpoints - 1].p[0], None) && float_equals(endpoints[0].p[1], endpoints[num_endpoints - 1].p[1], None)) {
        log::warn!("Don't expect this");
        return false; // Need a closed contour
    }

    let mut r = xor(inverse, winding(endpoints));
    r = xor(r, even_odd(endpoints, all_endpoints, start_index));

    if r {
        glyphy_outline_reverse(endpoints);
        return true;
    }

    return false;
}

/**
 * 用奇偶规则计算轮廓的winding number
 * @returns 如果修改了轮廓，则返回true
 */
pub fn glyphy_outline_winding_from_even_odd(
    endpoints: &Vec<ArcEndpoint>,
    inverse: bool,
) -> bool {
    /*
     * Algorithm:
     *
     * - Process one contour（闭合曲线）at a time.
     */

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

// const is_zero = (v: number) => {
//     return Math.abs(v) < GLYPHY_EPSILON;
// }
