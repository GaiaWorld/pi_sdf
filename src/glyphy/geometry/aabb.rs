use derive_deref_rs::Deref;
use parry2d::{bounding_volume::Aabb as AabbInner, shape::Segment};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
use std::fmt;

use crate::{
    glyphy::util::GLYPHY_INFINITY,
    Point,
};

use super::{arc::Arc, segment::{PPoint, PSegment}};

// 方向枚举类型，用于描述几何形状的各个方向
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Top,    // 表示顶部方向
    Bottom, // 表示底部方向
    Left,   // 表示左边方向
    Right,  // 表示右边方向
    Row,    // 行方向
    Col,    // 列方向
}

// Aabb结构体，封装了parry2d库中的AabbInner类型，同时实现了Deref trait以便直接访问内部方法
#[derive(Debug, Clone, Copy, Deref)]
pub struct Aabb(pub AabbInner);

/// Aabb序列化实现，将Aabb结构体序列化为包含四个字段的结构体
impl Serialize for Aabb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Aabb", 4)?;
        s.serialize_field("MinX", &self.mins.x)?;
        s.serialize_field("MinY", &self.mins.y)?;
        s.serialize_field("MaxX", &self.maxs.x)?;
        s.serialize_field("MaxY", &self.maxs.y)?;
        s.end()
    }
}

/// Aabb反序列化实现，通过指定的反序列化器将数据反序列化为Aabb结构体
impl<'de> Deserialize<'de> for Aabb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Field {
            MinX,
            MinY,
            MaxX,
            MaxY,
        }

        struct AabbVisitor;

        impl<'de> Visitor<'de> for AabbVisitor {
            type Value = Aabb;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Aabb")
            }

            /// 从映射结构中反序列化Aabb结构
            fn visit_map<V>(self, mut map: V) -> Result<Aabb, V::Error>
            where
                V: MapAccess<'de>
            {
                let mut min_x = None;
                let mut min_y = None;
                let mut max_x = None;
                let mut max_y = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MinX => {
                            if min_x.is_some() {
                                return Err(de::Error::duplicate_field("min_x"));
                            }
                            min_x = Some(map.next_value()?);
                        }
                        Field::MinY => {
                            if min_y.is_some() {
                                return Err(de::Error::duplicate_field("min_y"));
                            }
                            min_y = Some(map.next_value()?);
                        }

                        Field::MaxX => {
                            if max_x.is_some() {
                                return Err(de::Error::duplicate_field("max_x"));
                            }
                            max_x = Some(map.next_value()?);
                        }

                        Field::MaxY => {
                            if max_y.is_some() {
                                return Err(de::Error::duplicate_field("max_y"));
                            }
                            max_y = Some(map.next_value()?);
                        }
                    }
                }
                let min_x = min_x.ok_or_else(|| de::Error::missing_field("min_x"))?;
                let min_y = min_y.ok_or_else(|| de::Error::missing_field("min_y"))?;
                let max_x = max_x.ok_or_else(|| de::Error::missing_field("max_x"))?;
                let max_y = max_y.ok_or_else(|| de::Error::missing_field("max_y"))?;
                Ok(Aabb(parry2d::bounding_volume::Aabb::new(
                    Point::new(min_x, min_y),
                    Point::new(max_x, max_y),
                )))
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Aabb, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;  // 读取第一个元素作为min_x
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;  // 读取第二个元素作为min_y
                let x1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;  // 读取第三个元素作为max_x
                let y1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;  // 读取第四个元素作为max_y

                Ok(Aabb(parry2d::bounding_volume::Aabb::new(
                    Point::new(x, y),     // 构建最小点
                    Point::new(x1, y1),   // 构建最大点
                )))
            }
        }

        const FIELDS: &'static [&'static str] = &["MinX", "MinY", "MaxX", "MaxY"];
        deserializer.deserialize_struct("Point", FIELDS, AabbVisitor)
    }
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: serde::Deserializer<'de> {

    //         deserializer.deserialize_struct("Aabb", &["mins_x", "mins_y", "maxs_x", "maxs_y"], visitor)
    //     }
}

impl Aabb {
    /// 创建一个新的包围盒实例，使用给定的最小点和最大点
    ///
    /// # 参数
    /// - `min`: 包围盒的最小坐标点
    /// - `max`: 包围盒的最大坐标点
    pub fn new(min: Point, max: Point) -> Self {
        Self(AabbInner::new(min, max))
    }

    /// 创建一个无效的包围盒实例，默认填充为无限值
    pub fn new_invalid() -> Self {
        Self(AabbInner::new_invalid())
    }

    /// 清空包围盒，设置其为无效状态
    pub fn clear(&mut self) {
        self.maxs = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
        self.mins = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
    }

