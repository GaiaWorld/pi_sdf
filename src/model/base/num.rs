//! 基础数值工具。
//!
//! 语义与 pi_sdf 的 glyphy/util.rs 一致（REQ-001 等价性前提）。

/// 默认浮点比较误差。
pub const EPSILON: f32 = 1e-4;

/// 浮点无穷。
pub const INFINITY: f32 = f32::INFINITY;

/// 是否为正负无穷。
pub fn is_inf(x: f32) -> bool {
    x == f32::INFINITY || x == f32::NEG_INFINITY
}

/// 以误差判定浮点相等；error 为 None 时用 EPSILON。
pub fn float_equals(f1: f32, f2: f32, error: Option<f32>) -> bool {
    let v = (f1 - f2).abs();
    match error {
        Some(e) => v < e,
        None => v < EPSILON,
    }
}

/// 二维浮点数组逐分量相等。
pub fn float2_equals(v1: &[f32; 2], v2: &[f32; 2]) -> bool {
    float_equals(v1[0], v2[0], None) && float_equals(v1[1], v2[1], None)
}

/// 以误差判定浮点是否为 0；error 为 None 时用 2 倍 EPSILON。
pub fn is_zero(v: f32, error: Option<f32>) -> bool {
    match error {
        Some(e) => float_equals(v, 0.0, Some(e)),
        None => float_equals(v, 0.0, Some(EPSILON * 2.0)),
    }
}

/// 布尔异或。
pub fn xor(a: bool, b: bool) -> bool {
    a != b
}
