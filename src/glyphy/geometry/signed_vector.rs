

use pi_shape::glam::Vec2;

use crate::glyphy::util::float_equals;

#[derive(Debug, Clone)]
pub struct SignedVector {
    pub vec2: Vec2,
    pub negative: bool,
}
impl SignedVector {
    pub fn new(x: f32, y: f32, negative: bool) -> Self {
        Self {
            vec2: Vec2::new(x, y),
            negative,
        }
    }

    /**
     * 从向量 创建 SignedVector
     */
    pub fn from_vector(v: Vec2, negative: bool) -> Self {
        Self { vec2: v, negative }
    }

    pub fn neg(&self) -> Self {
        Self {
            vec2: -self.vec2,
            negative: !self.negative,
        }
    }
}

/**
 * this 是否等于 sv
 */
impl PartialEq for SignedVector {
    fn eq(&self, other: &Self) -> bool {
        float_equals(self.vec2.x, other.vec2.x, None)
            && float_equals(self.vec2.y, other.vec2.y, None)
            && self.negative == other.negative
    }
}
