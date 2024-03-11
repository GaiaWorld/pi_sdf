// import { float_equals } from "../util";

use pi_shape::glam::Vec2;

pub trait VectorEXT {
    fn sdf_angle(&self) -> f32;
    /// 逆时针旋转90度
    fn ortho(&self) -> Vec2;
    /**
     * 重建向量
     */

    fn rebase(&self, bx: &Vec2, by: &Vec2) -> Vec2;
    fn rebase_other(&self, oth: &Vec2) -> Vec2;
    /**
     * 向量 叉积
     */
    fn sdf_cross(&self, other: &Vec2) -> f32;
}

impl VectorEXT for Vec2 {
    fn ortho(&self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }

    fn sdf_angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    fn rebase(&self, bx: &Vec2, by: &Vec2) -> Vec2 {
        return Vec2::new(self.dot(*bx), self.dot(*by));
    }

    fn rebase_other(&self, oth: &Vec2) -> Vec2 {
        return self.rebase(oth, &oth.ortho());
    }

    /**
     * 向量 叉积
     */
    fn sdf_cross(&self, other: &Vec2) -> f32 {
        return self.x * other.y - self.y * other.x;
    }
}
// export class Vec2 {

//     x: number;
//     y: number;

//     constructor(x_ = 0.0, y_ = 0.0) {
//         this.x = x_;
//         this.y = y_;
//     }

//     /**
//      * 克隆 向量
//      */
//     clone() {
//         return new Vec2(this.x, this.y);
//     }

//     /**
//      * 向量的 长度
//      */
//     equals(v: Vec2) {
//         return float_equals(this.x, v.x) && float_equals(this.y, v.y);
//     }

//     /**
//      * 向量 取反
//      * @returns {Vec2}
//      */
//     neg() {
//         return new Vec2(-this.x, -this.y);
//     }

//     /**
//      * 向量 加法
//      */
//     add(v: Vec2) {
//         return new Vec2(this.x + v.x, this.y + v.y);
//     }

//     /**
//      * 向量 减法
//      */
//     sub(v: Vec2) {
//         return new Vec2(this.x - v.x, this.y - v.y);
//     }

//     /**
//      * 向量 数量积
//      */
//     scale(s: number) {
//         return new Vec2(this.x * s, this.y * s);
//     }

//     /**
//      * 向量 数量商
//      */
//     div(s: number) {
//         return new Vec2(this.x / s, this.y / s);
//     }

//     /**
//      * 加法 赋值
//      */
//     add_assign(v: Vec2) {
//         this.x += v.x;
//         this.y += v.y;
//         return this;
//     }

//     /**
//      * 减法 赋值
//      */
//     sub_assign(v: Vec2) {
//         this.x -= v.x;
//         this.y -= v.y;
//         return this;
//     }

//     /**
//      * 数量积 赋值
//      */
//     scale_assign(s: number) {
//         this.x *= s;
//         this.y *= s;
//         return this;
//     }

//     /**
//      * 数量商 赋值
//      */
//     div_assign(s: number) {
//         this.x /= s;
//         this.y /= s;
//         return this;
//     }

//     /**
//      * 向量 点积
//      */
//     dot(v: Vec2) {
//         return this.x * v.x + this.y * v.y;
//     }

//     /**
//      * 向量 叉积
//      */
//     cross(other: Vec2) {
//         return this.x * other.y - this.y * other.x;
//     }

//     /**
//      * 是否 为零向量
//      */
//     is_zero() {
//         return float_equals(this.x, 0.0) && float_equals(this.y, 0.0)
//     }

//     /**
//      * 向量 长度的平方
//      */
//     len2() {
//         return this.dot(this)
//     }

//     /**
//      * 向量 长度
//      */
//     len() {
//         return Math.sqrt(this.len2());
//     }

//     /**
//      * 向量 归一化
//      */
//     normalized() {
//         let d = this.len();
//         return float_equals(d, 0.0) ? this.clone() : this.div(d);
//     }

//     /**
//      * 垂直 向量
//      */
//     ortho() {
//         return new Vec2(-this.y, this.x);
//     }

//     /**
//      * 垂直 单位向量
//      */
//     normal() {
//         return this.ortho().normalized();
//     }

//     /**
//      * 向量 角度
//      */
//     angle() {
//         return Math.atan2(this.y, this.x);
//     }

//     /**
//      * 重建向量
//      */
//     rebase(bx: Vec2, by: Vec2) {
//         return new Vec2(this.dot(bx), this.dot(by));
//     }

//     /**
//      * 重建向量
//      */
//     rebase_other(bx: Vec2) {
//         return this.rebase(bx, bx.ortho());
//     }
// }
