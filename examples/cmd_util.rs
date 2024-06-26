use std::{io::{Cursor, Read}, path::PathBuf, sync::Arc};

use parry2d::na::{self};
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;

// use nalgebra::Vector3;
use pi_sdf::{font::{FontFace, SdfInfo}, glyphy::blob::TexData, svg::Svg, utils::create_indices};
use pi_wgpu as wgpu;
use wgpu::{util::DeviceExt, BlendState, ColorTargetState};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};


fn main() {
    let cfg = std::fs::read("./sdf.json").unwrap();
    let cfg  = serde_json::from_slice::<serde_json::Value>(&cfg).unwrap();
    let mut buffer = Vec::new();
    let mut buffer2: Vec<u8> = Vec::new();
    for font in cfg.as_array().unwrap().iter(){
        let path = PathBuf::from(font["path"].as_str().unwrap());
        let family_name = font["family_name"].as_str().unwrap();
        let font_data = std::fs::read(&path).unwrap();
        let text = font["text"].as_str().unwrap();
        println!("text: {}", text);
        let mut font_face = FontFace::new(font_data);
        let mut sdf = font_face.compute_text_sdf(text);
        let mut count = 0;
        for item in &sdf{
            // println!("==============1");
            buffer2.extend(&item.data_tex);
            println!("===========1: {:?}", (buffer2.len(), item.data_tex.len() / 4));
            buffer2.extend(&item.index_tex);
            println!("===========2: {:?}", (buffer2.len(), item.index_tex.len()));
            buffer2.extend(&item.sdf_tex1);
            println!("===========3: {:?}", (buffer2.len(), item.sdf_tex1.len()));
            count += 1;
        };
        
        println!("count: {}", count);

        buffer.push((family_name.to_string(), sdf));
        
    }

    let time = std::time::Instant::now();
    let buffer = bincode::serialize(&buffer).unwrap();
    println!("time: {:?}", time.elapsed());
    let mut out = Vec::new();
    let mut reader = brotli::CompressorReader::new(Cursor::new(&buffer), buffer.len() /* buffer size */, 6, 22);
    reader.read_to_end(&mut out).unwrap();

    std::fs::write("./font.sdf2", &out).unwrap();

    let time = std::time::Instant::now();
    let decoded: Vec<(String, Vec<SdfInfo>)> = bincode::deserialize(&buffer[..]).unwrap();
    println!("time2: {:?}", time.elapsed());
    println!("decoded: {:?}", (&decoded[0].0, &decoded[0].1[0].tex_info));
}