use crate::{
    glyphy::{
        blob::{glyphy_arc_list_encode_blob2, BlobArc},
        geometry::{
            aabb::AabbEXT,
            arc::{Arc, ArcEndpoint},
            point::PointExt,
            vector::VectorEXT,
        },
        outline::glyphy_outline_winding_from_even_odd,
        util::GLYPHY_INFINITY,
        vertex::GlyphInfo,
    },
    utils::{FontFace, GlyphVisitor},
};
use parry2d::{bounding_volume::Aabb, math::Point};

pub static MIN_FONT_SIZE: f32 = 10.0;

pub static TOLERANCE: f32 = 10.0 / 1024.;

pub static ENLIGHTEN_MAX: f32 = 0.0001; /* Per EM */

pub static EMBOLDEN_MAX: f32 = 0.0001; /* Per EM */

// 取 char对应的 arc
// 实现 encode_ft_glyph
//
// #[wasm_bindgen]
pub fn get_char_arc(
    gi: &mut GlyphInfo,
    ft_face: &mut FontFace,
    char: char,
    tolerance_per_em: Option<f32>,
) -> BlobArc {
    let tolerance_per_em = if let Some(v) = tolerance_per_em {
        v
    } else {
        TOLERANCE
    };

    let upem = ft_face.font.head_table().unwrap().unwrap().units_per_em as f32;

    // let upem = font.unitsPerEm;
    let tolerance = upem * tolerance_per_em; /* in font design units */
    let faraway = upem / (MIN_FONT_SIZE * 2.0f32.sqrt());
    let enlighten_max = upem * ENLIGHTEN_MAX;
    let embolden_max = upem * EMBOLDEN_MAX;
    let begin = std::time::Instant::now();

    let mut sink = GlyphVisitor::new(1.0);
    sink.accumulate.tolerance = tolerance;

    ft_face.to_outline(char, &mut sink);

    let endpoints = &mut sink.accumulate.result;
    println!("get_endpoints: {:?}", begin.elapsed());

    println!("endpoints len: {:?}", endpoints);
    println!("ArcEndpoint size: {}", std::mem::size_of::<ArcEndpoint>());
    // 单位：Per EM
    //值越大，划分的单元格 越多，需要的纹理空间 就越大
    //值越小，划分的单元格 越少，单个格子的圆弧数 有可能 越多
    // 一般 字体越复杂，需要越大的数字

    // const GRID_SIZE = 30; /* Per EM */
    // let grid_size = GRID_SIZE;

    let mut grid_size = (endpoints.len() as f32 / 4 as f32).ceil(); /* Per EM */
    grid_size = if grid_size < 20.0 { 20.0 } else { grid_size };

    let unit_size = upem / grid_size;

    log::warn!("####################### grid_size = ${grid_size}, unit_size = ${unit_size}");

    if endpoints.len() > 0 {
        // 用奇偶规则，计算 每个圆弧的 环绕数
        let begin = std::time::Instant::now();
        glyphy_outline_winding_from_even_odd(endpoints, false);
        println!("计算 每个圆弧的 环绕数: {:?}", begin.elapsed());
    }

    // console.log("")
    // console.log("============== 03. 应用奇偶规则后的结果：");
    // let s = []
    // for (let r of endpoints) {
    //     s.push(`    { x: ${r.p.x}, y: ${r.p.y}, d: ${r.d} }`);
    // }
    // console.log(s.join(",\n"));
    // console.log("");

    let mut extents = Aabb::new(
        Point::new(f32::INFINITY, f32::INFINITY),
        Point::new(f32::INFINITY, f32::INFINITY),
    );

    // 将 指令 编码
    // let begin = std::time::Instant::now();
    println!("get_endpoints: {:?}", begin.elapsed());
    let arcs = glyphy_arc_list_encode_blob2(
        &endpoints,
        faraway,
        unit_size,
        enlighten_max,
        embolden_max,
        &mut extents,
    );
    // println!("指令 编码: {:?}", begin.elapsed());

    extents.scale(1.0 / upem, 1.0 / upem);

    gi.nominal_w = arcs.width_cells;
    gi.nominal_h = arcs.height_cells;

    gi.extents.set(&extents);

    // arcs.tex_data
    return arcs;
}

// 取 char对应的 arc
// 实现 encode_ft_glyph
//

pub fn to_arc_cmds(endpoints: &Vec<ArcEndpoint>) -> (Vec<Vec<String>>, Vec<[f32; 2]>) {
    let mut _cmd = vec![];
    let mut cmd_array = vec![];
    let mut current_point = None;
    let mut pts = vec![];
    for ep in endpoints {
        pts.push([ep.p.x, ep.p.y]);

        if ep.d == GLYPHY_INFINITY {
            if current_point.is_none() || !ep.p.equals(current_point.as_ref().unwrap()) {
                if _cmd.len() > 0 {
                    cmd_array.push(_cmd);
                    _cmd = vec![];
                }
                _cmd.push(format!(" M ${}, ${}", ep.p.x, ep.p.y));
                current_point = Some(ep.p);
            }
        } else if ep.d == 0.0 {
            assert!(current_point.is_some());
            if current_point.is_some() && !ep.p.equals(current_point.as_ref().unwrap()) {
                _cmd.push(format!(" L {}, {}", ep.p.x, ep.p.y));
                current_point = Some(ep.p);
            }
        } else {
            assert!(current_point.is_some());
            let mut _current_point = current_point.as_ref().unwrap();
            if !ep.p.equals(_current_point) {
                let arc = Arc::new(_current_point.clone(), ep.p, ep.d);
                let center = arc.center();
                let radius = arc.radius();
                let start_v = _current_point - center;
                let start_angle = start_v.sdf_angle();

                let end_v = ep.p - (center);
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
