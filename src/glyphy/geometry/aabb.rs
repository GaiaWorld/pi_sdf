use std::ops::Range;


use pi_shape::plane::{aabb::Aabb, segment::Segment, Point};

use crate::glyphy::{geometry::segment::SegmentEXT, util::GLYPHY_INFINITY};

use super::arc::Arc;

pub enum Direction {
    Top,
    Bottom,
    Left,
    Right,
    Row,
    Col,
}
// use pr
pub trait AabbEXT {
    fn clear(&mut self);
    fn set(&mut self, other: &Aabb);
    fn add(&mut self, p: Point);
    fn is_empty(&self) -> bool;
    fn extend(&mut self, other: &Aabb);
    fn includes(&self, p: &Point) -> bool;
    fn scale(&mut self, x_scale: f32, y_scale: f32);
    fn near_area(&self, direction: Direction) -> Aabb;
    fn near_arcs<'a>(
        &self,
        arcs: &Vec<&'static Arc>,
        segment: &Segment,
        result: &mut Vec<&'static Arc>,
        temps : &mut Vec<(Point, f32, Vec<Range<f32>>)>,
    );
    fn bound(&self, direction: Direction) -> Segment;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn half(&self, direction: Direction) -> (Aabb, Aabb);
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

    fn add(&mut self, p: Point) {
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

    fn includes(&self, p: &Point) -> bool {
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

    fn near_area(&self, direction: Direction) -> Aabb {
        match direction {
            Direction::Top => Aabb::new(
                Point::new(self.mins.x, -f32::INFINITY),
                Point::new(self.maxs.x, self.mins.y),
            ),
            Direction::Bottom => Aabb::new(
                Point::new(self.mins.x, self.maxs.y),
                Point::new(self.maxs.x, f32::INFINITY),
            ),
            Direction::Left => Aabb::new(
                Point::new(-f32::INFINITY, self.mins.y),
                Point::new(self.mins.x, self.maxs.y),
            ),
            Direction::Right => Aabb::new(
                Point::new(self.maxs.x, self.mins.y),
                Point::new(f32::INFINITY, self.maxs.y),
            ),
            Direction::Row => Aabb::new(
                Point::new(self.mins.x, -f32::INFINITY),
                Point::new(self.maxs.x, f32::INFINITY),
            ),
            Direction::Col => Aabb::new(
                Point::new(-f32::INFINITY, self.mins.y),
                Point::new(f32::INFINITY, self.maxs.y),
            ),
        }
    }

    fn bound(&self, direction: Direction) -> Segment {
        match direction {
            Direction::Top => Segment::new(self.mins, Point::new(self.maxs.x, self.mins.y)),
            Direction::Bottom => Segment::new(Point::new(self.mins.x, self.maxs.y), self.maxs),
            Direction::Left => Segment::new(self.mins, Point::new(self.mins.x, self.maxs.y)),
            Direction::Right => Segment::new(Point::new(self.maxs.x, self.mins.y), self.maxs),
            _ => panic!("bound not surport col or row!!!"),
        }
    }

    fn near_arcs(
        &self,
        arcs: &Vec<&'static Arc>,
        segment: &Segment,
        result_arcs: &mut Vec<&'static Arc>,
        temps : &mut Vec<(Point, f32, Vec<Range<f32>>)>,
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
                    let dist = result_arc.squared_distance_to_point2(&p).length_squared();
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

                        let d11 = result_arc.squared_distance_to_point2(&p1).length_squared();
                        let d12 = result_arc.squared_distance_to_point2(&p2).length_squared();

                        let d21 = arcs[i].squared_distance_to_point2(&p1).length_squared();
                        let d22 = arcs[i].squared_distance_to_point2(&p2).length_squared();

                        if (d11 < d21 && d12 < d22) || (d11 < d22  && d12 < d21){
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
                        let d = arcs[i].squared_distance_to_point2(&p).length_squared();
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

    //                 let dist0 = arc.squared_distance_to_point2(&line0.b).length_squared();
    //                 let dist1 = if line0.b == line1.b {
    //                     dist0
    //                 } else {
    //                     arc.squared_distance_to_point2(&line1.b).length_squared()
    //                 };

    //                 if dist0 < line0.length_squared() && dist1 < line1.length_squared() {
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

    fn width(&self) -> f32 {
        self.maxs.x - self.mins.x
    }

    fn height(&self) -> f32 {
        self.maxs.y - self.mins.y
    }

    fn half(&self, direction: Direction) -> (Aabb, Aabb) {
        match direction {
            Direction::Row => {
                let temp_y = self.mins.y + (self.maxs.y - self.mins.y) / 2.0;
                (
                    Aabb::new(self.mins, Point::new(self.maxs.x, temp_y)),
                    Aabb::new(Point::new(self.mins.x, temp_y), self.maxs),
                )
            }
            Direction::Col => {
                let temp_x = self.mins.x + (self.maxs.x - self.mins.x) / 2.0;
                (
                    Aabb::new(self.mins, Point::new(temp_x, self.maxs.y)),
                    Aabb::new(Point::new(temp_x, self.mins.y), self.maxs),
                )
            }
            _ => panic!("half not surport!!!"),
        }
    }
}

#[test]
fn test() {
    let arc = Arc::new(
        Point::new(220.0, 171.0),
        Point::new(91.0, 744.0),
        -0.14173229,
    );
    let dist = arc.squared_distance_to_point2(&Point::new(216.85324, 171.0));
    println!("dist : {:?}", dist);
}
