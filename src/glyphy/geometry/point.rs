use crate::glyphy::geometry::line::Line;
use crate::glyphy::geometry::signed_vector::SignedVector;
use crate::glyphy::util::float_equals;
use allsorts::pathfinder_geometry::vector::Vector2F;
use pi_shape::plane::Point;
use pi_shape::glam::Vec2;


pub trait PointExt {
    /**
     * Point 转 向量
     */
    fn into_vector(self) -> Vec2;
    /**
     * 到 线l的最短距离
     */
    fn shortest_distance_to_line(&self, l: &Line) -> SignedVector;
    /**
     * 到 点p的距离的平方
     */
    fn squared_distance_to_point(&self, p: &Point) -> f32;
    /**
     * 到 点p的距离
     */
    fn distance_to_point(&self, p: &Point) -> f32;
    /**
     * 取中点
     */
    fn midpoint(&self, p: &Point) -> Point;

    /**
     * 点 减 点
     */
    fn add_vector(&self, p: &Vec2) -> Point;
    /**
     * this 是否等于 p
     */
    fn equals(&self, p: &Point) -> bool;

    fn into_vec2f(&self) -> Vector2F;
}

impl PointExt for Point {
    fn into_vector(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    fn shortest_distance_to_line(&self, l: &Line) -> SignedVector {
        l.sub(&self).neg()
    }

    fn squared_distance_to_point(&self, p: &Point) -> f32 {
        let v = *self - *p;
        v.length_squared()
    }

    fn distance_to_point(&self, p: &Point) -> f32 {
        let v = *self - *p;
        v.length()
    }

    fn midpoint(&self, p: &Point) -> Point {
        return Point::new((self.x + p.x) / 2.0, (self.y + p.y) / 2.0);
    }

    fn add_vector(&self, v: &Vec2) -> Point {
        return Point::new(self.x + v.x, self.y + v.y);
    }

    fn equals(&self, p: &Point) -> bool {
        return float_equals(self.x, p.x, None) && float_equals(self.y, p.y, None);
    }

    fn into_vec2f(&self) -> Vector2F {
        Vector2F::new(self.x, self.y)
    }
}
