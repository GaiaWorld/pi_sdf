use pi_sdf::font::FontFace;

const FONT_SIZE: usize = 32;
const PXRANGE: u32 = 7;

fn main() {
    let buffer = std::fs::read("E:/app_new_gui/pi_ui_render/examples/text_sdf2/source/ht.ttf").unwrap();
    let mut face = FontFace::new(buffer.into());
    for ch in ['.'] {
        let o = face.to_outline(ch);
        let l = o.compute_layout(FONT_SIZE, PXRANGE, PXRANGE);
        let arcs = o.compute_near_arcs(2.0);
        let info = o.compute_sdf_tex(arcs, FONT_SIZE, PXRANGE, false, PXRANGE);
        let n = info.tex_size as usize;
        println!("char={:?} tex_size={} atlas={:?} distance={}", ch, n, l.atlas_bounds, l.distance);
        for j in 0..n {
            let mut line = String::new();
            for i in 0..n {
                let v = info.sdf_tex[j * n + i];
                let c = if v == 0 { '.' } else if v < 128 { (b'0' + (v / 26).max(1)) as char } else { '#' };
                line.push(c);
            }
            println!("{}", line);
        }
    }
}