    /// 将另一个包围盒的值复制到当前包围盒中
    ///
    /// # 参数
    /// - `other`: 要复制的另一个包围盒
    pub fn set(&mut self, other: &Aabb) {
        self.mins.clone_from(&other.mins);
        self.maxs.clone_from(&other.maxs);
    }

    /// 将一个点扩展到当前包围盒中
    ///
    /// # 参数
    /// - `p`: 要扩展加入的点
    pub fn add(&mut self, p: Point) {
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

    /// 检查包围盒是否为空（即无效）
    ///
    /// # 返回值
    /// - `bool`: 包围盒是否为空
    pub fn is_empty(&self) -> bool {
        // 当最小值是无穷时，包围盒是空的
        return self.mins.x == GLYPHY_INFINITY || self.mins.x == -GLYPHY_INFINITY;
    }

    /// 通过另一个包围盒扩展当前包围盒，使其包含两个包围盒的内容
    ///
    /// # 参数
    /// - `other`: 要扩展加入的另一个包围盒
    pub fn extend(&mut self, other: &Aabb) {
        // 对方是空，就是自己
        if other.is_empty() {
            return;
        }

        // 自己是空，就是对方
        if self.is_empty() {
            self.set(other);
            return;
        }

        self.mins = self.mins.inf(&other.mins);
        self.maxs = self.maxs.sup(&other.maxs);
    }

    /// 使用给定的 x 和 y 值扩展包围盒
    ///
    /// # 参数
    /// - `x`: 要扩展的 x 值
    /// - `y`: 要扩展的 y 值
    pub fn extend_by(&mut self, x: f32, y: f32) {
        self.mins.x = self.mins.x.min(x);
        self.mins.y = self.mins.y.min(y);
        self.maxs.x = self.maxs.x.max(x);
        self.maxs.y = self.maxs.y.max(y);
    }

    /// 检查点是否在包围盒内
    ///
    /// # 参数
    /// - `p`: 要检查的点
    ///
    /// # 返回值
    /// - `bool`: 点是否在包围盒内部
    pub fn includes(&self, p: &Point) -> bool {
        return self.mins.x <= p.x
            && p.x <= self.maxs.x
            && self.mins.y <= p.y
            && p.y <= self.maxs.y;
    }

    /// 将包围盒的尺寸按照给定的缩放因子进行缩放
    ///
    /// # 参数
    /// - `x_scale`: x 轴方向的缩放因子
    /// - `y_scale`: y 轴方向的缩放因子
    pub fn scale(&mut self, x_scale: f32, y_scale: f32) {
        self.mins.x *= x_scale;
        self.maxs.x *= x_scale;
        self.mins.y *= y_scale;
        self.maxs.y *= y_scale;
    }

    pub fn near_area(&self, direction: Direction) -> Aabb {
        let ab = match direction {
            Direction::Top => AabbInner::new(
                Point::new(self.mins.x, -f32::INFINITY),
                Point::new(self.maxs.x, self.mins.y),
            ),
            Direction::Bottom => AabbInner::new(
                Point::new(self.mins.x, self.maxs.y),
                Point::new(self.maxs.x, f32::INFINITY),
            ),
            Direction::Left => AabbInner::new(
                Point::new(-f32::INFINITY, self.mins.y),
                Point::new(self.mins.x, self.maxs.y),
            ),
            Direction::Right => AabbInner::new(
                Point::new(self.maxs.x, self.mins.y),
                Point::new(f32::INFINITY, self.maxs.y),
            ),
            Direction::Row => AabbInner::new(
                Point::new(self.mins.x, -f32::INFINITY),
                Point::new(self.maxs.x, f32::INFINITY),
            ),
            Direction::Col => AabbInner::new(
                Point::new(-f32::INFINITY, self.mins.y),
                Point::new(f32::INFINITY, self.maxs.y),
            ),
        };
        Self(ab)
    }

    pub fn bound(&self, direction: Direction) -> Segment {
        match direction {
            Direction::Top => Segment::new(self.mins, Point::new(self.maxs.x, self.mins.y)),
            Direction::Bottom => Segment::new(Point::new(self.mins.x, self.maxs.y), self.maxs),
            Direction::Left => Segment::new(self.mins, Point::new(self.mins.x, self.maxs.y)),
            Direction::Right => Segment::new(Point::new(self.maxs.x, self.mins.y), self.maxs),
            _ => panic!("bound not surport col or row!!!"),
        }
    }

    pub fn bound_to_ref(&self, direction: Direction, result: &mut PSegment) {
        match direction {
            Direction::Top => result.modify_by_points((self.mins.x, self.mins.y), (self.maxs.x, self.mins.y)),
            Direction::Bottom => result.modify_by_points((self.mins.x, self.maxs.y), (self.maxs.x, self.maxs.y)),
            Direction::Left => result.modify_by_points((self.mins.x, self.mins.y), (self.mins.x, self.maxs.y)),
            Direction::Right => result.modify_by_points((self.maxs.x, self.mins.y), (self.maxs.x, self.maxs.y)),
            _ => panic!("bound not surport col or row!!!"),
        }
    }


    pub fn near_arcs(
        &self,
        arcs: &Vec<&'static Arc>,

        segment: &PSegment,
        result_arcs: &mut Vec<&'static Arc>,
        temps: &mut Vec<(PPoint, f32)>,
        delete_index: &mut Vec<usize>,
    ) {
        temps.clear();

        let mut temp = segment.clone();
        let mut isfirst = true;
        let mut p1: Point = Point::new(0., 0.);
        let mut p2: Point = Point::new(0., 0.);
        for arc in arcs.iter() {
            let (rang, min_dist) = arc.projection_to_bound_call2(self, segment, &mut temp);

            // log::debug!(
            //     "arcs: {:?}, rang: {:?}, dist: {},p: {}",
            //     arcs[i], rang, min_dist, p
            // );
            // let p = &temp.a;
            let p = &temp.a;
            if isfirst {
                result_arcs.push(*arc);
                temps.push((*p, min_dist));
                isfirst = false;
            } else {
                let mut is_push = true;

                for result_arc in result_arcs.iter() {
                    let dist = result_arc.squared_distance_to_point2_and_norm_square(p);
                    // log::debug!("dist: {}", dist);
                    if min_dist >= dist {
                        if segment.a.x == segment.b.x {
                            p1.x = segment.a.x;
                            p1.y = rang.start;
                            p2.x = segment.a.x;
                            p2.y = rang.end;
                        } else {
                            p1.x = rang.start;
                            p1.y = segment.a.y;
                            p2.x = rang.end;
                            p2.y = segment.a.y;
                        };

                        let d11 = result_arc.squared_distance_to_point2_and_norm_square(&p1);
                        let d12 = result_arc.squared_distance_to_point2_and_norm_square(&p2);

                        let d21 = arc.squared_distance_to_point2_and_norm_square(&p1);
                        let d22 = arc.squared_distance_to_point2_and_norm_square(&p2);

                        if (d11 < d21 && d12 < d22) || (d11 < d22 && d12 < d21) {
                            is_push = false;
                            break;
                        }
                    }
                }

                if is_push {
                    delete_index.clear();
                    for j in 0..result_arcs.len() {
                        let p = temps[j].0;
                        let dist = temps[j].1;
                        // let d = arc.squared_distance_to_point2(&p).norm_squared();
                        let d = arc.squared_distance_to_point2_and_norm_square(&p);
                        // log::debug!("dist: {}, d: {}", dist, d);
                        // 浮点误差
                        if dist - d > 0.01 {
                            delete_index.push(j);
                        }
                    }

                    let len = delete_index.len();
                    for i in 0..len {
                        let idx =  delete_index[len - i - 1];
                        let _r = result_arcs.remove(idx);
                        temps.remove(idx);
                        // log::debug!("remove : {:?}", r);
                    }

                    result_arcs.push(*arc);
                    temps.push((*p, min_dist));
                }
            }
        }
    }

    pub fn width(&self) -> f32 {
        self.maxs.x - self.mins.x
    }

    pub fn height(&self) -> f32 {
        self.maxs.y - self.mins.y
    }

    pub fn half(&self, direction: Direction) -> (Aabb, Aabb) {
        match direction {
            Direction::Row => {
                let temp_y = self.mins.y + (self.maxs.y - self.mins.y) / 2.0;
                (
                    Self(AabbInner::new(self.mins, Point::new(self.maxs.x, temp_y))),
                    Self(AabbInner::new(Point::new(self.mins.x, temp_y), self.maxs)),
                )
            }
            Direction::Col => {
                let temp_x = self.mins.x + (self.maxs.x - self.mins.x) / 2.0;
                (
                    Self(AabbInner::new(self.mins, Point::new(temp_x, self.maxs.y))),
                    Self(AabbInner::new(Point::new(temp_x, self.mins.y), self.maxs)),
                )
            }
            _ => panic!("half not surport!!!"),
        }
    }

    pub fn collision(&self, other: &Aabb) -> Option<Aabb> {
        let minx = self.mins.x.max(other.mins.x);
        let miny = self.mins.y.max(other.mins.y);
        let maxx = self.maxs.x.min(other.maxs.x);
        let maxy = self.maxs.y.min(other.maxs.y);

        if minx <= maxx && miny <= maxy {
            return Some(Self(AabbInner::new(Point::new(minx, miny), Point::new(maxx, maxy))));
        }

        return None;
    }
}


#[test]
fn test() {
    // let arc = Arc::new(
    //     Point::new(220.0, 171.0),
    //     Point::new(91.0, 744.0),
    //     -0.14173229,
    // );
    // let dist = arc.squared_distance_to_point2(&Point::new(216.85324, 171.0));
    // log::debug!("dist : {:?}", dist);
}
