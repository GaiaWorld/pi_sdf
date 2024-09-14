use derive_deref_rs::Deref;
use parry2d::{bounding_volume::Aabb as AabbInner, shape::Segment};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};
use std::{fmt, ops::Range};

use crate::{
    glyphy::{geometry::segment::SegmentEXT, util::GLYPHY_INFINITY},
    Point,
};

use super::arc::Arc;

pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
    Row,
    Col,
}

#[derive(Debug, Clone, Copy, Deref)]
pub struct Aabb(pub AabbInner);
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

            fn visit_map<V>(self, mut map: V) -> Result<Aabb, V::Error>
            where
                V: MapAccess<'de>,
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
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let x1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let y1 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(Aabb(parry2d::bounding_volume::Aabb::new(
                    Point::new(x, y),
                    Point::new(x1, y1),
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
    pub fn new(min: Point, max: Point) -> Self {
        Self(AabbInner::new(min, max))
    }

    pub fn new_invalid() -> Self {
        Self(AabbInner::new_invalid())
    }
    pub fn clear(&mut self) {
        self.maxs = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
        self.mins = Point::new(GLYPHY_INFINITY, GLYPHY_INFINITY);
    }

    pub fn set(&mut self, other: &Aabb) {
        self.mins.x = other.mins.x;
        self.mins.y = other.mins.y;
        self.maxs.x = other.maxs.x;
        self.maxs.y = other.maxs.y;
    }

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

    pub fn is_empty(&self) -> bool {
        // 当最小值是无穷时，包围盒是空的
        return self.mins.x == GLYPHY_INFINITY || self.mins.x == -GLYPHY_INFINITY;
    }

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

    pub fn extend_by(&mut self, x: f32, y: f32) {
        self.mins.x = self.mins.x.min(x);
        self.mins.y = self.mins.y.min(y);
        self.maxs.x = self.maxs.x.max(x);
        self.maxs.y = self.maxs.y.max(y);
    }

    pub fn includes(&self, p: &Point) -> bool {
        return self.mins.x <= p.x
            && p.x <= self.maxs.x
            && self.mins.y <= p.y
            && p.y <= self.maxs.y;
    }

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

    pub fn near_arcs(
        &self,
        arcs: &Vec<&'static Arc>,
        segment: &Segment,
        result_arcs: &mut Vec<&'static Arc>,
        temps: &mut Vec<(Point, f32, Vec<Range<f32>>)>,
    ) {
        // let mut temps = Vec::with_capacity(arcs.len());
        temps.clear();
        // println!("segment: {:?}", segment);
        for i in 0..arcs.len() {
            let (rang, s, min_dist) = arcs[i].projection_to_bound(self, &segment);
            // println!(
            //     "arcs: {:?}, rang: {:?}, dist: {},p: {}",
            //     arcs[i], rang, min_dist, p
            // );
            let p = s.a;
            if i == 0 {
                result_arcs.push(&arcs[i]);
                temps.push((p, min_dist, vec![rang]));
            } else {
                let mut is_push = true;

                for j in 0..result_arcs.len() {
                    let result_arc = result_arcs[j];
                    let dist = result_arc.squared_distance_to_point2(&p).norm_squared();
                    // println!("dist: {}", dist);
                    if min_dist >= dist {
                        let (p1, p2) = if segment.a.x == segment.b.x {
                            (
                                Point::new(segment.a.x, rang.start),
                                Point::new(segment.a.x, rang.end),
                            )
                        } else {
                            (
                                Point::new(rang.start, segment.a.y),
                                Point::new(rang.end, segment.a.y),
                            )
                        };

                        let d11 = result_arc.squared_distance_to_point2(&p1).norm_squared();
                        let d12 = result_arc.squared_distance_to_point2(&p2).norm_squared();

                        let d21 = arcs[i].squared_distance_to_point2(&p1).norm_squared();
                        let d22 = arcs[i].squared_distance_to_point2(&p2).norm_squared();

                        if (d11 < d21 && d12 < d22) || (d11 < d22 && d12 < d21) {
                            is_push = false;
                            break;
                        }
                    }
                }
                let mut delete_index = vec![];
                if is_push {
                    for j in 0..result_arcs.len() {
                        let p = temps[j].0;
                        let dist = temps[j].1;
                        let d = arcs[i].squared_distance_to_point2(&p).norm_squared();
                        // println!("dist: {}, d: {}", dist, d);
                        if d < dist {
                            // let rangs = &mut temps[j].2;
                            // let mut new_rang = vec![];
                            // for r in rangs.iter() {
                            //     if rang.contains(&r.start) && rang.contains(&r.end) {
                            //         continue;
                            //     } else if rang.contains(&r.start) {
                            //         if (rang.end - r.end).abs() > 0.1 {
                            //             new_rang.push(rang.end..r.end)
                            //         }
                            //     } else if rang.contains(&r.end) {
                            //         if (r.start - rang.start).abs() > 0.1 {
                            //             new_rang.push(r.start..rang.start);
                            //         }
                            //     } else if r.contains(&rang.end) && r.contains(&rang.start) {
                            //         if (r.start - rang.start).abs() > 0.1 {
                            //             new_rang.push(r.start..rang.start);
                            //         }

                            //         if (r.end - rang.end).abs() > 0.1 {
                            //             new_rang.push(rang.end..r.end);
                            //         }
                            //     } else {
                            //         if (r.start - r.end).abs() > 0.1 {
                            //             new_rang.push(r.clone());
                            //         }
                            //     }
                            // }
                            delete_index.push(j);
                        }
                    }
                }

                // println!("delete_index: {:?}", delete_index);
                for i in (0..delete_index.len()).rev() {
                    let _r = result_arcs.remove(delete_index[i]);
                    temps.remove(delete_index[i]);
                    // println!("remove : {:?}", r);
                }

                if is_push {
                    // println!("is_push");
                    result_arcs.push(&arcs[i]);
                    temps.push((p, min_dist, vec![rang]));
                }
            }
        }
    }

    // fn near_arcs(
    //     &self,
    //     arcs: &Vec<&'static Arc>,
    //     segment: &Segment,
    //     result_arcs: &mut Vec<&'static Arc>,
    // ) {
    //     let mut temps = vec![];
    //     println!("segment: {:?}", segment);
    //     for i in 0..arcs.len() {
    //         let line0 = segment.squared_distance_to_point2(&arcs[i].p0);
    //         let line1 = segment.squared_distance_to_point2(&arcs[i].p1);
    //         println!("line0: {:?}, line1: {:?}", line0, line1);

    //         if i == 0 {
    //             result_arcs.push(&arcs[i]);
    //             temps.push((line0, line1));
    //         } else {
    //             let mut is_push = true;

    //             let p0 = line0.b;
    //             let p1 = line1.b;

    //             for j in 0..result_arcs.len() {
    //                 let arc = result_arcs[j];

    //                 let dist0 = arc.squared_distance_to_point2(&line0.b).norm_squared();
    //                 let dist1 = if line0.b == line1.b {
    //                     dist0
    //                 } else {
    //                     arc.squared_distance_to_point2(&line1.b).norm_squared()
    //                 };

    //                 if dist0 < line0.norm_squared() && dist1 < line1.norm_squared() {
    //                     is_push = false
    //                 }
    //             }

    //             let mut delete_index = vec![];
    //             if is_push {
    //                 for j in 0..result_arcs.len() {
    //                     let p = temps[j].0;
    //                     let dist = temps[j].1;
    //                     let d = arcs[i].squared_distance_to_point2(&p);
    //                     println!("dist: {}, d: {}", dist, d);
    //                     if d < dist {
    //                         // let rangs = &mut temps[j].2;
    //                         // let mut new_rang = vec![];
    //                         // for r in rangs.iter() {
    //                         //     if rang.contains(&r.start) && rang.contains(&r.end) {
    //                         //         continue;
    //                         //     } else if rang.contains(&r.start) {
    //                         //         if (rang.end - r.end).abs() > 0.1 {
    //                         //             new_rang.push(rang.end..r.end)
    //                         //         }
    //                         //     } else if rang.contains(&r.end) {
    //                         //         if (r.start - rang.start).abs() > 0.1 {
    //                         //             new_rang.push(r.start..rang.start);
    //                         //         }
    //                         //     } else if r.contains(&rang.end) && r.contains(&rang.start) {
    //                         //         if (r.start - rang.start).abs() > 0.1 {
    //                         //             new_rang.push(r.start..rang.start);
    //                         //         }

    //                         //         if (r.end - rang.end).abs() > 0.1 {
    //                         //             new_rang.push(rang.end..r.end);
    //                         //         }
    //                         //     } else {
    //                         //         if (r.start - r.end).abs() > 0.1 {
    //                         //             new_rang.push(r.clone());
    //                         //         }
    //                         //     }
    //                         // }
    //                         // *rangs = new_rang;

    //                         // if rangs.is_empty() {
    //                         //     delete_index.push(j);
    //                         // }

    //                         delete_index.push(j);
    //                     }
    //                 }
    //             }
    //             // println!("delete_index: {:?}", delete_index);
    //             for i in (0..delete_index.len()).rev() {
    //                 let r = result_arcs.remove(delete_index[i]);
    //                 temps.remove(delete_index[i]);
    //                 println!("remove : {:?}", r);
    //             }

    //             if is_push {
    //                 // println!("is_push");
    //                 result_arcs.push(&arcs[i]);
    //                 temps.push((p, min_dist, vec![rang]));
    //             }
    //         }
    //     }
    // }

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
    // println!("dist : {:?}", dist);
}
