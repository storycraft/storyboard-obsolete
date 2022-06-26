use std::error::Error;

use storyboard_text::rasterizer::GlyphRasterizer;
use ttf_parser::Face;

pub static FONT: &'static [u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

#[test]
fn rasterizer_test() -> Result<(), Box<dyn Error>> {
    let mut font = Face::from_slice(FONT, 0)?;

    let index = font.glyph_index('a').unwrap();

    let mut rasterizer = GlyphRasterizer::new(&mut font);
    let rasterized = rasterizer.rasterize(index.0, 16.0).unwrap();

    println!("{rasterized:?}");

    Ok(())
}