// import { float_equals } from "../util";

use parry2d::math::Vector;

pub trait VectorEXT {
    fn sdf_angle(&self) -> f32;
    /// 逆时针旋转90度
    fn ortho(&self) -> Vector<f32>;
    /**
     * 重建向量
     */

    fn rebase(&self, bx: &Vector<f32>, by: &Vector<f32>) -> Vector<f32>;
    fn rebase_other(&self, oth: &Vector<f32>) -> Vector<f32>;
    /**
     * 向量 叉积
     */
    fn sdf_cross(&self, other: &Vector<f32>) -> f32;
}

impl VectorEXT for Vector<f32> {
    fn ortho(&self) -> Vector<f32> {
        Vector::new(-self.y, self.x)
    }

    fn sdf_angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    fn rebase(&self, bx: &Vector<f32>, by: &Vector<f32>) -> Vector<f32> {
        return Vector::new(self.dot(&bx), self.dot(&by));
    }

    fn rebase_other(&self, oth: &Vector<f32>) -> Vector<f32> {
        return self.rebase(oth, &oth.ortho());
    }

    /**
     * 向量 叉积
     */
    fn sdf_cross(&self, other: &Vector<f32>) -> f32 {
        return self.x * other.y - self.y * other.x;
    }
}