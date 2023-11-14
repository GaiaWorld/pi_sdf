

// 浮点数：最小误差
pub const GLYPHY_EPSILON: f32 = 1e-4;

// 浮点数：无穷
pub const GLYPHY_INFINITY: f32 = f32::INFINITY;

pub const GLYPHY_MAX_D: f32 = 0.5;

/**
 * 返回 是否 无穷
 */
pub fn is_inf(x: f32) -> bool {
    return x == f32::INFINITY || x == -f32::INFINITY;
}

/**
 * 比较 浮点数 是否相等
 * @param error; 比较的误差
 */
pub fn float_equals(f1: f32, f2: f32, error: Option<f32>) -> bool {
    let v = (f1 - f2).abs();
    if let Some(e) = error {
        return v < e;
    } else {
        return v < GLYPHY_EPSILON;
    }
}

/**
 * 比较 浮点数 是否等于0
 * @param error 比较的误差
 */
pub fn is_zero(v: f32, error: Option<f32>) -> bool {
    if let Some(e) = error {
        return float_equals(v, 0.0, Some(e));
    } else {
        return float_equals(v, 0.0, Some(GLYPHY_EPSILON * 2.0));
    }
}

/**
 * 异或
 */
pub fn xor(a: bool, b: bool) -> bool {
    return (a || b) && !(a && b);
}

/**
 * 断言：参数为false时，抛异常
 */
pub fn assert(arg: bool, msg: Option<&str>) {
    assert!(arg, "Assertion failed: msg = {:?}", msg);
}