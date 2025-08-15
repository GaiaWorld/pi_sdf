use allsort::font::{Font, FontData};
use anyhow::{bail, Context, Result};
use font_kit::handle::Handle;
use pi_sdf::font::FontFace;
use pi_share::Share;
use std::fs;
use std::sync::Arc;

// 新的、正确的函数，使用 font-kit
fn parse_font_from_ttc(ttc_data: &[u8], font_index: u32) -> Result<Font<'static>> {
    // 1. 将数据包装进 Arc<Vec<u8>>，这是 font-kit 期望的格式
    let arc_data = Arc::new(ttc_data.to_vec());

    // 2. 使用 font-kit 的 Handle 从内存数据和索引中创建句柄
    // font-kit 在内部处理了 TTC 的所有复杂性
    let handle = Handle::from_memory(arc_data, font_index);

    // 3. 从句柄加载字体。font-kit 会返回一个 Font 对象
    let font_kit_font = font_kit::font::Font::from_handle(&handle)
        .context("font-kit failed to load font from handle.")?;

    // 4. (核心步骤) 从 font-kit 的 Font 对象中复制出独立的字体数据
    let ttf_data_arc = font_kit_font
        .copy_font_data()
        .context("font-kit could not copy font data.")?;
        
    let ttf_data = ttf_data_arc.context("Extracted font data is empty.")?;

    println!(
        "Successfully extracted font #{} data with font-kit (size: {} bytes). Now passing to allsort.",
        font_index,
        ttf_data.len()
    );

    // 5. 将提取出的 ttf 数据传递给 allsort
    // 注意：因为 ttf_data 是一个 Arc<Vec<u8>>，它拥有数据所有权。
    // 我们需要将其转换为 'static 生命周期的数据以便 allsort 解析。
    // 最简单的方式是将其内容泄漏掉，或者找到 allsort 接受 owned data 的方式。
    // 这里我们使用 Box::leak 将其生命周期变为 'static
    let ttf_data = FontFace::new(Share::new(ttf_data));

    Ok(font)
}


fn main() -> Result<()> {
    let ttc_data = fs::read("NotoSansCJK-Regular.ttc")
        .unwrap();

    // 尝试解析 TTC 中的第一个字体 (索引 0)
    let font = parse_font_from_ttc(&ttc_data, 0)?;
    println!("Successfully parsed font with allsort!");

    // ... 后续使用 allsort 的 font 对象 ...

    Ok(())
}

// fn main(){
//     let data = std::fs::read("NotoSansCJK-Regular.ttc").unwrap();
//     // let face = Face::parse(&data, 9).unwrap();
//     // for n in  face.names().into_iter(){
//     //     println!("Font family: {:?}", n);
//     // }
//     // println!("Font family: {:?}", face.names().into_iter().find(|n| n.name_id == 1));
//     let r = fonts_in_collection(&data);
//     if let Some(r) = r {
//         for i in 0..r{
//             let face = Face::parse(&data, i).unwrap();
            
//             println!("Font family: {:?}", face.names().into_iter().find(|n| n.name_id == 1));
//             face.outline_glyph(glyph_id, builder)
//         }
//     }

//     let ttc = ttf_parser::ttc::parse_ttc(ttc_data)
//     .context("Failed to parse TTC header. Is this a valid TTC file?")?;

// // 2. 检查索引是否有效
// if font_index >= ttc.len() {
//     bail!(
//         "Invalid font index {} but TTC only contains {} fonts.",
//         font_index,
//         ttc.len()
//     );
// }

// // 3. 获取单个字体数据的偏移量 (offset)
// // 这是最关键的一步，我们拿到了这个字体在整个文件中的起始位置
// let font_offset = ttc.offset_table()[font_index as usize];
//     // println!("============ r: {:?}", (r, face.().len()));
//     // face.
//     // extract_ttc_fonts("NotoSansCJK-Regular.ttc", "./");
// }