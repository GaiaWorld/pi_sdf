use std::{
    fmt,
    io::{Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

// use bincode::Deserializer;
use derive_deref_rs::Deref;
use parry2d::{
    math::Point,
    na::{self},
};
use serde::{
    de::{self, Error, MapAccess, SeqAccess, Unexpected, Visitor},
    ser::{SerializeMap, SerializeStruct},
    Deserialize, Serialize, Serializer,
};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;
// use nalgebra::Vector3;
use pi_sdf::{
    font::{FontFace, SdfInfo},
    glyphy::blob::TexData,
    svg::Svg,
    utils::create_indices,
};
use pi_wgpu as wgpu;
use serde::Deserializer;
use wgpu::{util::DeviceExt, BlendState, ColorTargetState};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
#[derive(Debug, Deref)]
struct Aabb(parry2d::bounding_volume::Aabb);

impl Serialize for Aabb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Aabb", 4)?;
        s.serialize_field("min_x", &self.mins.x)?;
        s.serialize_field("min_y", &self.mins.y)?;
        s.serialize_field("max_x", &self.maxs.x)?;
        s.serialize_field("max_y", &self.maxs.y)?;
        s.end()
    }
}
#[derive(Deserialize)]
enum Field {
    min_x,
    min_y,
    max_x,
    max_y,
}

impl<'de> Deserialize<'de> for Aabb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
                // let mut min_x = None;
                // let mut min_y = None;
                // let mut max_x = None;
                // let mut max_y = None;
                // while let Some(key) = map.next_key::<Field>()? {
                //     match key {
                //         Field::min_x => {
                //             if min_x.is_some() {
                //                 return Err(de::Error::duplicate_field("min_x"));
                //             }
                //             min_x = Some(map.next_value::<f32>()?);
                //         }
                //         Field::min_y => {
                //             if min_y.is_some() {
                //                 return Err(de::Error::duplicate_field("min_y"));
                //             }
                //             min_y = Some(map.next_value::<f32>()?);
                //         }
                //         Field::max_x => {
                //             if max_x.is_some() {
                //                 return Err(de::Error::duplicate_field("max_x"));
                //             }
                //             max_x = Some(map.next_value::<f32>()?);
                //         },
                //         Field::max_y => {
                //             if max_y.is_some() {
                //                 return Err(de::Error::duplicate_field("max_y"));
                //             }
                //             max_y = Some(map.next_value::<f32>()?);
                //         },
                //     }
                // }
                // let min_x = min_x.ok_or_else(|| de::Error::missing_field("x"))?;
                // let min_y = min_y.ok_or_else(|| de::Error::missing_field("y"))?;
                // let max_x = max_x.ok_or_else(|| de::Error::missing_field("x"))?;
                // let max_y = max_y.ok_or_else(|| de::Error::missing_field("y"))?;
                Ok(Aabb(parry2d::bounding_volume::Aabb::new(
                    Point::new(1.0, 1.0),
                    Point::new(1.0, 1.0),
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

        // fn visit_ne

        println!("=============.");
        const FIELDS: &'static [&'static str] = &["min_x", "min_y", "max_x", "max_y"];
        deserializer.deserialize_struct("Aabb", FIELDS, AabbVisitor)
    }

    // fn v
}

#[derive(Debug)]
struct Point2 {
    x: i32,
    y: i32,
}

// 手动实现 Serialize 特性
impl Serialize for Point2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Point2", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

// 手动实现 Deserialize 特性
impl<'de> Deserialize<'de> for Point2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Field {
            x,
            y,
        };

        struct PointVisitor;

        impl<'de> Visitor<'de> for PointVisitor {
            type Value = Point2;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Point2")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Point2, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;
                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::x => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }
                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
                Ok(Point2 { x, y })
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Point2, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Point2 { x, y })
            }
        }

        const FIELDS: &'static [&'static str] = &["x", "y"];
        deserializer.deserialize_struct("Point2", FIELDS, PointVisitor)
    }
}

fn main() {
    let json_data = Point2 { x: 1, y: 3 };
    let bytes = bincode::serialize(&json_data).unwrap();
    println!("bytes: {:?}", bytes);
    let point: Point2 = bincode::deserialize(&bytes).unwrap();
    println!("{:?}", point);

    let ab = Aabb(parry2d::bounding_volume::Aabb::new(Point::new(1., 2.), Point::new(3., 4.)));

    let bytes = bincode::serialize(&ab).unwrap();
    let ab : Aabb= bincode::deserialize(&bytes).unwrap();

    println!("{:?}", ab);
}
// fn main() {
//     let cfg = std::fs::read("./sdf.json").unwrap();
//     let cfg  = serde_json::from_slice::<serde_json::Value>(&cfg).unwrap();
//     let mut buffer = Vec::new();
//     let mut buffer2: Vec<u8> = Vec::new();
//     for font in cfg.as_array().unwrap().iter(){
//         let path = PathBuf::from(font["path"].as_str().unwrap());
//         let family_name = font["family_name"].as_str().unwrap();
//         let font_data = std::fs::read(&path).unwrap();
//         let text = font["text"].as_str().unwrap();
//         println!("text: {}", text);
//         let mut font_face = FontFace::new(font_data);
//         let mut sdf = font_face.compute_text_sdf(text);
//         let mut count = 0;
//         for item in &sdf{
//             // println!("==============1");
//             buffer2.extend(&item.data_tex);
//             println!("===========1: {:?}", (buffer2.len(), item.data_tex.len() / 4));
//             buffer2.extend(&item.index_tex);
//             println!("===========2: {:?}", (buffer2.len(), item.index_tex.len()));
//             buffer2.extend(&item.sdf_tex1);
//             println!("===========3: {:?}", (buffer2.len(), item.sdf_tex1.len()));
//             count += 1;
//         };

//         println!("count: {}", count);

//         buffer.push((family_name.to_string(), sdf));

//     }

//     let time = std::time::Instant::now();
//     let buffer = bincode::serialize(&buffer).unwrap();
//     println!("time: {:?}", time.elapsed());
//     let mut out = Vec::new();
//     let mut reader = brotli::CompressorReader::new(Cursor::new(&buffer), buffer.len() /* buffer size */, 6, 22);
//     reader.read_to_end(&mut out).unwrap();

//     std::fs::write("./font.sdf2", &out).unwrap();

//     let time = std::time::Instant::now();
//     let decoded: Vec<(String, Vec<SdfInfo>)> = bincode::deserialize(&buffer[..]).unwrap();
//     println!("time2: {:?}", time.elapsed());
//     println!("decoded: {:?}", (&decoded[0].0, &decoded[0].1[0].tex_info));
// }
