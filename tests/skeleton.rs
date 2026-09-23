//! TASK-01 骨架冒烟：常量与数值工具的等价语义（REQ-001）。

use pi_sdf2::model::base::{consts, num};

#[test]
fn tolerance_matches_baseline() {
    assert_eq!(consts::TOLERANCE, 10.0 / 1024.0);
}

#[test]
fn distance_and_grid_limits_match_baseline() {
    assert_eq!(consts::FARWAY, 20.0);
    assert_eq!(consts::MAX_GRID_SIZE, 63.0);
    assert_eq!(consts::GLYPHY_MAX_D, 0.5);
    assert_eq!(consts::SCALE, 2048.0);
}

#[test]
fn is_inf_detects_both_infinities_only() {
    assert!(num::is_inf(f32::INFINITY));
    assert!(num::is_inf(f32::NEG_INFINITY));
    assert!(!num::is_inf(1.0));
    assert!(!num::is_inf(f32::NAN));
}

#[test]
fn float_equals_defaults_to_epsilon() {
    assert!(num::float_equals(1.0, 1.0, None));
    assert!(num::float_equals(1.0, 1.0 + 1e-6, None));
    assert!(!num::float_equals(1.0, 1.1, None));
    assert!(num::float_equals(1.0, 1.05, Some(0.1)));
}

#[test]
fn is_zero_uses_doubled_epsilon_by_default() {
    assert!(num::is_zero(0.0, None));
    assert!(num::is_zero(1e-5, None));
    assert!(!num::is_zero(1e-3, None));
    assert!(num::is_zero(1e-3, Some(0.01)));
}

#[test]
fn float2_equals_compares_each_component() {
    assert!(num::float2_equals(&[1.0, 2.0], &[1.0, 2.0]));
    assert!(!num::float2_equals(&[1.0, 2.0], &[1.0, 2.1]));
}
