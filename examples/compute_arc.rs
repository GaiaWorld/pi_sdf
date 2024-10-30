#![feature(box_into_inner)]
#![feature(test)]
extern crate test;

#[cfg(test)]
mod test_mod {
    use std::sync::Arc;
    use pi_sdf::font::FontFace;
    use test::Bencher;

    #[bench]
    fn performance(b: &mut Bencher) {
        // let buffer = std::fs::read("./source/SOURCEHANSANSK-MEDIUM.TTF").unwrap();
        let buffer = std::fs::read("./source/msyh.ttf").unwrap();
        let mut ft_face = FontFace::new(Arc::new(buffer));
        
        
        log::debug!("max_box_normaliz: {:?}", ft_face.max_box_normaliz());
        let pxrange = 10;
        let time = std::time::Instant::now();
        let mut outline_info = ft_face.to_outline('魔');

        b.iter(move || {
            // log::debug!("===================plane_bounds: {:?}", plane_bounds);
            let result_arcs = outline_info.compute_near_arcs(2.0);
        });
    }
}

fn main() {
    use std::sync::Arc;
    use pi_sdf::font::FontFace;

    // let buffer = std::fs::read("./source/SOURCEHANSANSK-MEDIUM.TTF").unwrap();
    let buffer = std::fs::read("./source/msyh.ttf").unwrap();
    let mut ft_face = FontFace::new(Arc::new(buffer));
    
    
    log::debug!("max_box_normaliz: {:?}", ft_face.max_box_normaliz());
    let pxrange = 10;
    let time = std::time::Instant::now();
    let mut outline_info = ft_face.to_outline('魔');

    loop {
        let time = std::time::Instant::now();
        let result_arcs = outline_info.compute_near_arcs(2.0);
        println!("{:?}", (std::time::Instant::now() - time).as_micros());
    }
}