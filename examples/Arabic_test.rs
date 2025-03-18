use std::sync::Arc;

use allsorts::{binary::read::ReadScope, font::MatchingPresentation, font_data::FontData, gsub::{FeatureMask, Features}, tag, Font};
use image::ColorType;

// use nalgebra::Vector3;
use pi_sdf::font::FontFace;


fn main() {
    let buffer = std::fs::read("./source/Rubik-VariableFont_wght.ttf").unwrap();
    let mut ft_face = FontFace::new(Arc::new(buffer));

    println!("max_box_normaliz: {:?}", ft_face.max_box_normaliz());
    let pxrange = 10;
    let time = std::time::Instant::now();
    let text = "اللغة العربية ";
    let mut i = 0;
    for c in text.chars(){
        let outline_info = ft_face.to_outline(c);

        let result_arcs = outline_info.compute_near_arcs(2.0);
        let pxrange = 5;
        let glpyh_info = outline_info.compute_sdf_tex(result_arcs, 32, pxrange, false, pxrange);
        // let glpyh_info = FontFace::compute_sdf_tex(outline_info.clone(),  32, pxrange, false);
        println!("time4: {:?}", time.elapsed());
        println!("glpyh_info: {:?}", glpyh_info.tex_info);
        let tex_size = glpyh_info.tex_size;
        let _ = image::save_buffer(
            format!("image{}.png", i),
            &glpyh_info.sdf_tex,
            tex_size as u32,
            tex_size as u32,
            ColorType::L8,
        );
        i+=1;
    }


    let script = tag::ARAB;
    let lang = tag::DFLT;
    let buffer = std::fs::read("./source/Rubik-VariableFont_wght.ttf")
        .expect("unable to read Klei.otf");
    let scope = ReadScope::new(&buffer);
    let font_file = scope.read::<FontData<'_>>().expect("unable to parse font");
    // Use a different index to access other fonts in a font collection (E.g. TTC)
    let provider = font_file
        .table_provider(0)
        .expect("unable to create table provider");
    let mut font = Font::new(provider)
        .expect("unable to load font tables")
        .expect("unable to find suitable cmap sub-table");
    
    let text = "لم";
    // Klei ligates ff
    let glyphs = font.map_glyphs(text, script, MatchingPresentation::NotRequired);
    let glyph_infos = font
        .shape(
            glyphs,
            script,
            Some(lang),
            &Features::Mask(FeatureMask::default()),
            true,
        )
        .expect("error shaping text");
    // We expect ff to be ligated so the number of glyphs (18) should be one less than the
    // number of input characters (19).
    // assert_eq!(glyph_infos.len(), 18);
    println!("glyph_infos: {:?}", glyph_infos);
    println!("text: {:?}", text.len());
    
}
