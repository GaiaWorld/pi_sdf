//! TASK-02：统一错误类型（API-002）的对外行为。
//!
//! 仅经由冻结接口（`Error` 的 Display / Debug / std::error::Error）断言，
//! 不直接调用 error_inner。

use pi_sdf2::model::Error;

fn all_variants() -> Vec<Error> {
    vec![
        Error::InvalidFont("glyf table missing"),
        Error::InvalidPathVerb(200),
        Error::InvalidParam("tex_size is 0"),
        Error::Decode("truncated header"),
        Error::Encode("offset overflow"),
        Error::Geometry("endpoint is NaN"),
    ]
}

#[test]
fn display_is_never_empty() {
    for e in all_variants() {
        let s = e.to_string();
        assert!(!s.is_empty(), "Display 不应为空：{:?}", e);
    }
}

#[test]
fn display_includes_inner_message() {
    assert!(Error::InvalidParam("tex_size is 0").to_string().contains("tex_size is 0"));
    assert!(Error::InvalidPathVerb(200).to_string().contains("200"));
    assert!(Error::Geometry("endpoint is NaN").to_string().contains("endpoint is NaN"));
}

#[test]
fn display_distinguishes_variants() {
    assert_ne!(Error::Decode("x").to_string(), Error::Encode("x").to_string());
    assert_ne!(Error::InvalidFont("x").to_string(), Error::InvalidParam("x").to_string());
    assert_ne!(Error::InvalidParam("x").to_string(), Error::Geometry("x").to_string());
}

#[test]
fn error_is_a_std_error() {
    let e = Error::Geometry("x");
    let as_std: &dyn std::error::Error = &e;
    assert!(!as_std.to_string().is_empty());
    assert!(!format!("{:?}", e).is_empty());
}
