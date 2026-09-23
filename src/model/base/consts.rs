//! 常量与数值容差。
//!
//! 取值与 pi_sdf 0.1.34 现状一致（REQ-001 等价性前提）。数值来源：
//! utils.rs、shape.rs、glyphy/util.rs、glyphy/blob.rs。

/// 坐标等价容差：10/1024，对应 REQ-001 的坐标 ≤1/1024 判据。
pub const TOLERANCE: f32 = 10.0 / 1024.0;

/// 距离场远点标记：距离超过它即可视为无穷远。
pub const FARWAY: f32 = 20.0;

/// 网格细分的单元数上限。
pub const MAX_GRID_SIZE: f32 = 63.0;

/// 单位弧纹理横坐标上限（量化解码）。
pub const MAX_X: f32 = 4095.0;

/// 单位弧纹理纵坐标上限（量化解码）。
pub const MAX_Y: f32 = 4095.0;

/// 归一化缩放基准。
pub const SCALE: f32 = 2048.0;

/// 曲率参数绝对值上限：超过即视为大弧。
pub const GLYPHY_MAX_D: f32 = 0.5;

/// 可用于测量边距的最小字号。
pub const MIN_FONT_SIZE: f32 = 10.0;

/// 每 EM 的最大加亮量。
pub const ENLIGHTEN_MAX: f32 = 0.0001;

/// 每 EM 的最大加粗量。
pub const EMBOLDEN_MAX: f32 = 0.0001;

/// 调试与测量用字符集。
pub const CHARS: &str = ".1Il-一|";
