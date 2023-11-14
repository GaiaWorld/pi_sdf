use parry2d::{bounding_volume::Aabb, math::Point};

use crate::glyphy::util::GLYPHY_INFINITY;

// use pr
pub trait AabbEXT {
    fn clear(&mut self);
    fn set(&mut self, other: &Aabb);
    fn add(&mut self, p: Point<f32>);
    fn is_empty(&self) -> bool;
    fn extend(&mut self, other: &Aabb);
    fn includes(&self, p: Point<f32>) -> bool;
    fn scale(&mut self, x_scale: f32, y_scale: f32);
}

impl AabbEXT for Aabb {
    fn clear(&mut self) {
        self.maxs = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
        self.mins = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
    }

    fn set(&mut self, other: &Aabb) {
        self.mins.x = other.mins.x;
        self.mins.y = other.mins.y;
        self.maxs.x = other.maxs.x;
        self.maxs.y = other.maxs.y;
    }

    fn add(&mut self, p: Point<f32>) {
        if self.is_empty() {
            self.mins.x = p.x;
            self.mins.y = p.y;
            self.maxs.x = p.x;
            self.maxs.y = p.y;
        }

        self.mins.x = if p.x < self.mins.x { p.x } else { self.mins.x };
        self.mins.y = if p.y < self.mins.y { p.y } else { self.mins.y };

        self.maxs.x = if p.x > self.maxs.x { p.x } else { self.maxs.x };
        self.maxs.y = if p.y > self.maxs.y { p.y } else { self.maxs.y };
    }

    fn is_empty(&self) -> bool {
        // 当最小值是无穷时，包围盒是空的
        return self.mins.x == GLYPHY_INFINITY || self.mins.x == -GLYPHY_INFINITY;
    }

    fn extend(&mut self, other: &Aabb) {
        // 对方是空，就是自己
        if other.is_empty() {
            return;
        }

        // 自己是空，就是对方
        if self.is_empty() {
            self.set(other);
            return;
        }

        self.mins.x = if self.mins.x < other.mins.x {
            self.mins.x
        } else {
            other.mins.x
        };
        self.mins.y = if self.mins.y < other.mins.y {
            self.mins.y
        } else {
            other.mins.y
        };
        self.maxs.x = if self.maxs.x > other.maxs.x {
            self.maxs.x
        } else {
            other.maxs.x
        };
        self.maxs.y = if self.maxs.y > other.maxs.y {
            self.maxs.y
        } else {
            other.maxs.y
        };
    }

    fn includes(&self, p: Point<f32>) -> bool {
        return self.mins.x <= p.x
            && p.x <= self.maxs.x
            && self.mins.y <= p.y
            && p.y <= self.maxs.y;
    }

    fn scale(&mut self, x_scale: f32, y_scale: f32) {
        self.mins.x *= x_scale;
        self.maxs.x *= x_scale;
        self.mins.y *= y_scale;
        self.maxs.y *= y_scale;
    }
}
