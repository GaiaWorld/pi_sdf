//! TASK-08 烘焙：模型侧单位弧仅经对外冻结接口验证（CASE-036）。
//!
//! core/bake 的测试因 core 为 pub(crate) 而内联在对应 _inner 文件中；
//! 本文件只覆盖对外可见的 model::raster::texture 数据不变量。

use pi_sdf2::model::{ArcEndpoint, UnitArc};

fn endpoint() -> ArcEndpoint {
    ArcEndpoint {
        px: 0.0,
        py: 0.0,
        d: 0.0,
        tag: None,
    }
}

/// CASE-036：单位弧的距离区间有序（sdf_min ≤ sdf_max）。
#[test]
fn case_036_unit_arc_interval_ordered() {
    // 严格有序。
    let ordered = UnitArc {
        endpoints: vec![endpoint()],
        sdf_min: -2.5,
        sdf_max: 3.5,
        show: true,
    };
    assert!(ordered.is_ordered(), "sdf_min < sdf_max 应为有序");

    // 边界：两端相等仍视为有序。
    let equal = UnitArc {
        endpoints: vec![endpoint()],
        sdf_min: 1.0,
        sdf_max: 1.0,
        show: false,
    };
    assert!(equal.is_ordered(), "sdf_min == sdf_max 应为有序");

    // 反序。
    let reversed = UnitArc {
        endpoints: vec![endpoint()],
        sdf_min: 3.5,
        sdf_max: -2.5,
        show: true,
    };
    assert!(!reversed.is_ordered(), "sdf_min > sdf_max 应无序");

    // 非有限：NaN 不可比较，判为无序而非 panic。
    let nan = UnitArc {
        endpoints: vec![endpoint()],
        sdf_min: f32::NAN,
        sdf_max: 1.0,
        show: true,
    };
    assert!(!nan.is_ordered(), "含 NaN 的区间应判为无序");
}
