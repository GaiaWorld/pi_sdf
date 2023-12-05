use ab_glyph_rasterizer::{point, Point, Rasterizer};
use allsorts::binary::read::ReadScope;

use allsorts::font::MatchingPresentation;
use allsorts::font_data::FontData;
use allsorts::gsub::RawGlyph;
use allsorts::outline::{OutlineBuilder, OutlineSink};
use allsorts::pathfinder_geometry::line_segment::LineSegment2F;
use allsorts::pathfinder_geometry::vector::Vector2F;
use allsorts::tables::glyf::GlyfTable;
use allsorts::tables::loca::LocaTable;
use allsorts::tables::FontTableProvider;
use allsorts::{tag, Font};
use image::{ImageBuffer, Rgba};
use std::path::Path;
use std::path::PathBuf;

macro_rules! _read_table {
    ($file:ident, $tag:path, $t:ty) => {
        $file
            .read_table($tag, 0)
            .expect("error reading table")
            .expect("no table found")
            .scope()
            .read::<$t>()
            .expect("unable to parse")
    };
}

pub fn fixture_path<P: AsRef<Path>>(path: P) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

/// Read a test fixture from a path relative to CARGO_MANIFEST_DIR
pub fn read_fixture<P: AsRef<Path>>(path: P) -> Vec<u8> {
    std::fs::read(&fixture_path(path)).expect("error reading file contents")
}

struct TestVisitor {
    rasterizer: Rasterizer,
    start: Point,
    previous: Point,
    scale: f32,
}

impl OutlineSink for TestVisitor {
    fn move_to(&mut self, to: Vector2F) {
        println!("move_to({}, {})", to.x(), to.y());
        let p = point(to.x() * self.scale, (to.y() + 500.0) * self.scale);
        self.start = p;
        self.previous = p;
    }

    fn line_to(&mut self, to: Vector2F) {
        println!(
            "line_to({}, {}, {}, {})",
            self.previous.x,
            self.previous.y,
            to.x(),
            to.y()
        );
        let to = point(to.x() * self.scale, (to.y() + 500.0) * self.scale);
        self.rasterizer.draw_line(self.previous, to);
        self.previous = to;
    }

    fn quadratic_curve_to(&mut self, control: Vector2F, to: Vector2F) {
        println!(
            "quad_to({}, {}, {}, {}, {}, {})",
            self.previous.x,
            self.previous.y,
            control.x(),
            control.y(),
            to.x(),
            to.y()
        );
        let control: Point = point(control.x() * self.scale, (control.y() + 500.0) * self.scale);
        let to = point(to.x() * self.scale, (to.y() + 500.0) * self.scale);
        self.rasterizer.draw_quad(self.previous, control, to);
        self.previous = to;
    }

    fn cubic_curve_to(&mut self, control: LineSegment2F, to: Vector2F) {
        println!(
            "curve_to({}, {}, {}, {}, {}, {})",
            control.from_x(),
            control.from_y(),
            control.to_x(),
            control.to_y(),
            to.x(),
            to.y()
        );

        let control_from: Point = point(
            control.from_x() * self.scale,
            (control.from_y() + 500.0) * self.scale,
        );
        let control_to: Point = point(
            control.to_x() * self.scale,
            (control.to_y() + 500.0) * self.scale,
        );
        let to = point(to.x() * self.scale, (to.x() + 500.0) * self.scale);

        self.rasterizer
            .draw_cubic(self.previous, control_from, control_to, to);
        self.previous = to;
    }

    fn close(&mut self) {
        if self.previous != self.start {
            // println!(
            //     "line_to({}, {}, {}, {})",
            //     self.previous.x, self.previous.y, self.start.x, self.start.y
            // );
            self.rasterizer.draw_line(self.previous, self.start)
        }
        // println!("close()");
    }
}

impl TestVisitor {
    pub fn _glyphs_to_path<T>(
        &mut self,
        builder: &mut T,
        glyphs: &[RawGlyph<()>],
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: OutlineBuilder,
        <T as OutlineBuilder>::Error: 'static,
    {
        for glyph in glyphs {
            builder.visit(glyph.glyph_index, self)?;
        }

        Ok(())
    }
}

fn main() {
    let buffer = read_fixture("./source/msyh.ttf");
    let time = std::time::Instant::now();
    let scope = ReadScope::new(&buffer);
    let font_file = scope.read::<FontData<'_>>().unwrap();
    let provider = font_file.table_provider(0).unwrap();
    let mut font = Font::new(provider).unwrap().unwrap();

    let (glyph_index, _) = font.lookup_glyph_index('魔', MatchingPresentation::NotRequired, None);

    let loca_data = font.font_table_provider.read_table_data(tag::LOCA).unwrap();
    let loca = ReadScope::new(&loca_data)
        .read_dep::<LocaTable<'_>>((
            usize::from(font.maxp_table.num_glyphs),
            font.head_table()
                .unwrap()
                .ok_or("missing head table")
                .unwrap()
                .index_to_loc_format,
        ))
        .unwrap();
    let glyf_data = font.font_table_provider.read_table_data(tag::GLYF).unwrap();
    let mut glyf = ReadScope::new(&glyf_data)
        .read_dep::<GlyfTable<'_>>(&loca)
        .unwrap();
    println!("init: {:?}", time.elapsed());

    let w = 512;
    let h = 512;

    let rasterizer = ab_glyph_rasterizer::Rasterizer::new(w, h);
    let mut sink = TestVisitor {
        rasterizer,
        start: point(0.0, 0.0),
        previous: point(0.0, 0.0),
        scale: 0.01,
    };

    // for glyph in glyphs {
    sink.rasterizer.clear();
    let time = std::time::Instant::now();
    let _ = glyf.visit(glyph_index, &mut sink);

    let mut img = ImageBuffer::from_fn(w as u32, h as u32, |_x, _y| Rgba([255u8, 0, 0, 0]));

    sink.rasterizer.for_each_pixel_2d(|x, y, a| {
        let rgba = img.get_pixel_mut(x, h as u32 - y - 1);
        rgba[3] = (a * 255.0) as u8;
    });
    println!("render: {:?}", time.elapsed());
    let _ = img.save("test.png");
}
